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

#[derive(Debug)]
struct ResourceRecordNameParser<'a> {
    data: &'a [u8],
    unparsed: &'a [u8],
}

impl<'a> ResourceRecordNameParser<'a> {
    fn new(data: &'a [u8]) -> Self {
        Self { data, unparsed: data }
    }

    //#[instrument(level = "debug")]
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
            debug!(len, "Parsed length byte");
            use std::io::Write;
            let mut stdout = std::io::stdout();
            stdout.flush();
            if len == 0 {
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
            let label = if self.unparsed.remaining() >= len {
                let label = String::from_utf8(self.unparsed[..len].to_vec())
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
        let serialized = serialize_name("mail.google.com.");
        let expected: [u8; 17] = [4, b'm', b'a', b'i', b'l', 6, b'g', b'o', b'o', b'g', b'l', b'e', 3, b'c', b'o', b'm', 0];
        assert_eq!(serialized.as_slice(), &expected);
    }

    #[traced_test]
    #[test]
    fn parse_name() -> anyhow::Result<()> {
        //tracing_subscriber::fmt::init();
        debug!("Running test");
        let data = serialize_name("mail.google.com.");
        let mut parser = ResourceRecordNameParser::new(data.as_slice());
        let name = parser.parse()?;
        assert_eq!(name, "mail.google.com.");
        Ok(())
    }

    #[test]
    fn parse_compressed_name() -> anyhow::Result<()> {
        Ok(())
    }
}