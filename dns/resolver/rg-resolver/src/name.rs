use anyhow::Context;
use bytes::{Buf, BufMut};

/// ptr holds the offset within the *message* of the tail end of a compressed name.
// TODO: To make this safer and ensure that the pointer offset is before the current
// TODO: offset into the message, create a Pointer structure and make the ptr
// TODO: parameter have type Option<Pointer>.
pub fn serialize(name: &str, ptr: Option<u16>) -> anyhow::Result<Vec<u8>> {
    if !name.is_ascii() {
        anyhow::bail!("serializing name: name not ASCII");
    }
    let mut buf = Vec::new();
    let labels = name.split('.').map(str::trim).collect::<Vec<_>>();
    for label in labels {
        buf.put_u8(label.len() as u8);
        label.chars().map(|c| c as u8).for_each(|b| buf.put_u8(b));
    }
    if let Some(offset) = ptr {
        if offset > 2_u16.pow(14) - 1 {
            anyhow::bail!("serializing name: offset too large");
        }
        if name.ends_with('.') {
            anyhow::bail!(
                "serializing name: the root label may not precede the pointer in a compressed name"
            );
        }
        buf.put_u16(0xc000 | offset);
    } else {
        if !name.ends_with('.') {
            anyhow::bail!("serializing name: a non-compressed name must end with the root label");
        }
        // * The call to split above results in an empty string when the name ends with a '.',
        // * causing a length byte of 0 to be added to the buffer for the NULL label as desired.
    }

    Ok(buf)
}

/// msg must point to the very first byte of the message.
pub fn parse<'a>(msg: &'a [u8], unparsed: &mut &'a [u8]) -> anyhow::Result<String> {
    let mut name = String::new();
    let mut buf = *unparsed;
    let mut input_slice_advanced = false;
    loop {
        if !buf.has_remaining() {
            anyhow::bail!("parsing name: incomplete name");
        }
        let len = {
            let mut peek: &[u8] = buf;
            peek.get_u8() as usize
        };
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
                anyhow::bail!("parsing name: name exceeds maximum length of 255");
            }
        }
        if is_compressed(len)? {
            if buf.remaining() < 2 {
                anyhow::bail!("parsing name: incomplete pointer");
            }
            let ptr_offset = unsafe { buf.as_ptr().offset_from(msg.as_ptr()) as usize };
            let offset = (buf.get_u16() & !0xc000) as usize;
            if offset >= ptr_offset {
                anyhow::bail!(
                    "parsing name: pointer must point to a name that exists earlier in the message"
                );
            }
            // Advance the input slice when the first pointer is encountered.
            // Pointed-to names are located earlier in the message so
            // the input slice should not be advanced after this.
            if !input_slice_advanced {
                *unparsed = buf;
                input_slice_advanced = true;
            }
            // Continue parsing the name starting at the pointed to location in the message.
            buf = &msg[offset..];
            continue;
        }
        // Advance past the length byte we only peeked at.
        buf.advance(1);
        if buf.remaining() < len {
            anyhow::bail!("parsing name: incomplete label")
        }
        let label = &buf[..len];
        buf.advance(len);
        let label = String::from_utf8(label.to_vec())
            .with_context(|| "parsing name: label not valid UTF-8")?;
        if !label.is_ascii() {
            anyhow::bail!("parsing name: label not ASCII");
        }
        name.push_str(&label);
        name.push('.');
    }
}

fn is_compressed(len: usize) -> anyhow::Result<bool> {
    match len & 0xc0 {
        0xc0 => Ok(true),
        0x00 => Ok(false),
        _ => Err(anyhow::anyhow!(
            "parsing name: use of reserved value in compression indication bits"
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
    fn serialize_non_ascii_name() {
        // Name is unicode "Ф.".
        let cp = 0x424;
        let b1 = 0xc0_u8 | ((cp >> 6) & 0x1f) as u8;
        let b2 = 0x80_u8 | (cp & 0x3f) as u8;
        let name = vec![b1, b2];
        let mut name = String::from_utf8(name).expect("mistake in utf-8 encoding for test");
        name.push('.');
        assert!(serialize(&name, None).is_err());
    }

    #[test]
    fn serialize_compressed_offset_too_long() {
        assert!(serialize("api", Some(2_u16.pow(14))).is_err());
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

    #[test]
    fn parse_incomplete_name() {
        let mut buf = Vec::new();
        let name1 = "name1";
        buf.put_u8(name1.len() as u8);
        buf.append(&mut name1.as_bytes().to_vec());
        // Does not end in 0 byte for NULL label.
        let mut unparsed = &buf[..];
        assert!(parse(&buf[..], &mut unparsed).is_err());
    }

    #[test]
    fn parse_use_reserved_pointer_bits() {
        let mut buf = Vec::new();

        // First name is "name1".
        let name1 = "name1";
        buf.put_u8(name1.len() as u8);
        buf.append(&mut name1.as_bytes().to_vec());
        // 0 byte for NULL label.
        buf.put_u8(0);

        // Second name is "name2.name1".
        let name2_ofs = buf.len();
        let name2_label: &str = "name2";
        buf.put_u8(name2_label.len() as u8);
        buf.append(&mut name2_label.as_bytes().to_vec());
        // Pointer offset starts with bits 10, which is a reserved pattern.
        // Points to name1 at offset 0.
        buf.put_u16(0x8000);

        let mut unparsed = &buf[name2_ofs..];
        assert!(parse(&buf[..], &mut unparsed).is_err());
    }

    #[test]
    fn parse_incomplete_pointer() {
        let mut buf = Vec::new();

        // First name is "name1".
        let name1 = "name1";
        buf.put_u8(name1.len() as u8);
        buf.append(&mut name1.as_bytes().to_vec());
        // 0 byte for NULL label.
        buf.put_u8(0);

        // Second name is "name2.name1".
        let name2_ofs = buf.len();
        let name2_label: &str = "name2";
        buf.put_u8(name2_label.len() as u8);
        buf.append(&mut name2_label.as_bytes().to_vec());
        // Pointer offset needs 2 bytes, but only 1 provided.
        buf.put_u8(0xc0);

        let mut unparsed = &buf[name2_ofs..];
        assert!(parse(&buf[..], &mut unparsed).is_err());
    }

    #[test]
    fn parse_pointer_to_later_in_msg() {
        let mut buf = Vec::new();

        // First name is "name1".
        let name1 = "name1";
        buf.put_u8(name1.len() as u8);
        buf.append(&mut name1.as_bytes().to_vec());
        // 0 byte for NULL label.
        buf.put_u8(0);

        // Second name is "name2.name3".
        let name2_ofs = buf.len();
        let name2_label: &str = "name2";
        buf.put_u8(name2_label.len() as u8);
        buf.append(&mut name2_label.as_bytes().to_vec());
        // Point to name3, which is *later* in the message.
        buf.put_u16(0xc000 + buf.len() as u16 + 2);

        // Third name is "name3".
        let name3 = "name3";
        buf.put_u8(name3.len() as u8);
        buf.append(&mut name3.as_bytes().to_vec());
        // 0 byte for NULL label.
        buf.put_u8(0);

        let mut unparsed = &buf[name2_ofs..];
        assert!(parse(&buf[..], &mut unparsed).is_err());
    }

    #[test]
    fn parse_pointer_outside_msg() {
        let mut buf = Vec::new();

        // First name is "name1".
        let name1 = "name1";
        buf.put_u8(name1.len() as u8);
        buf.append(&mut name1.as_bytes().to_vec());
        // 0 byte for NULL label.
        buf.put_u8(0);

        // Second name is "name2.name1".
        let name2_ofs = buf.len();
        let name2_label: &str = "name2";
        buf.put_u8(name2_label.len() as u8);
        buf.append(&mut name2_label.as_bytes().to_vec());
        // Pointer points past the end of the message.
        buf.put_u16(0xc000 | (buf.len() + 20) as u16);

        let mut unparsed = &buf[name2_ofs..];
        assert!(parse(&buf[..], &mut unparsed).is_err());
    }

    #[test]
    fn parse_incomplete_label() {
        let mut buf = Vec::new();

        // First name is "name1".
        let name1 = "name1";
        // Make the length such that more data is expected than is present.
        buf.put_u8(name1.len() as u8 + 10);
        buf.append(&mut name1.as_bytes().to_vec());
        // 0 byte for NULL label.
        buf.put_u8(0);

        let mut unparsed = &buf[..];
        assert!(parse(&buf[..], &mut unparsed).is_err());
    }

    #[test]
    fn parse_label_invalid_utf8() {
        let mut buf = Vec::new();

        // Name should be unicode "Ф", but an invalid UTF-8 encoding is used.
        let cp = 0x424;
        let b1 = 0xe0_u8 | ((cp >> 6) & 0x1f) as u8;
        let b2 = 0xc0_u8 | (cp & 0x3f) as u8;
        buf.put_u8(2);
        buf.put_u8(b1);
        buf.put_u8(b2);
        // 0 byte for NULL label.
        buf.put_u8(0);

        let mut unparsed = &buf[..];
        assert!(parse(&buf[..], &mut unparsed).is_err());
    }

    #[test]
    fn parse_label_not_ascii() {
        let mut buf = Vec::new();

        // Name is unicode "Ф".
        let cp = 0x424;
        let b1 = 0xc0_u8 | ((cp >> 6) & 0x1f) as u8;
        let b2 = 0x80_u8 | (cp & 0x3f) as u8;
        let name1 = vec![b1, b2];
        let name1 = String::from_utf8(name1).expect("mistake in utf-8 encoding for test");
        buf.put_u8(name1.len() as u8);
        buf.append(&mut name1.as_bytes().to_vec());
        // 0 byte for NULL label.
        buf.put_u8(0);

        let mut unparsed = &buf[..];
        assert!(parse(&buf[..], &mut unparsed).is_err());
    }

    #[test]
    fn parse_label_too_long() {
        let mut buf = Vec::new();

        // Name is a single label that's 64 characters, exceeding the maximum label length of 63.
        let name1 = "abcdefghij".repeat(6) + "abcd";
        buf.put_u8(name1.len() as u8);
        buf.append(&mut name1.as_bytes().to_vec());
        // 0 byte for NULL label.
        buf.put_u8(0);

        let mut unparsed = &buf[..];
        assert!(parse(&buf[..], &mut unparsed).is_err());
    }

    #[test]
    fn parse_name_too_long() {
        let mut buf = Vec::new();

        // FName consists of 5 labels, the first 4 60 characters each and the last 16 characters.
        // This makes the name 256 characters long, which exceeds the maximum name length of 255.
        let first4_labels = "abcdefghij".repeat(6);
        for _ in 0..4 {
            buf.put_u8(first4_labels.len() as u8);
            buf.append(&mut first4_labels.as_bytes().to_vec());
        }
        let last_label = "abcdefghijklmnop";
        buf.put_u8(last_label.len() as u8);
        buf.append(&mut last_label.as_bytes().to_vec());
        // 0 byte for NULL label.
        buf.put_u8(0);

        let mut unparsed = &buf[..];
        assert!(parse(&buf[..], &mut unparsed).is_err());
    }
}
