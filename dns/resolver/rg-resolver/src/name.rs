use anyhow::Context;
use bytes::{Buf, BufMut};
use tracing::debug;

/// ptr holds the offset within the *message* of the tail end of a compressed name.
pub fn serialize(name: &str, ptr: Option<u16>) -> anyhow::Result<Vec<u8>> {
    let mut buf = Vec::new();
    let labels = name.split('.').collect::<Vec<_>>();
    for label in labels {
        buf.put_u8(label.len() as u8);
        for b in label.chars().map(|c| c as u8) {
            buf.put_u8(b);
        }
    }
    if let Some(offset) = ptr {
        if name.ends_with('.') {
            anyhow::bail!("the root label may not precede the pointer in a compressed name");
        }
        buf.put_u16(0xc000 | offset);
    } else {
        if !name.ends_with('.') {
            anyhow::bail!("a non-compressed name must end with the root label");
        }
    }

    Ok(buf)
}

/// msg must point to the very first byte of the message,
/// not the current location in the message.
pub fn parse<'a>(msg: &'a [u8], unparsed: &mut &'a [u8]) -> anyhow::Result<String> {
    let mut name = String::new();
    let mut buf = *unparsed;
    let mut input_slice_advanced = false;
    loop {
        let len = if buf.has_remaining() {
            let mut peek: &[u8] = buf;
            peek.get_u8() as usize
        } else {
            anyhow::bail!("incomplete name")
        };
        debug!(len, "Peeked length byte");
        if len == 0 {
            // Advance past the length byte we only peeked at.
            buf.advance(1);
            // Advance the input slice when the end of the name is reached
            // only if no pointers were encountered.
            if !input_slice_advanced {
                *unparsed = buf;
            }
            if name.len() <= 255 {
                return Ok(name);
            } else {
                anyhow::bail!("name exceeds maximum length of 255");
            }
        }
        if is_compressed(len)? {
            let offset = if buf.remaining() >= 2 {
                (buf.get_u16() & !0xc000) as usize
            } else {
                anyhow::bail!("incomplete pointer")
            };
            // Advance the input slice when the first pointer is encountered.
            // Pointed-to names are located earlier in the message so
            // the input slice should not be advanced after this.
            if !input_slice_advanced {
                *unparsed = buf;
                input_slice_advanced = true;
            }
            // Continue parsing the name starting at the pointed to location earlier in the message.
            buf = &msg[offset..];
            continue;
        }
        // Advance past the length byte we only peeked at.
        buf.advance(1);
        let label = if buf.remaining() >= len {
            let label = &buf[..len];
            buf.advance(len);
            let label =
                String::from_utf8(label.to_vec()).with_context(|| "label not valid UTF-8")?;
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
        _ => Err(anyhow::anyhow!(
            "use of reserved value in compression indication bits"
        )),
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use bytes::BufMut;

    #[test]
    fn serialize_uncompressed() -> anyhow::Result<()> {
        let name = serialize("google.com.", None)?;
        let expected = [
            6, b'g', b'o', b'o', b'g', b'l', b'e', 3, b'c', b'o', b'm', 0,
        ];
        assert_eq!(name, expected);

        assert!(serialize("google.com", None).is_err());
        Ok(())
    }

    #[test]
    fn serialize_compressed() -> anyhow::Result<()> {
        let name = serialize("api", Some(7))?;
        let expected = [3, b'a', b'p', b'i', 0xc0, 7];
        assert_eq!(name, expected);

        assert!(serialize("api.", Some(7)).is_err());
        Ok(())
    }

    #[test]
    fn parse_uncompressed() -> anyhow::Result<()> {
        let mut msg = Vec::new();
        for i in 1..11 {
            msg.put_u8(i)
        }
        let name_offset = msg.len();
        let name = "google.com.";
        let mut name_ser = serialize(name, None).expect("serialize name");
        let name_ser_len = name_ser.len();
        msg.append(&mut name_ser);

        let mut unparsed = &msg[name_offset..];
        let parse_start = unparsed;
        let parsed_name = parse(&msg[..], &mut unparsed)?;
        assert_eq!(parsed_name, name);
        assert_eq!(
            unsafe { unparsed.as_ptr().offset_from(parse_start.as_ptr()) as usize },
            name_ser_len
        );

        Ok(())
    }

    #[test]
    fn parse_compressed() -> anyhow::Result<()> {
        // Test parsing a nested compressed name: "drive.api.google.com.".
        // "google.com." is stored as an uncompressed name.
        // "api.google.com." is stored as a compressed name with uncompressed base "google.com.".
        // "drive.api.google.com." is stored as a compressed name with compressed base "api.google.com.".

        let mut msg = Vec::new();

        for i in 1..11 {
            msg.put_u8(i)
        }
        let name1_offset = msg.len();
        let name1 = "google.com.";
        let mut name1_ser = serialize(name1, None).expect("serialize name1");
        msg.append(&mut name1_ser);

        for i in 11..21 {
            msg.put_u8(i)
        }
        let name2_offset = msg.len();
        let name2 = "api";
        let mut name2_ser = serialize(name2, Some(name1_offset as u16)).expect("serialize name2");
        msg.append(&mut name2_ser);

        for i in 21..31 {
            msg.put_u8(i)
        }
        let name3_offset = msg.len();
        let name3 = "drive";
        let mut name3_ser = serialize(name3, Some(name2_offset as u16)).expect("serialize name3");
        let name3_ser_len = name3_ser.len();
        msg.append(&mut name3_ser);

        let name = [name3, name2, name1].join(".");
        let mut unparsed = &msg[name3_offset..];
        let parse_start = unparsed;
        let parsed_name = parse(&msg[..], &mut unparsed)?;
        assert_eq!(parsed_name, name);
        assert_eq!(
            unsafe { unparsed.as_ptr().offset_from(parse_start.as_ptr()) as usize },
            name3_ser_len
        );

        Ok(())
    }
}
