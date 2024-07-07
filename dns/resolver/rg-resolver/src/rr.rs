use anyhow::Context;
use bytes::{Buf, BufMut};
use tracing::debug;
use crate::name;

pub struct ResourceRecord {
    pub name: String,
    pub r#type: Type,
    pub class: Class,
    pub ttl: u32,
    pub data: Option<Vec<u8>>,
}

impl ResourceRecord {
    pub fn parse<'a>(msg: &'a [u8], rr: &'a [u8]) -> anyhow::Result<(ResourceRecord, usize)> {
        let mut unparsed = &rr[..];

        let mut name_parser = name::Parser::new(msg, unparsed);
        let (name, bytes_parsed) = name_parser.parse()?;
        unparsed.advance(bytes_parsed);

        let r#type = Type::parse(&mut unparsed)?;
        let class = Class::IN;
        let ttl = 10;
        let mut data = Vec::new();
        for i in 1..11 {
            data.push(i);
        }
        let rr = ResourceRecord { name, r#type, class, ttl, data: Some(data) };
        Ok((rr, 10))
    }
}

#[derive(Debug, PartialEq)]
pub enum Type {
    A,
    NS,
    MD,
    MF,
    CNAME,
    SOA,
    MB,
    MG,
    MR,
    NULL,
    WKS,
    PTR,
    HINFO,
    MINFO,
    MX,
    TXT,
}

impl Type {
    fn parse(buf: &mut &[u8]) -> anyhow::Result<Self> {
        match buf.get_u16() {
            0 => Ok(Type::A),
            1 => Ok(Type::NS),
            2 => Ok(Type::MD),
            3 => Ok(Type::MF),
            4 => Ok(Type::CNAME),
            5 => Ok(Type::SOA),
            6 => Ok(Type::MB),
            7 => Ok(Type::MG),
            8 => Ok(Type::MR),
            9 => Ok(Type::NULL),
            10 => Ok(Type::WKS),
            11 => Ok(Type::PTR),
            12 => Ok(Type::HINFO),
            13 => Ok(Type::MINFO),
            14 => Ok(Type::MX),
            15 => Ok(Type::TXT),
            n => Err(anyhow::anyhow!("invalid RR type '{n}'")),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum Class {
    IN,
    CS,
    CH,
    HS,
}

#[cfg(test)]
mod test {
    use super::*;
    use bytes::BufMut;
    use tracing_test::traced_test;
    use crate::name;

    #[test]
    fn parse() -> anyhow::Result<()> {
        let mut rr = Vec::new();
        let mut name = name::serialize("google.com.", None).expect("serialize name");
        rr.append(&mut name);
        rr.put_u16(1);  // type = NS
        rr.put_u16(0);  // class = IN
        rr.put_u32(10); // ttl = 10
        let mut data = Vec::new();
        for i in 1..11 {
            data.push(i);
        }
        rr.put_u16(data.len() as u16);
        rr.append(&mut data);

        let (parsed_rr, bytes_parsed) = ResourceRecord::parse(&rr[..], &rr[..])?;
        assert_eq!(parsed_rr.name, "google.com.");
        assert_eq!(parsed_rr.r#type, Type::NS);
        assert_eq!(parsed_rr.class, Class::IN);
        assert_eq!(parsed_rr.ttl, 10);
        let data_matches = match parsed_rr.data {
            Some(parsed_data) => parsed_data == data,
            None => false
        };
        assert!(data_matches);
        assert_eq!(bytes_parsed, rr.len());

        Ok(())
    }
}
