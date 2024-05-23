use bytes::buf::{Buf, BufMut};
use std::fmt;

macro_rules! invalid_field_value {
    ($field_name:literal, $value:expr) => {
        Err(Error::InvalidFieldValue {
            field_name: $field_name.to_string(),
            value: $value,
        })
    }
}

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
    pub qr: QR,
    pub opcode: Opcode,
    /*
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

#[derive(Debug)]
pub enum QR {
    Query,
    Response,
}

impl QR {
    const POS: usize = 15;
    const WIDTH: u16 = 1;

    pub fn build(&self) -> u16 {
        match self {
            QR::Query => 0,
            QR::Response => 1 << QR::POS,
        }
    }

    pub fn parse(word: u16) -> QR {
        match (word >> QR::POS) & QR::WIDTH  {
            0 => QR::Query,
            1 => QR::Response,
            _ => QR::Query, // Not possible
        }
    }
}

#[derive(Debug)]
pub enum Opcode {
    Query,
    IQuery,
    Status,
}

impl Opcode {
    const POS: usize = 11;
    const WIDTH: u16 = 4;

    pub fn build(&self) -> u16 {
        match self {
            Opcode::Query => 0,
            Opcode::IQuery => 1 << Opcode::POS,
            Opcode::Status => 2 << Opcode::POS,
         }
    }

    pub fn parse(word: u16) -> Result<Opcode> {
        match (word >> Opcode::POS) & Opcode::WIDTH {
            0 => Ok(Opcode::Query),
            1 => Ok(Opcode::IQuery),
            2 => Ok(Opcode::Status),
            val => invalid_field_value!("Opcode", val),
        }
    }
}

// TODO: Impl default for all these enums.
#[derive(Debug)]
pub enum AA {
    NonAuthoritative,
    Authoritative,
}

impl AA {
    const POS: usize = 15;
    const WIDTH: u16 = 1;

    pub fn build(&self) -> u16 {
        match self {
            QR::Query => 0,
            QR::Response => 1 << QR::POS,
        }
    }

    pub fn parse(word: u16) -> QR {
        match (word >> QR::POS) & QR::WIDTH  {
            0 => QR::Query,
            1 => QR::Response,
            _ => QR::Query, // Not possible
        }
    }
}

#[derive(Debug)]
pub enum Error {
    InvalidFieldValue {
        field_name: String,
        value: u16,
    },
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::InvalidFieldValue { field_name, value } => {
                write!(f, "{value} is an invalid value for the '{field_name}' field")
            },
        }
    }
    
}

impl std::error::Error for Error {}

pub type Result<T> = std::result::Result<T, Error>;


#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn encode_name_simple() {
        let encoded = encode_name("google.com");

        let mut expected = Vec::new();
        expected.push(6);
        expected.extend_from_slice(b"google");
        expected.push(3);
        expected.extend_from_slice(b"com");
        expected.push(0);

        assert_eq!(encoded, expected);
    }

    #[test]
    fn build_qr() {
        let query = QR::Query;
        assert_eq!(query.build(), 0);

        let response = QR::Response;
        assert_eq!(query.build(), 0x8000);
    }

}
