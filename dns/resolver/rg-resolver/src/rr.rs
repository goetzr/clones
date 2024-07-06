use bytes::{Buf, BufMut};
use anyhow::Context;
use tracing::{instrument, debug};

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

macro_rules! buf_get_u8 {
    ($buf:expr, $err_msg:expr) => {
        if $buf.has_remaining() {
            $buf.get_u8()
        } else {
            anyhow::bail!($err_msg)
        }
    };
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

    fn serialize_rr(name: &str, ptr: Option<u16>, r#type: u16, class: u16, ttl: u32, data: Vec<u8>) -> Vec<u8> {
        // TODO: Need to consider compressed name both in name and in data.
        // TODO: Bundle name into type (&str, Option<u16>)?
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

    #[test]
    fn serialize_rr_helper() {
        let data1 = serialize_name("ns1.google.com.");
        let serialized1 = serialize_rr("google.com.", None, 1 /* NS */, 0 /* IN */, 10, data1);
        let expected = [
            6, b'g', b'o', b'o', b'g', b'l', b'e', 3, b'c', b'o', b'm', 0, // name
            0, 1,   // type
            0, 0,   // class
            0, 0, 0, 10,  // ttl
            16,     // data length
            3, b'n', b's', b'1', 6, b'g', b'o', b'o', b'g', b'l', b'e', 3, b'c', b'o', b'm', 0   // data
        ];
        assert_eq!(serialized1, expected);

        let data2 = serialize_name("ns2.google.com");
        let serialized2 = serialize_rr("api.google.com.", Some(0), 1 /* NS */, 0 /* IN */, 12, data2);
        let expected = [
            3, b'a', b'p', b'i', 6, b'g', b'o', b'o', b'g', b'l', b'e', 3, b'c', b'o', b'm', 0, // name
            0, 1,   // type
            0, 0,   // class
            0, 0, 0, 12,  // ttl
            6,     // data length
            3, b'n', b's', b'2', 0xc0, 0   // data
        ];
        assert_eq!(serialized2, expected);

    }

    #[traced_test]
    #[test]
    fn parse_name() -> anyhow::Result<()> {
        let data = serialize_name("mail.google.com.");
        let mut parser = ResourceRecordNameParser::new(data.as_slice());
        let name = parser.parse()?;
        assert_eq!(name, "mail.google.com.");
        Ok(())
    }

    #[test]
    fn parse_compressed_name() -> anyhow::Result<()> {
        let mut data = Vec::new();

        let ofs_name1 = 0_u16;
        let mut rr1 = Vec::new();
        let mut name1 = serialize_name("google.com.");
        rr1.append(&mut name1);
        for _ in 0..10 {
            rr1.put_u8(1);
        }


        // First subdomain name encoded with pointer.
        let mut sub1 = serialize_label("api"); // api.google.com
        // Record the offset of the first subdomain as it is used as the base of the second subdomain.
        let offset_sub1 = data.len() as u16;
        data.append(&mut sub1);
        // Store pointer from first subdomain to its base at offset 0.
        let ptr1 = 0xc000_u16 | 0;
        data.put_u16(ptr1);
        // Second subdomain name encoded with pointer.
        let mut sub2 = serialize_label("time"); // time.api.google.com
        data.append(&mut sub2);
        // Store pointer from second subdomain to its base (first subdomain).
        let ptr2 = 0xc000_u16 | offset_sub1;
        data.put_u16(ptr2);

        let mut parser = ResourceRecordNameParser::new(data.as_slice());
        let name1 = parser.parse()

        Ok(())
    }
}