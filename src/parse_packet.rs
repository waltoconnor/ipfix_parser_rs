use nom::error::VerboseError;
use nom::number::complete::{be_u16, be_u32};
use nom::Slice;

use crate::parse_data::*;
use crate::template_ring::TemplateRing;
use crate::templates::IPFIXTemplate;

pub enum PacketResult {
    Ok(PacketInfo),
    AbortError
}

pub struct PacketInfo {
    pub export_time: u32,
    pub seq_num: u32,
    pub templates: Vec<IPFIXTemplate>,
    pub data: Vec<DataSet>,
    pub set_error_count: u32,
    pub odid: u32
}

impl PacketInfo {
    fn new(export_time: u32, seq_num: u32, odid: u32, parse_results: Vec<ParseResult>) -> PacketResult {
        let mut templates = Vec::new();
        let mut data = Vec::new();
        let mut set_error_count: u32 = 0;

        for r in parse_results {
            match r {
                ParseResult::AbortError => { return PacketResult::AbortError; },
                ParseResult::Error => { set_error_count += 1; }
                ParseResult::Template(t) => { templates.push(t); }
                ParseResult::Data(d) => { data.extend(d); }
            }
        }

        PacketResult::Ok(PacketInfo { 
            export_time: export_time, 
            seq_num: seq_num, 
            templates: templates, 
            data: data, 
            set_error_count: set_error_count,
            odid: odid
        })
        
    }
}

enum ParseResult {
    Data(Vec<DataSet>),
    Template(IPFIXTemplate),
    Error, //error where we can keep reading the packet
    AbortError //error where we can NOT keep reading the packet
}

fn handle_set<'a>(set_head: &'a [u8], odid: u32, tring: &TemplateRing) -> (ParseResult, u16, &'a [u8]) {
    let (id_rest, set_id) = match be_u16::<&[u8], VerboseError<&[u8]>>(set_head) {
        Ok(v) => v,
        Err(_e) => { return (ParseResult::AbortError, 0, set_head); }
    };

    let (_len_rest, set_len) = match be_u16::<&[u8], VerboseError<&[u8]>>(id_rest) {
        Ok(v) => v,
        Err(_e) => {return (ParseResult::AbortError, 0, id_rest); }
    };

    let data;
    let next;
    if set_id == 2 {
        (next, data) = match IPFIXTemplate::from(set_head, odid) {
            Ok((n, t)) => (n, ParseResult::Template(t)),
            Err(_e) => (set_head.slice((set_len as usize)..set_head.len()), ParseResult::Error)
        };
    }
    else {
        (next, data) = match DataSet::get_datasets(set_head, tring, odid) {
            Ok((n, d)) => (n, ParseResult::Data(d)),
            Err(_e) => (set_head.slice((set_len as usize)..set_head.len()), ParseResult::Error)
        };
    }

    return (data, set_len, next);
}

pub fn parse_packet(tring: &TemplateRing, pkt: &[u8]) -> PacketResult {
    let (rest, _version) = match be_u16::<&[u8], VerboseError<&[u8]>>(pkt) {
        Ok(v) => v,
        Err(_e) => { return PacketResult::AbortError; }
    };

    let (rest, len) = match be_u16::<&[u8], VerboseError<&[u8]>>(rest) {
        Ok(v) => v,
        Err(_e) => {return PacketResult::AbortError; }
    };

    let (rest, export_time) = match be_u32::<&[u8], VerboseError<&[u8]>>(rest) {
        Ok(v) => v,
        Err(_e) => { return PacketResult::AbortError; }
    };

    let (rest, seq_num) = match be_u32::<&[u8], VerboseError<&[u8]>>(rest) {
        Ok(v) => v,
        Err(_e) => { return PacketResult::AbortError; }
    };

    let (rest, odid) = match be_u32::<&[u8], VerboseError<&[u8]>>(rest) {
        Ok(v) => v,
        Err(_e) => { return PacketResult::AbortError; }
    };

    let mut result_vec = Vec::new();

    let body_len = len - 32;
    let mut bytes_read: u32 = 0;
    let mut loop_rest = rest;
    let mut cur_data;
    while bytes_read < body_len as u32 {
        let set_len;
        //process the set
        (cur_data, set_len, loop_rest) = handle_set(loop_rest, odid, tring);
        bytes_read += set_len as u32;

        match cur_data { 
            ParseResult::AbortError => { return PacketResult::AbortError; },
            _ => {}
        };

        result_vec.push(cur_data);
    }

    PacketInfo::new(export_time, seq_num, odid, result_vec)

}