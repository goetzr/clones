use crate::name;
use bytes::{Buf, BufMut};
use std::net::Ipv4Addr;
use anyhow::Context;

#[derive(Debug, Clone, PartialEq)]
pub struct ResourceRecord {
    name: String,
    r#type: Type,
    class: Class,
    ttl: i32,
    data: Data,
}

impl ResourceRecord {
    fn new(name: String, r#type: Type, class: Class, ttl: i32, data: Data) -> anyhow::Result<Self> {
        let types_match = match r#type {
            Type::A => matches!(data, Data::A(_)),
            Type::NS => matches!(data, Data::NS(_)),
            Type::MD => matches!(data, Data::MD(_)),
            Type::MF => matches!(data, Data::MF(_)),
            Type::CNAME => matches!(data, Data::CNAME(_)),
            Type::SOA => matches!(data, Data::SOA { .. }),
            Type::MB => matches!(data, Data::MB(_)),
            Type::MG => matches!(data, Data::MG(_)),
            Type::MR => matches!(data, Data::MR(_)),
            Type::NULL => matches!(data, Data::NULL(_)),
            Type::WKS => matches!(data, Data::WKS { .. }),
            Type::PTR => matches!(data, Data::PTR(_)),
            Type::HINFO => matches!(data, Data::HINFO { .. }),
            Type::MINFO => matches!(data, Data::MINFO { .. }),
            Type::MX => matches!(data, Data::MX { .. }),
            Type::TXT => matches!(data, Data::TXT(_)),
        };
        if !types_match {
            anyhow::bail!("RR type doesn't match its data type");
        }

        let rr = ResourceRecord {
            name, r#type, class, ttl, data
        };
        Ok(rr)
    }

    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    pub fn r#type(&self) -> Type {
        self.r#type
    }

    pub fn class(&self) -> Class {
        self.class
    }

    pub fn ttl(&self) -> i32 {
        self.ttl
    }

    pub fn data(&self) -> &Data {
        &self.data
    }
    
    /// * msg must point to the very first byte of the message.
    pub fn parse<'a>(msg: &'a [u8], unparsed: &mut &'a [u8]) -> anyhow::Result<ResourceRecord> {
        let name = name::parse(msg, unparsed)?;
        let r#type = Type::parse(unparsed)?;
        let class = Class::parse(unparsed)?;
        let ttl = Self::parse_ttl(unparsed)?;
        let data = Data::parse(msg, unparsed, r#type)?;

        let rr = ResourceRecord {
            name,
            r#type,
            class,
            ttl,
            data,
        };
        Ok(rr)
    }

    fn parse_ttl(unparsed: &mut &[u8]) -> anyhow::Result<i32> {
        if unparsed.remaining() < 4 {
            anyhow::bail!("incomplete RR TTL");
        }
        Ok(unparsed.get_i32())
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
        // * A nameserver storing multiple RRs in a message must truncate messages
        // * larger than 512 bytes.
        let mut buf = Vec::new();
        buf.append(&mut name::serialize(&self.name, None)?);
        buf.put_u16(self.r#type.serialize());
        buf.put_u16(self.class.serialize());
        buf.put_i32(self.ttl);
        let mut data = self.data.serialize()?;
        buf.put_u16(data.len() as u16);
        buf.append(&mut data);
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
            anyhow::bail!("incomplete RR type");
        }
        use Type::*;
        match unparsed.get_u16() {
            1 => Ok(A),
            2 => Ok(NS),
            3 => Ok(MD),
            4 => Ok(MF),
            5 => Ok(CNAME),
            6 => Ok(SOA),
            7 => Ok(MB),
            8 => Ok(MG),
            9 => Ok(MR),
            10 => Ok(NULL),
            11 => Ok(WKS),
            12 => Ok(PTR),
            13 => Ok(HINFO),
            14 => Ok(MINFO),
            15 => Ok(MX),
            16 => Ok(TXT),
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
            anyhow::bail!("incomplete RR class");
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

#[derive(Clone, Debug, PartialEq)]
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
            anyhow::bail!("incomplete RR data length");
        }
        let data_len = unparsed.get_u16() as usize;
        if unparsed.remaining() < data_len {
            anyhow::bail!("incomplete RR data");
        }
        let mut data = &unparsed[..data_len];
        unparsed.advance(data_len);

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
                let bit_map = data[..].to_vec();

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

    pub fn serialize(&self) -> anyhow::Result<Vec<u8>> {
        let mut data = Vec::new();
        use Data::*;
        match self {
            A(address) => data.append(&mut address.octets().to_vec()),
            NS(nsdname) => data.append(&mut name::serialize(nsdname, None)?),
            MD(madname) => data.append(&mut name::serialize(madname, None)?),
            MF(madname) => data.append(&mut name::serialize(madname, None)?),
            CNAME(cname) => data.append(&mut name::serialize(cname, None)?),
            SOA {
                mname,
                rname,
                serial,
                refresh,
                retry,
                expire,
                minimum,
            } => {
                data.append(&mut name::serialize(mname, None)?);
                data.append(&mut name::serialize(rname, None)?);
                data.put_u32(*serial);
                data.put_u32(*refresh);
                data.put_u32(*retry);
                data.put_u32(*expire);
                data.put_i32(*minimum);
            }
            MB(madname) => data.append(&mut name::serialize(madname, None)?),
            MG(mgmname) => data.append(&mut name::serialize(mgmname, None)?),
            MR(newname) => data.append(&mut name::serialize(newname, None)?),
            NULL(any) => data.append(&mut any.clone()),
            WKS {
                address,
                protocol,
                bit_map,
            } => {
                data.append(&mut address.octets().to_vec());
                data.put_u8(*protocol);
                data.append(&mut bit_map.clone());
            }
            PTR(ptrdname) => data.append(&mut name::serialize(ptrdname, None)?),
            HINFO {
                cpu,
                os,
            } => {
                data.append(&mut CharacterString::serialize(cpu)?);
                data.append(&mut CharacterString::serialize(os)?);
            }
            MINFO {
                rmailbx,
                emailbx,
            } => {
                data.append(&mut name::serialize(rmailbx, None)?);
                data.append(&mut name::serialize(emailbx, None)?);
            }
            MX {
                preference,
                exchange,
            } => {
                data.put_i16(*preference);
                data.append(&mut name::serialize(exchange, None)?);
            }
            TXT(txt_data) => {
                for txt in txt_data {
                    data.append(&mut CharacterString::serialize(txt)?);
                }
            }
        };
        Ok(data)
    }
}

struct CharacterString;

impl CharacterString {
    const MAX_CHARS: usize = 255;

    fn parse(data: &[u8]) -> anyhow::Result<String> {
        let mut data = data;
        if data.remaining() == 0 {
            anyhow::bail!("incomplete character string length");
        }
        let len = data.get_u8() as usize;
        if len > CharacterString::MAX_CHARS {
            anyhow::bail!("character string too long");
        }
        if data.remaining() < len {
            anyhow::bail!("incomplete character string");
        }
        Ok(String::from_utf8(data.to_vec())?)
    }

    fn serialize(name: &str) -> anyhow::Result<Vec<u8>> {
        if name.len() > CharacterString::MAX_CHARS {
            anyhow::bail!("string to long to be a character string");
        }
        let mut data = Vec::new();
        data.put_u8(name.len() as u8);
        data.append(&mut name.as_bytes().to_vec());
        Ok(data)
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
        todo!("write this test");

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

    // A(address) => data.append(&mut address.octets().to_vec()),
    #[test]
    fn serialize_data_a() -> anyhow::Result<()> {
        let octets = [160, 23, 58, 191];
        let data = Data::A(Ipv4Addr::new(octets[0], octets[1], octets[2], octets[3]));
        assert_eq!(data.serialize()?, octets);
        Ok(())
    }
    
    // NS(nsdname) => data.append(&mut name::serialize(nsdname, None)?),
    #[test]
    fn serialize_data_ns() -> anyhow::Result<()> {
        let nsdname = "google.com.";
        let data = Data::NS(nsdname.to_string());
        let expected = name::serialize(nsdname, None)?;
        assert_eq!(data.serialize()?, expected);
        Ok(())
    }

    // MD(madname) => data.append(&mut name::serialize(madname, None)?),
    #[test]
    fn serialize_data_md() -> anyhow::Result<()> {
        let madname = "google.com.";
        let data = Data::MD(madname.to_string());
        let expected = name::serialize(madname, None)?;
        assert_eq!(data.serialize()?, expected);
        Ok(())
    }

    // MF(madname) => data.append(&mut name::serialize(madname, None)?),
    #[test]
    fn serialize_data_mf() -> anyhow::Result<()> {
        let madname = "google.com.";
        let data = Data::MF(madname.to_string());
        let expected = name::serialize(madname, None)?;
        assert_eq!(data.serialize()?, expected);
        Ok(())
    }

    // CNAME(cname) => data.append(&mut name::serialize(cname, None)?),
    #[test]
    fn serialize_data_cname() -> anyhow::Result<()> {
        let cname = "google.com.";
        let data = Data::CNAME(cname.to_string());
        let expected = name::serialize(cname, None)?;
        assert_eq!(data.serialize()?, expected);
        Ok(())
    }

    // SOA {
    //     mname,
    //     rname,
    //     serial,
    //     refresh,
    //     retry,
    //     expire,
    //     minimum,
    // } => {
    //     data.append(&mut name::serialize(mname, None)?);
    //     data.append(&mut name::serialize(rname, None)?);
    //     data.put_u32(*serial);
    //     data.put_u32(*refresh);
    //     data.put_u32(*retry);
    //     data.put_u32(*expire);
    //     data.put_i32(*minimum);
    // }
    #[test]
    fn serialize_data_soa() -> anyhow::Result<()> {
        let mname = "google.com.";
        let rname = "amazon.com.";
        let serial = 25;
        let refresh = 10;
        let retry = 12;
        let expire = 24;
        let minimum = 30;
        let soa = Data::SOA {
            mname: mname.to_string(),
            rname: rname.to_string(),
            serial,
            refresh,
            retry,
            expire,
            minimum,
        };
        let mut expected = Vec::new();
        expected.append(&mut name::serialize(mname, None)?);
        expected.append(&mut name::serialize(rname, None)?);
        expected.put_u32(serial);
        expected.put_u32(refresh);
        expected.put_u32(retry);
        expected.put_u32(expire);
        expected.put_i32(minimum);
        assert_eq!(soa.serialize()?, expected);
        Ok(())
    }

    // MB(madname) => data.append(&mut name::serialize(madname, None)?),
    #[test]
    fn serialize_data_mb() -> anyhow::Result<()> {
        let madname = "google.com.";
        let data = Data::MB(madname.to_string());
        let expected = name::serialize(madname, None)?;
        assert_eq!(data.serialize()?, expected);
        Ok(())
    }

    // MG(mgmname) => data.append(&mut name::serialize(mgmname, None)?),
    #[test]
    fn serialize_data_mg() -> anyhow::Result<()> {
        let mgmname = "google.com.";
        let data = Data::MG(mgmname.to_string());
        let expected = name::serialize(mgmname, None)?;
        assert_eq!(data.serialize()?, expected);
        Ok(())
    }

    // MR(newname) => data.append(&mut name::serialize(newname, None)?),
    #[test]
    fn serialize_data_mr() -> anyhow::Result<()> {
        let newname = "google.com.";
        let data = Data::MR(newname.to_string());
        let expected = name::serialize(newname, None)?;
        assert_eq!(data.serialize()?, expected);
        Ok(())
    }

    // NULL(any) => data.append(&mut any.clone()),
    #[test]
    fn serialize_data_null() -> anyhow::Result<()> {
        let any = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        let data = Data::NULL(any.clone());
        assert_eq!(data.serialize()?, any);
        Ok(())
    }

    // WKS {
    //     address,
    //     protocol,
    //     bit_map,
    // } => {
    //     data.append(&mut address.octets().to_vec());
    //     data.put_u8(*protocol);
    //     data.append(&mut bit_map.clone());
    // }
    #[test]
    fn serialize_data_wks() -> anyhow::Result<()> {
        let octets = [123, 45, 98, 112];
        let address = Ipv4Addr::new(octets[0], octets[1], octets[2], octets[3]);
        let protocol = 6;
        let bit_map = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        let data = Data::WKS {
            address,
            protocol,
            bit_map: bit_map.clone(),
        };
        let mut expected = Vec::new();
        expected.append(&mut octets.to_vec());
        expected.put_u8(protocol);
        expected.append(&mut bit_map.clone());
        assert_eq!(data.serialize()?, expected);
        Ok(())
    }

    // PTR(ptrdname) => data.append(&mut name::serialize(ptrdname, None)?),
    #[test]
    fn serialize_data_ptr() -> anyhow::Result<()> {
        let ptrdname = "google.com.";
        let data = Data::PTR(ptrdname.to_string());
        let expected = name::serialize(ptrdname, None)?;
        assert_eq!(data.serialize()?, expected);
        Ok(())
    }

    // HINFO {
    //     cpu,
    //     os,
    // } => {
    //     data.append(&mut CharacterString::serialize(cpu)?);
    //     data.append(&mut CharacterString::serialize(os)?);
    // }
    #[test]
    fn serialize_data_hinfo() -> anyhow::Result<()> {
        let cpu = "x64";
        let os = "Ubuntu";
        let data = Data::HINFO {
            cpu: cpu.to_string(),
            os: os.to_string(),
        };
        let mut expected = Vec::new();
        expected.append(&mut CharacterString::serialize(cpu)?);
        expected.append(&mut CharacterString::serialize(os)?);
        assert_eq!(data.serialize()?, expected);
        Ok(())
    }

    // MINFO {
    //     rmailbx,
    //     emailbx,
    // } => {
    //     data.append(&mut name::serialize(rmailbx, None)?);
    //     data.append(&mut name::serialize(emailbx, None)?);
    // }
    #[test]
    fn serialize_data_minfo() -> anyhow::Result<()> {
        let rmailbx = "google.com.";
        let emailbx = "amazon.com.";
        let data = Data::MINFO {
            rmailbx: rmailbx.to_string(),
            emailbx: emailbx.to_string(),
        };
        let mut expected = Vec::new();
        expected.append(&mut name::serialize(rmailbx, None)?);
        expected.append(&mut name::serialize(emailbx, None)?);
        assert_eq!(data.serialize()?, expected);
        Ok(())
    }

    // MX {
    //     preference,
    //     exchange,
    // } => {
    //     data.put_i16(*preference);
    //     data.append(&mut name::serialize(exchange, None)?);
    // }
    #[test]
    fn serialize_data_mx() -> anyhow::Result<()> {
        let preference = 12;
        let exchange = "google.com.";
        let data = Data::MX {
            preference,
            exchange: exchange.to_string(),
        };
        let mut expected = Vec::new();
        expected.put_i16(preference);
        expected.append(&mut name::serialize(exchange, None)?);
        assert_eq!(data.serialize()?, expected);
        Ok(())
    }

    // TXT(txt_data) => {
    //     for txt in txt_data {
    //         data.append(&mut CharacterString::serialize(txt)?);
    //     }
    // }
    #[test]
    fn serialize_data_txt() -> anyhow::Result<()> {
        let mut txt_data = Vec::new();
        let txt1 = "content 1";
        let txt2 = "content 2";
        let txt3 = "content 3";
        txt_data.push(txt1.to_string());
        txt_data.push(txt2.to_string());
        txt_data.push(txt3.to_string());
        let data = Data::TXT(txt_data);
        let mut expected = Vec::new();
        expected.append(&mut CharacterString::serialize(txt1)?);
        expected.append(&mut CharacterString::serialize(txt2)?);
        expected.append(&mut CharacterString::serialize(txt3)?);
        assert_eq!(data.serialize()?, expected);
        Ok(())
    }
    
    /// ! When/if a nameserver is implemented, which ideally will use compressed names,
    /// ! this test should be updated to exercise compressed names in ResourceRecord instances.
    #[test]
    fn serialize_rr() -> anyhow::Result<()> {
        let rr = ResourceRecord::new(
            "google.com.".to_string(),
            Type::A,
            Class::IN,
            100,
            Data::A(Ipv4Addr::new(43, 56, 121, 92)),
        )?;

        let mut expected = Vec::new();
        expected.append(&mut name::serialize(&rr.name, None)?);
        expected.put_u16(rr.r#type.serialize());
        expected.put_u16(rr.class.serialize());
        expected.put_i32(rr.ttl);
        let mut data_ser = rr.data.serialize()?;
        expected.put_u16(data_ser.len() as u16);
        expected.append(&mut data_ser);

        assert_eq!(rr.serialize()?, expected);
        Ok(())
    }

    #[test]
    fn serialize_character_string() -> anyhow::Result<()> {
        let teststr = "testing 1 2 3";
        let mut expected = Vec::new();
        expected.put_u8(teststr.len() as u8);
        expected.append(&mut teststr.as_bytes().to_vec());
        assert_eq!(CharacterString::serialize(teststr)?, expected);
        Ok(())
    }
}
