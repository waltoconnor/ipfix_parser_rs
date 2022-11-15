use crate::parse_data::*;
use crate::template_ring::TemplateRing;
use crate::templates::IPFIXTemplate;

pub enum ParseResult {
    Data(DataSet),
    Template(IPFIXTemplate),
    Error
}

pub fn parse_packet(tring: &TemplateRing, pkt: &[u8]) -> ParseResult {

    

}