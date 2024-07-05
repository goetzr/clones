use bytes::Buf;
use anyhow::Context;

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
        let mut name = String::new();
        loop {
            let len = if self.unparsed.has_remaining() {
                let mut peek = self.unparsed;
                peek.get_u8() as usize
            } else {
                anyhow::bail!("malformed name")
            };
            if len == 0 {
                name.push('.');
                return Ok(name);
            }
            if Self::is_compressed(len)? {
                let offset = if self.unparsed.remaining() >= 2 {
                    (self.unparsed.get_u16() & !0xc000) as usize
                } else {
                    anyhow::bail!("incomplete pointer")
                };
                self.unparsed = &self.data[offset..];
                continue;
            }
            let label = if self.unparsed.remaining() >= len {
                // TODO: Need to make sure it's all ASCII too
                String::from_utf8(self.unparsed[..len].to_vec()).map(|e| e.with_context(|| ")
            }
            if self.unparsed.remaining() < len {
                anyhow::bail!("incomplete name");
            }

        }
        
        // Total length of name (label octets + label length octets) must be <= 255
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