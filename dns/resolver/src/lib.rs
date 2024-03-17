use bytes::buf::{Buf, BufMut};

pub fn encode_name(name: &str) -> Vec<u8> {
    let mut qname = Vec::new();

    for label in name.split('.') {
        qname.put_u8(label.len() as u8);
        qname.put_slice(label.as_bytes());
    }
    qname.put_u8(0);

    qname
}

pub struct Header {
    pub id: u16,
    pub qr: MessageType,
    /*
    pub opcode: u16 = 0; // standard query
    pub aa: u16 = 0; // authoritative answer: ignored in query
    pub tc: u16 = 0; // truncation: ignore in query
    pub rd: u16 = 0; // recursion not desired
    pub ra: u16 = 0; // recursion available: ignore in query
    pub z: u16 = 0; // must be zero
    pub rcode: u16 = 0; // response code: ignore in query
    pub qdcount = 1; // number of entries in question section
    pub ancount = 0; // number of entries in answer section: ignored in query
    pub nscount = 0; // number of entries in authority section: ignored in query
    pub arcount = 0; // number of entries in additional record section: ignored in query
    */
}

pub enum MessageType {
    Query,
    Response,
}

impl MessageType {
    pub fn to_int(&self) -> u16 {
        match self {
            MessageType::Query => 0,
            MessageType::Response => 1,
        }
    }

    pub fn from_int(num: u16) -> Result<MessageType> {
        match num {
            0 => Ok(MessageType::Query),
            1 => Ok(MessageType::Response),
            _ => invalid_field_value!(
                    "
        }
    }
}

pub enum Error {
    InvalidFieldValue {
        field_name: String,
        message: String,
    },
}

impl std::error::Error for Error {}

pub type Result<T> = std::result::Result<T, Error>;

macro_rules! invalid_field_value(

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn encode_name1() {
        let encoded = encode_name("google.com");

        let mut expected = Vec::new();
        expected.push(6);
        expected.extend_from_slice(b"google");
        expected.push(3);
        expected.extend_from_slice(b"com");
        expected.push(0);

        assert_eq!(encoded, expected);
    }
}
