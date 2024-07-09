use bytes::Buf;
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
    fn parse_type() -> anyhow::Result<()> {
        macro_rules! test_type {
            ($data:tt, $type:tt) => {
                let mut data: &[u8] = &$data;
                assert!(matches!(Type::parse(&mut data), Ok(Type::$type)));
                assert!(data.is_empty());
            };
        }

        test_type!([0, 1], A);
        test_type!([0, 2], NS);
        test_type!([0, 3], MD);
        test_type!([0, 4], MF);
        test_type!([0, 5], CNAME);
        test_type!([0, 6], SOA);
        test_type!([0, 7], MB);
        test_type!([0, 8], MG);
        test_type!([0, 9], MR);
        test_type!([0, 10], NULL);
        test_type!([0, 11], WKS);
        test_type!([0, 12], PTR);
        test_type!([0, 13], HINFO);
        test_type!([0, 14], MINFO);
        test_type!([0, 15], MX);
        test_type!([0, 16], TXT);

        let mut data: &[u8] = &[0, 0];
        assert!(Type::parse(&mut data).is_err());

        let mut data: &[u8] = &[1];
        assert!(Type::parse(&mut data).is_err());
        
        Ok(())
    }

    #[test]
    fn parse_class() -> anyhow::Result<()> {
        macro_rules! test_class {
            ($data:tt, $class:tt) => {
                let mut data: &[u8] = &$data;
                assert!(matches!(Class::parse(&mut data), Ok(Class::$class)));
                assert!(data.is_empty());
            };
        }

        test_class!([0, 1], IN);
        test_class!([0, 2], CS);
        test_class!([0, 3], CH);
        test_class!([0, 4], HS);

        let mut data: &[u8] = &[0, 0];
        assert!(Class::parse(&mut data).is_err());

        let mut data: &[u8] = &[1];
        assert!(Class::parse(&mut data).is_err());

        Ok(())
    }

    #[test]
    fn parse_ttl() -> anyhow::Result<()> {
        let mut data: &[u8] = &[0, 0, 0, 12];
        let ttl = ResourceRecord::parse_ttl(&mut data)?;
        assert_eq!(ttl, 12);

        let mut data: &[u8] = &[0, 12];
        assert!(ResourceRecord::parse_ttl(&mut data).is_err());

        Ok(())
    }

    #[test]
    fn parse_data() -> anyhow::Result<()> {
        let mut buf = Vec::new();
        let data = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        buf.put_u16(data.len() as u16);
        let mut data_copy = data.clone();
        buf.append(&mut data_copy);
        let mut buf = &buf[..];

        let parsed_data = ResourceRecord::parse_data(&mut buf)?;
        assert_eq!(parsed_data, data);

        let buf = vec![10];
        let mut buf = &buf[..];
        let parsed_data = ResourceRecord::parse_data(&mut buf);
        assert!(parsed_data.is_err());

        let mut buf = Vec::new();
        buf.put_u16(data.len() as u16);
        let mut buf = &buf[..];
        let parsed_data = ResourceRecord::parse_data(&mut buf);
        assert!(parsed_data.is_err());


        Ok(())
    }

    #[test]
    fn parse_rr() -> anyhow::Result<()> {
        let mut msg = Vec::new();
        let name = "google.com.";
        let mut name_ser = name::serialize(name, None)?;
        msg.append(&mut name_ser);
        let r#type = Type::NS;
        msg.put_u16(2); // NS
        let class = Class::IN;
        msg.put_u16(1); // IN
        let ttl = 12;
        msg.put_u32(ttl);
        let data = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        msg.put_u16(data.len() as u16);
        let mut data_copy = data.clone();
        msg.append(&mut data_copy);

        let (rr, bytes_parsed) = ResourceRecord::parse(&msg[..], &msg[..])?;
        assert_eq!(rr.name, name);
        assert_eq!(rr.r#type, r#type);
        assert_eq!(rr.class, class);
        assert_eq!(rr.ttl, ttl);
        assert!(
            match rr.data {
                Some(d) if d == data => true,
                _ => false,
            }
        );
        assert_eq!(bytes_parsed, 12 + 2 + 2 + 4 + 2 + 10);

        Ok(())
    }
}
