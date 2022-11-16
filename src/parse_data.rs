use crate::template_ring::TemplateRing;

use nom::{number::complete::be_u16, error::VerboseError, Slice};


pub enum DataType {
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    BYTES(Vec<u8>)
}

pub struct DataRow {
    pub id: u16,
    pub en: u32,
    pub data: DataType
}

impl DataRow {
    pub fn new(id: u16, en: u32, val: DataType) -> Self {
        DataRow { id, en, data: val }
    }
}

pub struct DataSet {
    pub id: u16,
    pub template: u16,
    pub fields: Vec<DataRow>
}

fn read_u8(buf: &[u8]) -> u8 {
    buf[0]
}

fn read_u16(buf: &[u8]) -> u16 {
    ((buf[0] as u16) << 8) + (buf[1] as u16)
}

fn read_u32(buf: &[u8]) -> u32 {
    ((buf[0] as u32) << 24) + ((buf[1] as u32) << 16 ) + ((buf[2] as u32) << 8) + (buf[3] as u32)
}

fn read_u64(buf: &[u8]) -> u64 {
    (buf[0] as u64) << 56 
    + (buf[1] as u64) << 48
    + (buf[2] as u64) << 40
    + (buf[3] as u64) << 32
    + (buf[4] as u64) << 24
    + (buf[5] as u64) << 16
    + (buf[6] as u64) << 40
    + (buf[7] as u64)
}

fn read_other(buf: &[u8], width: usize) -> Vec<u8> {
    Vec::from(buf.slice(0..width))
}

fn read_value_from_byte(i: &[u8], offset: usize, width: u16) -> Result<DataType, String> {
    //get a view of the buffer that starts with at the provided offset
    let buf = i.slice(offset..i.len());

    //if the buffer after the offset isn't big enough, we can't read it
    if buf.len() <= width.into() {
        return Result::Err(format!("Field at {} wants {} bytes, but only {} are left in the buffer", offset, width, buf.len()));
    }

    //extract the value
    Ok(match width {
        8 => DataType::U64(read_u64(buf)),
        4 => DataType::U32(read_u32(buf)),
        2 => DataType::U16(read_u16(buf)),
        1 => DataType::U8(read_u8(buf)),
        n => DataType::BYTES(read_other(buf, n.into()))
    })
}

impl DataSet {
    //expects the first byte of i to be the first byte of the data packet set id
    pub fn get_datasets<'a>(i: &'a [u8], tmp_ring: &TemplateRing, odid: u32) -> Result<(&'a [u8], Vec<Self>), String> {
        //take data set header
        let (rest, set_id) = be_u16::<&[u8], VerboseError<&[u8]>>(i)
            .or(Result::Err(String::from("Failed to parse data set set_id")))?;

        //take length
        let (rest, len) = be_u16::<&[u8], VerboseError<&[u8]>>(rest)
            .or(Result::Err(String::from("Failed to parse data set len")))?;

        //get the template
        let template = match tmp_ring.get_template(set_id, odid) {
            None => { return Result::Err(format!("No template with ID {} in template set", set_id)); },
            Some(t) => t
        };

        //keep going until we run out of records
        let mut bytes_read = 0;
        let mut loop_rest = rest;
        let mut cur;
        let mut datasets = Vec::new();
        while bytes_read < len {
            //multiple "instances" of templates come in each data packet, read one here
            (cur, loop_rest) = loop_rest.split_at(len as usize);
            let mut ds = DataSet { id: set_id, fields: Vec::new(), template: template.id };

            for tmplt_field in template.fields.iter() {
                let val = read_value_from_byte(cur, tmplt_field.start_byte as usize, tmplt_field.width)?;
                bytes_read += tmplt_field.width;
                ds.fields.push(DataRow::new(tmplt_field.field_id, tmplt_field.en, val));

            }
            datasets.push(ds);
        }

        Ok((loop_rest, datasets))
    }
}

