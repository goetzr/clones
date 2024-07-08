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
        let class = Class::parse(&mut unparsed)?;
        let ttl = Self::parse_ttl(&mut unparsed)?;
        let data = Self::parse_data(&mut unparsed)?;
        
        let bytes_parsed = Self::calc_num_bytes_parsed(unparsed, rr);
        let rr = ResourceRecord { name, r#type, class, ttl, data: Some(data) };
        Ok((rr, bytes_parsed))
    }

    fn parse_ttl(unparsed: &mut &[u8]) -> anyhow::Result<u32> {
        if unparsed.remaining() < 4 {
            anyhow::bail!("incomplete TTL");
        }
        Ok(unparsed.get_u32())
    }

    fn parse_data(unparsed: &mut &[u8]) -> anyhow::Result<Vec<u8>> {
        if unparsed.remaining() < 2 {
            anyhow::bail!("incomplete data length");
        }
        let data_len = unparsed.get_u16() as usize;
        let mut data = vec![0; data_len];
        if unparsed.remaining() < data_len {
            anyhow::bail!("incomplete data");
        }
        unparsed.copy_to_slice(&mut data[..]);
        Ok(data)
    }

    fn calc_num_bytes_parsed(unparsed: &[u8], rr: &[u8]) -> usize {
        unsafe {
            unparsed.as_ptr().offset_from(rr.as_ptr()) as usize
        }
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
        if buf.remaining() < 2 {
            anyhow::bail!("incomplete type");
        }
        match buf.get_u16() {
            1 => Ok(Type::A),
            2 => Ok(Type::NS),
            3 => Ok(Type::MD),
            4 => Ok(Type::MF),
            5 => Ok(Type::CNAME),
            6 => Ok(Type::SOA),
            7 => Ok(Type::MB),
            8 => Ok(Type::MG),
            9 => Ok(Type::MR),
            10 => Ok(Type::NULL),
            11 => Ok(Type::WKS),
            12 => Ok(Type::PTR),
            13 => Ok(Type::HINFO),
            14 => Ok(Type::MINFO),
            15 => Ok(Type::MX),
            16 => Ok(Type::TXT),
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

impl Class {
    fn parse(buf: &mut &[u8]) -> anyhow::Result<Self> {
        if buf.remaining() < 2 {
            anyhow::bail!("incomplete class");
        }
        match buf.get_u16() {
            1 => Ok(Class::IN),
            2 => Ok(Class::CS),
            3 => Ok(Class::CH),
            4 => Ok(Class::HS),
            n => Err(anyhow::anyhow!("invalid RR class '{n}'")),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use bytes::BufMut;
    use tracing_test::traced_test;
    use crate::name;

    #[test]
    fn parse_rr() -> anyhow::Result<()> {
        let mut rr = Vec::new();
        let mut name = name::serialize("google.com.", None).expect("serialize name");
        rr.append(&mut name);
        rr.put_u16(2);  // type = NS
        rr.put_u16(1);  // class = IN
        rr.put_u32(10); // ttl = 10
        let mut data = Vec::new();
        for i in 1..11 {
            data.push(i);
        }
        let data_copy = data.clone();
        rr.put_u16(data.len() as u16);
        rr.append(&mut data);

        let (parsed_rr, bytes_parsed) = ResourceRecord::parse(&rr[..], &rr[..])?;
        assert_eq!(parsed_rr.name, "google.com.");
        assert_eq!(parsed_rr.r#type, Type::NS);
        assert_eq!(parsed_rr.class, Class::IN);
        assert_eq!(parsed_rr.ttl, 10);
        let data_matches = match parsed_rr.data {
            Some(parsed_data) => parsed_data == data_copy,
            None => false,
        };
        assert!(data_matches);
        assert_eq!(bytes_parsed, rr.len());

        Ok(())
    }

    #[test]
    fn parse_type() -> anyhow::Result<()> {
        
        Ok(())
    }

    #[test]
    fn parse_class() -> anyhow::Result<()> {
        Ok(())
    }
}
