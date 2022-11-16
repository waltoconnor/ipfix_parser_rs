use std::net::Ipv4Addr;


#[derive(Clone)]
pub struct Config {
    pub ipfix_listen_addr: Ipv4Addr,
    pub ipfix_listen_port: u16,
    pub num_threads: u32
}