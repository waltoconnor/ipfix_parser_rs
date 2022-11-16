use std::collections::HashMap;
use std::sync::mpsc::{Receiver, Sender, self};
use std::thread::{Thread, self};

use crate::config::Config;
use crate::parse_packet::{PacketResult, PacketInfo, parse_packet};
use crate::templates::IPFIXTemplate;
use crate::template_ring::{TemplateRing, self};

pub struct IPFIXCollectorHandle {
    coordinator: Sender<MsgToCoordinatorThread>,
    parsers: Vec<Sender<MsgToParserThread>>,
    aggregator: Sender<MsgToAggregatorThread>
}

impl IPFIXCollectorHandle {
    pub fn start(config: &Config) -> Self {
        let (agg_tx, agg_rx) = mpsc::channel();
        thread::spawn(move ||{ agg_thread(agg_rx); });

        //need this early so parsers can talk with coordinator for template updates
        let (coord_tx, coord_rx) = mpsc::channel();

        let mut parser_threads_recs = Vec::new();
        for i in 0..config.num_threads {
            let (tx, rx) = mpsc::channel();
            parser_threads_recs.push(tx);

            let agg_sender_clone = agg_tx.clone();
            let coord_sender_clone = coord_tx.clone();

            thread::spawn(move ||{ parser_thread(i, rx, coord_sender_clone, agg_sender_clone); });
        }

        let cfg_clone = (*config).clone();
        let parser_threads_clone = parser_threads_recs.clone();
        thread::spawn(move ||{ coord_thread(coord_rx, parser_threads_clone, cfg_clone); });

        IPFIXCollectorHandle { coordinator: coord_tx, parsers: parser_threads_recs, aggregator: agg_tx }
    }

    pub fn stop(&mut self) {
        self.coordinator.send(MsgToCoordinatorThread::STOP).expect("Failed to stop coordinator thread");
        self.aggregator.send(MsgToAggregatorThread::STOP).expect("Failed to stop aggregator thread");
        for t in self.parsers.iter() {
            t.send(MsgToParserThread::STOP).expect("Failed to stop parser thread");
        }
    }
}

//INTER THREAD MESSAGES
enum MsgToParserThread {
    STOP, //stops thread
    TEMPLATE(IPFIXTemplate), //send new template to parser thread to add to parser ring
    WORK(Box<[u8]>) //packet that arrived
}

enum MsgToCoordinatorThread {
    STOP, //stops thread
    NEW_TEMPLATE(IPFIXTemplate) //parser thread found a new template and needs everyone to be updated
}

enum MsgToAggregatorThread {
    RESULT(PacketInfo), //parser thread finished picking apart a packet
    STOP //stops thread
}

fn parser_thread(idx: u32, parser_rec: Receiver<MsgToParserThread>, coord_snd: Sender<MsgToCoordinatorThread>, agg_snd: Sender<MsgToAggregatorThread>) {
    let mut templates = TemplateRing::new();

    loop {
        match parser_rec.recv().expect(format!("Thread {} failed to receive message from coordinator", idx).as_str()) {
            MsgToParserThread::STOP => { return; },
            MsgToParserThread::TEMPLATE(t) => { 
                let odid = t.odid;
                templates.insert_template(t, odid); 
            },
            MsgToParserThread::WORK(pkt) => {
                match parse_packet(&templates, &pkt) {
                    PacketResult::AbortError => { eprintln!("Thread {} failed to parse a full packet", idx); },
                    PacketResult::Ok(info) => {
                        for t in info.templates.iter() {
                            coord_snd.send(MsgToCoordinatorThread::NEW_TEMPLATE(t.clone())).expect("Failed to send template to coordinator");
                        }
                        agg_snd.send(MsgToAggregatorThread::RESULT(info)).expect("Failed to send message to aggregator");
                    }
                }
            }//end work block
        }//end match
    }//end loop
}

//coordinator thread, passes work and new templates to each parser thread
fn coord_thread(coord_rec: Receiver<MsgToCoordinatorThread>, parser_threads: Vec<Sender<MsgToParserThread>>, cfg: Config) {
    let timeout = std::time::Duration::from_millis(50);
    let socket = std::net::UdpSocket::bind(format!("{}:{}", cfg.ipfix_listen_addr, cfg.ipfix_listen_port)).expect("Failed to open IPFIX listen socket");
    socket.set_read_timeout(Some(timeout)).expect("Failed to set socket timeout");

    let mut cur_parser_thread = 0;

    let mut buf = [0u8; 10000]; //This just needs to be larger than the max sized IPFIX report, and reports are capped in size by the MTU of the link they travel across

    loop {
        match socket.recv_from(&mut buf) {
            Err(_e) => match coord_rec.try_recv() { //happens when the socket times out
                Ok(msg) => match msg { //see if we have a stop or template message waiting
                    MsgToCoordinatorThread::STOP => { return; },
                    MsgToCoordinatorThread::NEW_TEMPLATE(tmp) => {
                        for thread in parser_threads.iter() {
                            thread.send(MsgToParserThread::TEMPLATE(tmp.clone())).expect("Could not send template to parser thread");
                        }
                    }
                },
                Err(_e) => { continue; } //nothing in coord rec queue
            },
            Ok((count, _sock_addr)) => {
                let trimmed_buf = &buf[..count];
                let mut vec: Vec<u8> = Vec::with_capacity(count);
                vec.extend_from_slice(&trimmed_buf);
                let boxed_buf = vec.into_boxed_slice();
                parser_threads[cur_parser_thread].send(MsgToParserThread::WORK(boxed_buf)).expect("Could not send work to parser thread");
                cur_parser_thread = (cur_parser_thread + 1) % parser_threads.len();
            }
        }
    }
}

//aggregator thread: receives data from parser threads and stores it in a hashmap as a vector of datasets per ODID
fn agg_thread(agg_rec: Receiver<MsgToAggregatorThread>) {
    let mut odid_map = HashMap::<u32, Vec<PacketInfo>>::new();

    loop {
        match agg_rec.recv().expect("Aggregator failed to receive message") {
            MsgToAggregatorThread::STOP => { return; },
            MsgToAggregatorThread::RESULT(d) => {
                let odid = d.odid;
                match odid_map.get_mut(&odid) {
                    None => { odid_map.insert(odid, Vec::from([d])); },
                    Some(v) => { v.push(d); }
                }
            }
        }
    }
}
