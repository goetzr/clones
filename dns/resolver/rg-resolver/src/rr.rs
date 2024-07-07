use bytes::{Buf, BufMut};
use anyhow::Context;
use tracing::debug;

pub struct ResourceRecord {
    name: String,
    r#type: Type,
    class: Class,
    ttl: u32,
    data_len: u16,
    data: Option<Vec<u8>>,
}

impl ResourceRecord {
    pub fn parse(data: &[u8]) {
        let mut parser = ResourceRecordParser::new(data);
        let name = parser.parse_name();
    }
}
struct ResourceRecordParser<'a> {
    data: &'a [u8],
    unparsed: &'a [u8],
}

impl<'a> ResourceRecordParser<'a> {
    fn new(data: &'a [u8]) -> Self {
        Self { data, unparsed: data }
    }

    fn parse_name(&mut self) -> anyhow::Result<String> {
        let mut name_parser = ResourceRecordNameParser::new(self.data);
        name_parser.parse()
    }
}

struct ResourceRecordNameParser<'a> {
    data: &'a [u8],
    unparsed: &'a [u8],
}

impl<'a> ResourceRecordNameParser<'a> {
    fn new(data: &'a [u8]) -> Self {
        Self { data, unparsed: data }
    }

    fn parse(&mut self) -> anyhow::Result<String> {
        debug!("Called parse");
        let mut name = String::new();
        loop {
            let len = if self.unparsed.has_remaining() {
                let mut peek = self.unparsed;
                peek.get_u8() as usize
            } else {
                anyhow::bail!("incomplete name")
            };
            debug!(len, "Peeked length byte");
            if len == 0 {
                // Advance past the length byte we only peeked at.
                let _ = self.unparsed.get_u8();
                if name.len() <= 255 {
                    return Ok(name);
                } else {
                    anyhow::bail!("name exceeds maximum length of 255");
                }
            }
            if Self::is_compressed(len)? {
                let offset = if self.unparsed.remaining() >= 2 {
                    (self.unparsed.get_u16() & !0xc000) as usize
                } else {
                    anyhow::bail!("incomplete pointer")
                };
                self.unparsed = &self.data[offset..];
                // Continue parsing the name starting at the pointed to location.
                continue;
            }
            // Advance past the length byte we only peeked at.
            let _ = self.unparsed.get_u8();
            let label = if self.unparsed.remaining() >= len {
                let label = &self.unparsed[..len];
                self.unparsed.advance(len);
                let label = String::from_utf8(label.to_vec())
                    .with_context(|| "label not valid UTF-8")?;
                if label.is_ascii() {
                    label
                } else {
                    anyhow::bail!("label not ASCII");
                }
            } else {
                anyhow::bail!("incomplete label")
            };
            name.push_str(&label);
            name.push('.');
        }
    }

    fn is_compressed(len: usize) -> anyhow::Result<bool> {
        match len & 0xc0 {
            0xc0 => Ok(true),
            0x00 => Ok(false),
            _  => Err(anyhow::anyhow!("use of reserved value in compression indication bits"))
        }
    }
}

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

    fn serialize_name(name: &str) -> Vec<u8> {
        let mut buf = Vec::new();
        let labels = name.split('.').collect::<Vec<_>>();
        for label in labels {
            buf.put_u8(label.len() as u8);
            for b in label.chars().map(|c| c as u8) {
                buf.put_u8(b);
            }
        }
        buf
    }

    #[test]
    fn serialize_name_helper() {
        let serialized = serialize_name("google");
        let expected = [6, b'g', b'o', b'o', b'g', b'l', b'e'];
        assert_eq!(serialized, expected);

        let serialized = serialize_name("google.com");
        let expected = [6, b'g', b'o', b'o', b'g', b'l', b'e', 3, b'c', b'o', b'm'];
        assert_eq!(serialized.as_slice(), &expected);

        let serialized = serialize_name("mail.google.com.");
        let expected = [4, b'm', b'a', b'i', b'l', 6, b'g', b'o', b'o', b'g', b'l', b'e', 3, b'c', b'o', b'm', 0];
        assert_eq!(serialized.as_slice(), &expected);
    }

    fn dump(data: &[u8]) {
        // TODO: Dump hexdump of data to stdout
    }

    fn serialize_rr(name: &str, ptr: Option<u16>, r#type: u16, class: u16, ttl: u32, data: Vec<u8>) -> Vec<u8> {
        let mut buf = Vec::new();

        let mut ser_name = serialize_name(name);
        buf.append(&mut ser_name);
        if let Some(offset) = ptr {
            assert!(!name.ends_with('.'));
            buf.put_u16(0xc000 | offset);
        } else {
            assert!(name.ends_with('.'));
        }

        buf.put_u16(r#type);
        buf.put_u16(class);
        buf.put_u32(ttl);
        buf.put_u16(buf.len() as u16);
        let mut data = data;
        buf.append(&mut data);

        buf
    }

    fn build_rrs() -> (Vec<u8>, Vec<u8>) {
        // Compressed name as data ("ns.google.com").
        let mut data1 = Vec::new();
        let mut data1_name = serialize_name("ns");
        data1.append(&mut data1_name);
        data1.put_u16(0xc000 | 0_u16); // 0 is the offset of "google.com." in the name of this RR
        // Uncompressed name as name.
        let rr1 = serialize_rr("google.com.", None, 1 /* NS */, 0 /* IN */, 10, data1);

        // Compressed name as data ("api.ns.google.com").
        let mut data2 = Vec::new();
        let mut data2_name = serialize_name("api");
        data2.append(&mut data2_name);
        data2.put_u16(0xc000 | 22_u16); // 22 is the offset of "ns.google.com." in the data of the previous RR
        // Compressed name as name ("api.google.com").
        // 0 is the offset of "google.com." in the name of the previous RR
        let rr2 = serialize_rr("api", Some(0), 1 /* NS */, 0 /* IN */, 12, data2);

        (rr1, rr2)
    }

    #[test]
    fn serialize_rr_helper() {
        let (rr1, rr2) = build_rrs();

        let expected = [
            6, b'g', b'o', b'o', b'g', b'l', b'e', 3, b'c', b'o', b'm', 0, // name
            0, 1,   // type
            0, 0,   // class
            0, 0, 0, 10,  // ttl
            0, 5,     // data length
            2, b'n', b's', 0xc0, 0  // data
        ];
        assert_eq!(rr1, expected, "first RR");

        let expected = [
            3, b'a', b'p', b'i', 0xc0, 0, // name
            0, 1,   // type
            0, 0,   // class
            0, 0, 0, 12,  // ttl
            0, 6,     // data length
            3, b'a', b'p', b'i', 0xc0, 22   // data
        ];
        assert_eq!(rr2, expected, "second RR");

    }

    #[traced_test]
    #[test]
    fn parse_name() -> anyhow::Result<()> {
        let mut data = Vec::new();
        let (mut rr1, mut rr2) = build_rrs();
        data.append(&mut rr1);
        data.append(&mut rr2);

        // Parse "google.com." from name of first RR.
        let mut parser = ResourceRecordNameParser::new(data.as_slice());
        let name1 = parser.parse()?;
        assert_eq!(name1, "google.com.");

        // Parse "ns.google.com." from data of first RR.
        let mut parser = ResourceRecordNameParser::new(&data[22..]);
        let name1 = parser.parse()?;
        assert_eq!(name1, "google.com.");

        // Parse "api.google.com." from name of second RR.
        let mut parser = ResourceRecordNameParser::new(&data[27..]);
        let name1 = parser.parse()?;
        assert_eq!(name1, "google.com.");

        // Parse "api.ns.google.com" from data of second RR.
        let mut parser = ResourceRecordNameParser::new(&data[43..]);
        let name1 = parser.parse()?;
        assert_eq!(name1, "google.com.");


        Ok(())
    }
}