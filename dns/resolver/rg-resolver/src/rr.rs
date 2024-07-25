use crate::name;
use bytes::{Buf, BufMut};
use std::net::Ipv4Addr;
use anyhow::Context;

#[derive(Debug, Clone, PartialEq)]
pub struct ResourceRecord {
    pub name: String,
    pub r#type: Type,
    pub class: Class,
    pub ttl: i32,
    pub data: Data,
}

impl ResourceRecord {
    /// * msg must point to the very first byte of the message.
    pub fn parse<'a>(msg: &'a [u8], unparsed: &mut &'a [u8]) -> anyhow::Result<ResourceRecord> {
        let name = name::parse(msg, unparsed)?;
        let r#type = Type::parse(unparsed)?;
        let class = Class::parse(unparsed)?;
        let ttl = Self::parse_ttl(unparsed)?;
        // TODO: Need to parse the type-specific data at this point.
        // TODO: msg is required if we need to parse any names.
        // TODO: Maybe the best thing to do it combine the r#type field and the data field
        // TODO: into a single data field that's a type-based enum:
        // TODO: enum Data {
        // TODO:    A(Ipv4Addr),
        // TODO:    NS(String),
        // TODO:    ...
        // TODO: }
        let data = Self::parse_data(unparsed)?;

        let rr = ResourceRecord {
            name,
            r#type,
            class,
            ttl,
            data: Some(data),
        };
        Ok(rr)
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

    /// * For a nameserver that needs to create ResourceRecord instances and serialize them,
    /// * it will ideally keep track of the names it's generated thus far,
    /// * and for every new name it needs to generate see if it's a superset of a
    /// * previously generated name and should be compressed.
    /// * For a resolver, the only name it needs to generate is the question name,
    /// * which is always the first name in the message so it can't be compressed.
    /// * Because only the resolver is being implemented at this point, and serialization
    /// * of ResourceRecord instances is only being implemented to test the
    /// * parsing of Message instances, simply serialize the name of each
    /// * ResourceRecord instance as an uncompressed name.
    pub fn serialize(&self) -> anyhow::Result<Vec<u8>> {
        let mut buf = Vec::new();
        buf.append(&mut name::serialize(&self.name, None)?);
        buf.put_u16(self.r#type.serialize());
        buf.put_u16(self.class.serialize());
        buf.put_u32(self.ttl);
        if let Some(data) = self.data.as_ref() {
            buf.put_u16(data.len() as u16);
            buf.append(&mut data.clone());
        } else {
            buf.put_u16(0);
        }
        Ok(buf)
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
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
    pub fn parse(unparsed: &mut &[u8]) -> anyhow::Result<Self> {
        if unparsed.remaining() < 2 {
            anyhow::bail!("incomplete type");
        }
        match unparsed.get_u16() {
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

    pub fn serialize(&self) -> u16 {
        use Type::*;

        match self {
            A => 1,
            NS => 2,
            MD => 3,
            MF => 4,
            CNAME => 5,
            SOA => 6,
            MB => 7,
            MG => 8,
            MR => 9,
            NULL => 10,
            WKS => 11,
            PTR => 12,
            HINFO => 13,
            MINFO => 14,
            MX => 15,
            TXT => 16,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Class {
    IN,
    CS,
    CH,
    HS,
}

impl Class {
    pub fn parse(unparsed: &mut &[u8]) -> anyhow::Result<Self> {
        if unparsed.remaining() < 2 {
            anyhow::bail!("incomplete class");
        }
        match unparsed.get_u16() {
            1 => Ok(Class::IN),
            2 => Ok(Class::CS),
            3 => Ok(Class::CH),
            4 => Ok(Class::HS),
            n => Err(anyhow::anyhow!("invalid RR class '{n}'")),
        }
    }

    pub fn serialize(&self) -> u16 {
        use Class::*;

        match self {
            IN => 1,
            CS => 2,
            CH => 3,
            HS => 4,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Data {
    A(Ipv4Addr),
    NS(String),
    MD(String),
    MF(String),
    CNAME(String),
    SOA {
        mname: String,
        rname: String,
        serial: u32,
        refresh: u32,
        retry: u32,
        expire: u32,
        minimum: i32,
    },
    MB(String),
    MG(String),
    MR(String),
    NULL(Vec<u8>),
    WKS {
        address: Ipv4Addr,
        protocol: u8,
        bit_map: Vec<u8>,
    },
    PTR(String),
    HINFO {
        cpu: String,
        os: String,
    },
    MINFO {
        rmailbx: String,
        emailbx: String,
    },
    MX {
        preference: i16,
        exchange: String,
    },
    TXT(Vec<String>),
}

impl Data {
    pub fn parse(msg: &[u8], unparsed: &mut &[u8], r#type: Type) -> anyhow::Result<Self> {
        if unparsed.remaining() < 2 {
            anyhow::bail!("incomplete data length");
        }
        let data_len = unparsed.get_u16() as usize;
        if unparsed.remaining() < data_len {
            anyhow::bail!("incomplete data");
        }
        let data = &(*unparsed)[..data_len];
        unparsed.advance(data_len);
        let mut data = &data[..];

        match r#type {
            Type::A => {
                if data_len != 4 {
                    anyhow::bail!("type A RR data not 4 bytes");
                }
                let addr = Ipv4Addr::new(data[0], data[1], data[2], data[3]);
                Ok(Data::A(addr))
            }
            Type::NS => {
                let name = name::parse(msg, &mut data).with_context(|| "type NS RR")?;
                Ok(Data::NS(name))
            }
            Type::MD => {
                let name = name::parse(msg, &mut data).with_context(|| "type MD RR")?;
                Ok(Data::MD(name))
            }
            Type::MF => {
                let name = name::parse(msg, &mut data).with_context(|| "type MF RR")?;
                Ok(Data::MF(name))
            }
            Type::CNAME => {
                let name = name::parse(msg, &mut data).with_context(|| "type CNAME RR")?;
                Ok(Data::CNAME(name))
            }
            Type::SOA => {
                let mname = name::parse(msg, &mut data).with_context(|| "type SOA RR mname field")?;
                let rname = name::parse(msg, &mut data).with_context(|| "type SOA RR rname field")?;
                if data.remaining() < 4 {
                    anyhow::bail!("incomplete type SOA RR serial field");
                }
                let serial = data.get_u32();
                if data.remaining() < 4 {
                    anyhow::bail!("incomplete type SOA RR refresh field");
                }
                let refresh = data.get_u32();
                if data.remaining() < 4 {
                    anyhow::bail!("incomplete type SOA RR retry field");
                }
                let retry = data.get_u32();
                if data.remaining() < 4 {
                    anyhow::bail!("incomplete type SOA RR expire field");
                }
                let expire = data.get_u32();
                if data.remaining() < 4 {
                    anyhow::bail!("incomplete type SOA RR minimum field");
                }
                let minimum = data.get_i32();
                Ok(Data::SOA {
                    mname,
                    rname,
                    serial,
                    refresh,
                    retry,
                    expire,
                    minimum,
                })
            }
            Type::MB => {
                let name = name::parse(msg, &mut data).with_context(|| "type MB RR")?;
                Ok(Data::MB(name))
            }
            Type::MG => {
                let name = name::parse(msg, &mut data).with_context(|| "type MG RR")?;
                Ok(Data::MG(name))
            }
            Type::MR => {
                let name = name::parse(msg, &mut data).with_context(|| "type MR RR")?;
                Ok(Data::MR(name))
            }
            Type::NULL => {
                if data_len > 65535 {
                    anyhow::bail!("type NULL RR data too long");
                }
                Ok(Data::NULL(data.to_vec()))
            }
            Type::WKS => {
                if data.remaining() < 4 {
                    anyhow::bail!("incomplete type WKS RR address field");
                }
                let address = Ipv4Addr::new(data[0], data[1], data[2], data[3]);
                data.advance(4);

                if data.remaining() < 1 {
                    anyhow::bail!("incomplete type WKS RR protocol field");
                }
                let protocol = data.get_u8();

                let mut bit_map = Vec::new();
                while data.has_remaining() {
                    bit_map.push(data.get_u8());
                }

                Ok(Data::WKS {
                    address,
                    protocol,
                    bit_map,
                })
            }
            Type::PTR => {
                let name = name::parse(msg, &mut data).with_context(|| "type PTR RR")?;
                Ok(Data::PTR(name))
            }
            Type::HINFO => {
                Ok(Data::HINFO {
                    cpu: CharacterString::parse(data).with_context(|| "type HINFO RR cpu field")?,
                    os: CharacterString::parse(data).with_context(|| "type HINFO RR ok field")?
                })
            }
            Type::MINFO => {
                Ok(Data::MINFO {
                    rmailbx: name::parse(msg, &mut data).with_context(|| "type MINFO RR rmailbx field")?,
                    emailbx: name::parse(msg, &mut data).with_context(|| "type MINFO RR emailbx field")?,
                })
            }
            Type::MX => {
                if data.remaining() < 2 {
                    anyhow::bail!("incomplete type MX RR preference field");
                }
                let preference = data.get_i16();
                Ok(Data::MX {
                    preference,
                    exchange: name::parse(msg, &mut data).with_context(|| "type MX RR exchange field")?
                })
            }
            Type::TXT => {
                let mut txt_data = Vec::new();
                while let Ok(ch_str) = CharacterString::parse(data) {
                    txt_data.push(ch_str);
                }
                Ok(Data::TXT(txt_data))
            }
        }
    }

    pub fn serialize(&self) -> u16 {
        use Type::*;

        match self {
            A => 1,
            NS => 2,
            MD => 3,
            MF => 4,
            CNAME => 5,
            SOA => 6,
            MB => 7,
            MG => 8,
            MR => 9,
            NULL => 10,
            WKS => 11,
            PTR => 12,
            HINFO => 13,
            MINFO => 14,
            MX => 15,
            TXT => 16,
        }
    }
}

struct CharacterString;

impl CharacterString {
    fn parse(data: &[u8]) -> anyhow::Result<String> {
        let mut data = data;
        if data.remaining() == 0 {
            anyhow::bail!("incomplete character string length");
        }
        let len = data.get_u8() as usize;
        if len > 255 {
            anyhow::bail!("character string too long");
        }
        if data.remaining() < len {
            anyhow::bail!("incomplete character string");
        }
        Ok(String::from_utf8(data.to_vec())?)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::name;
    use bytes::BufMut;

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

        let mut data: &[u8] = &[0, 17];
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

        let mut data: &[u8] = &[0, 5];
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
        let rr = ResourceRecord {
            name: "google.com.".to_string(),
            r#type: Type::A,
            class: Class::IN,
            ttl: 100,
            data: Some(vec![43, 56, 121, 92]),
        };
        let buf = rr.serialize()?;

        let mut unparsed = &buf[..];
        let parsed_rr = ResourceRecord::parse(buf.as_slice(), &mut unparsed)?;
        assert_eq!(parsed_rr.name, rr.name);
        assert_eq!(parsed_rr.r#type, rr.r#type);
        assert_eq!(parsed_rr.class, rr.class);
        assert_eq!(parsed_rr.ttl, rr.ttl);
        assert_eq!(parsed_rr.data.unwrap(), rr.data.unwrap());
        assert_eq!(
            unsafe { unparsed.as_ptr().offset_from(buf.as_ptr()) as usize },
            buf.len()
        );

        Ok(())
    }

    #[test]
    fn parse_character_string() {
        // TODO: First write the serialize test.
        todo!("write this test");
    }

    #[test]
    fn serialize_type() {
        assert_eq!(Type::A.serialize(), 1);
        assert_eq!(Type::NS.serialize(), 2);
        assert_eq!(Type::MD.serialize(), 3);
        assert_eq!(Type::MF.serialize(), 4);
        assert_eq!(Type::CNAME.serialize(), 5);
        assert_eq!(Type::SOA.serialize(), 6);
        assert_eq!(Type::MB.serialize(), 7);
        assert_eq!(Type::MG.serialize(), 8);
        assert_eq!(Type::MR.serialize(), 9);
        assert_eq!(Type::NULL.serialize(), 10);
        assert_eq!(Type::WKS.serialize(), 11);
        assert_eq!(Type::PTR.serialize(), 12);
        assert_eq!(Type::HINFO.serialize(), 13);
        assert_eq!(Type::MINFO.serialize(), 14);
        assert_eq!(Type::MX.serialize(), 15);
        assert_eq!(Type::TXT.serialize(), 16);
    }

    #[test]
    fn serialize_class() {
        assert_eq!(Class::IN.serialize(), 1);
        assert_eq!(Class::CS.serialize(), 2);
        assert_eq!(Class::CH.serialize(), 3);
        assert_eq!(Class::HS.serialize(), 4);
    }

    /// ! When/if a nameserver is implemented, which ideally will use compressed names,
    /// ! this test should be updated to exercise compressed names in ResourceRecord instances.
    #[test]
    fn serialize_rr() -> anyhow::Result<()> {
        let rr = ResourceRecord {
            name: "google.com.".to_string(),
            r#type: Type::A,
            class: Class::IN,
            ttl: 100,
            data: Some(vec![43, 56, 121, 92]),
        };

        let buf = rr.serialize()?;
        let name_ser = name::serialize(&rr.name, None)?;
        assert_eq!(&buf[..name_ser.len()], name_ser);
        let mut cursor = &buf[name_ser.len()..];
        assert_eq!(cursor.get_u16(), rr.r#type.serialize());
        assert_eq!(cursor.get_u16(), rr.class.serialize());
        assert_eq!(cursor.get_u32(), rr.ttl);
        assert_eq!(cursor.get_u16() as usize, rr.data.as_ref().unwrap().len());
        assert_eq!(cursor, rr.data.unwrap());

        Ok(())
    }

    #[test]
    fn serialize_character_string() {
        todo!("write this test");
    }
}
