use nom::{number::complete::{be_u16, be_u32}, error::VerboseError};

#[derive(Clone)]
pub struct IPFIXTemplate {
    pub id: u16,
    pub odid: u32,
    pub fields: Vec<IPFIXField>
}

#[derive(Clone)]
pub struct IPFIXField {
    pub width: u16,
    pub start_byte: u32,
    pub en: u32,
    pub field_id: u16
}

impl IPFIXTemplate {

    //This function wants a set of bytes where the first byte in this string is the first byte of the template id for a given template
    //It will then return the parsed template, plus the first byte that appears after this template (nominally the first byte of the next template's template id)
    pub fn from(i: &[u8], odid: u32) -> Result<(&[u8], Self), String> {
        //this can be minimum 4 bytes: template id, field count
        if i.len() < 4 {
            return Result::Err(format!("template packet is below the minimum size ({} bytes found, 4 needed)", i.len()));
        }

        //get set id
        let (rest, set_id) = match be_u16::<&[u8], VerboseError<&[u8]>>(i) {
            Ok(val) => Result::Ok(val),
            Err(_) => Result::Err(String::from("failed to parse set_id"))
        }?;

        if set_id != 2 {
            return Result::Err(format!("template packet parser was given bytes that do not appear to be a template packet"))
        }

        //get set template length
        let (_rest, len) = match be_u16::<&[u8], VerboseError<&[u8]>>(rest) {
            Ok(v) => Result::Ok(v),
            Err(_) => Result::Err(String::from("failed to parse template packet length"))
        }?;

        //if there are less bytes in our working array than the packet says it has, the template is incomplete
        if i.len() < len as usize {
            return Result::Err(format!("Parsing template packet with set_id: {} failed, needed {} bytes, found {}", set_id, len, i.len()))
        }

        //get template id
        let (rest, template_id) = match be_u16::<&[u8], VerboseError<&[u8]>>(i) {
           Ok(v) => Result::Ok(v),
           Err(_) => Result::Err(String::from("failed to parse template packet id"))
        }?;


        //get the field count
        let (rest, field_count) = match be_u16::<&[u8], VerboseError<&[u8]>>(rest){
            Ok(v) => Ok(v),
            Err(_) => Err(String::from("failed to parse template packet field count"))
        }?;

        //make the template with the template id
        let mut template = IPFIXTemplate {
            id: template_id,
            odid: odid,
            fields: Vec::new()
        };

        //loop through the fields and add them to the template
        let mut cur_byte = 0; //keep track of where the first byte of the field is in the data packet
        //the rest of our system wants to work in an offset + width world, but IPFIX templates just have a bunch of widths, so accumulate the widths to give us an offset

        let mut loop_rest = rest;
        for i in 0..field_count {
            let loop_id: u16;
            let loop_len: u16;
            let mut loop_en: u32 = 0;

            //get the id
            (loop_rest, loop_id) = match be_u16::<&[u8], VerboseError<&[u8]>>(loop_rest) {
                Ok(v) => Ok(v),
                Err(_) => Err(format!("failed to parse template field id for field {}", i))
            }?;

            //get the size
            (loop_rest, loop_len) = match be_u16::<&[u8], VerboseError<&[u8]>>(loop_rest) {
                Ok(v) => Ok(v),
                Err(_) => Err(format!("failed to parse field length for template field {}", i))
            }?;

            //check if there is an enterprise number and grab it, otherwise it defaults to 0
            if loop_id & 0x8000u16 > 0 { //one bit at the head of the ID means there is an enterprise number
                (loop_rest, loop_en) = match be_u32::<&[u8], VerboseError<&[u8]>>(loop_rest) {
                    Ok(v) => Ok(v),
                    Err(_) => Err(format!("failed to parse field length for template field {}", i))
                }?;
            }

            let loop_id_no_en_bit = loop_id & !0x8000u16; //mask off the bit for en number

            let row = IPFIXField {
                field_id: loop_id_no_en_bit,
                width: loop_len,
                start_byte: cur_byte,
                en: loop_en
            };

            template.fields.push(row);
            cur_byte += loop_len as u32; //update the offset with the current width
        }

        Ok((loop_rest, template))

    }
}