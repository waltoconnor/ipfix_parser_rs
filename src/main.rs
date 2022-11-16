
mod templates;
mod template_ring;
mod parse_data;
mod parse_packet;
mod executor;
mod config;

use std::{net::Ipv4Addr, time::Duration};

pub use executor::IPFIXCollectorHandle;
pub use config::Config;

fn main() {
    let cfg = Config {
        ipfix_listen_addr: Ipv4Addr::new(127, 0, 0, 1),
        ipfix_listen_port: 64000,
        num_threads: 32
    };

    let collector = IPFIXCollectorHandle::start(&cfg);
    //TODO: ADD WAY TO ACCESS DATA STORED IN THE COLLECTOR
    println!("Collector Started on {:?}:{} with {} parser threads", cfg.ipfix_listen_addr, cfg.ipfix_listen_port, cfg.num_threads);

    loop {
        std::thread::sleep(Duration::from_secs(1));
    }

    //collector.stop();
}
