use crate::{name, rr};
use bytes::{Buf, BufMut};

pub struct Message {
    header: Header,
    questions: Vec<Question>,
    answers: Vec<rr::ResourceRecord>,
    authorities: Vec<rr::ResourceRecord>,
    additionals: Vec<rr::ResourceRecord>,
}

impl Message {
    pub fn parse(msg: &mut &[u8]) -> anyhow::Result<Message> {
        // Keep msg pointing at the first byte of the message until the very end.
        let mut unparsed = *msg;
        let header = Header::parse(&mut unparsed)?;

        let mut questions = Vec::with_capacity(header.question_count);
        for _ in 0..header.question_count {
            let question = Question::parse(msg, &mut unparsed)?;
            questions.push(question);
        }

        let mut answers = Vec::with_capacity(header.answer_count);
        for _ in 0..header.answer_count {
            let answer = rr::ResourceRecord::parse(msg, &mut unparsed)?;
            answers.push(answer);
        }

        let mut authorities = Vec::with_capacity(header.authority_count);
        for _ in 0..header.authority_count {
            let authority = rr::ResourceRecord::parse(msg, &mut unparsed)?;
            authorities.push(authority);
        }

        let mut additionals = Vec::with_capacity(header.additional_count);
        for _ in 0..header.additional_count {
            let additional = rr::ResourceRecord::parse(msg, &mut unparsed)?;
            additionals.push(additional);
        }

        let message = Message {
            header,
            questions,
            answers,
            authorities,
            additionals,
        };
        Ok(message)
    }
}
pub struct Header {
    id: u16,
    is_response: bool,
    opcode: Opcode,
    is_authoritative_answer: bool,
    is_truncated: bool,
    is_recursion_desired: bool,
    is_recursion_available: bool,
    response_code: ResponseCode,
    question_count: usize,
    answer_count: usize,
    authority_count: usize,
    additional_count: usize,
}

impl Header {
    fn parse(unparsed: &mut &[u8]) -> anyhow::Result<Header> {
        macro_rules! check_remaining {
            ($size:expr, $field:expr) => {
                if unparsed.remaining() < $size {
                    anyhow::bail!("incomplete header field: {}", $field);
                }
            };
        }
        check_remaining!(2, "id");
        let id = unparsed.get_u16();

        check_remaining!(2, "bitfields");
        let bitfields = unparsed.get_u16();
        let is_response = (bitfields >> 15) & 1 == 1;
        let opcode = Opcode::parse(bitfields)?;
        let is_authoritative_answer = (bitfields >> 10) & 1 == 1;
        let is_truncated = (bitfields >> 9) & 1 == 1;
        let is_recursion_desired = (bitfields >> 8) & 1 == 1;
        let is_recursion_available = (bitfields >> 7) & 1 == 1;
        let zeros = (bitfields >> 4) & 7;
        if zeros != 0 {
            anyhow::bail!("reserved area in header must be all zeros");
        }
        let response_code = ResponseCode::parse(bitfields)?;

        check_remaining!(2, "question count");
        let question_count = unparsed.get_u16() as usize;
        check_remaining!(2, "answer count");
        let answer_count = unparsed.get_u16() as usize;
        check_remaining!(2, "authority count");
        let authority_count = unparsed.get_u16() as usize;
        check_remaining!(2, "additional count");
        let additional_count = unparsed.get_u16() as usize;

        let header = Header {
            id,
            is_response,
            opcode,
            is_authoritative_answer,
            is_truncated,
            is_recursion_desired,
            is_recursion_available,
            response_code,
            question_count,
            answer_count,
            authority_count: authority_count,
            additional_count,
        };
        Ok(header)
    }

    fn serialize(&self) -> Vec<u8> {
        let mut buf = Vec::new();

        buf.put_u16(self.id);
        let bitfields: u16 = (self.is_response as u16) << 15
            | (self.opcode.serialize() as u16) << 11
            | (self.is_authoritative_answer as u16) << 10
            | (self.is_truncated as u16) << 9
            | (self.is_recursion_desired as u16) << 8
            | (self.is_recursion_available as u16) << 7
            | self.response_code.serialize() as u16;
        buf.put_u16(bitfields);
        buf.put_u16(self.question_count as u16);
        buf.put_u16(self.answer_count as u16);
        buf.put_u16(self.authority_count as u16);
        buf.put_u16(self.additional_count as u16);

        buf
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
enum Opcode {
    StandardQuery,
    InverseQuery,
    ServerStatusRequest,
}

impl Opcode {
    fn parse(bitfields: u16) -> anyhow::Result<Self> {
        match (bitfields >> 11) & 0xf {
            0 => Ok(Opcode::StandardQuery),
            1 => Ok(Opcode::InverseQuery),
            2 => Ok(Opcode::ServerStatusRequest),
            n => Err(anyhow::anyhow!("reserved opcode: {n}")),
        }
    }

    fn serialize(&self) -> u16 {
        panic!("Make this return the shifted result");
        use Opcode::*;
        match self {
            StandardQuery => 0,
            InverseQuery => 1,
            ServerStatusRequest => 2,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
enum ResponseCode {
    NoError,
    FormatError,
    ServerFailure,
    NameError,
    NotImplemented,
    Refused,
}

impl ResponseCode {
    fn parse(bitfields: u16) -> anyhow::Result<Self> {
        match bitfields & 0xf {
            0 => Ok(ResponseCode::NoError),
            1 => Ok(ResponseCode::FormatError),
            2 => Ok(ResponseCode::ServerFailure),
            3 => Ok(ResponseCode::NameError),
            4 => Ok(ResponseCode::NotImplemented),
            5 => Ok(ResponseCode::Refused),
            n => Err(anyhow::anyhow!("reserved response code: {n}")),
        }
    }

    fn serialize(&self) -> u16 {
        panic!("Make this return the shifted result");
        use ResponseCode::*;
        match self {
            NoError => 0,
            FormatError => 1,
            ServerFailure => 2,
            NameError => 3,
            NotImplemented => 4,
            Refused => 5,
        }
    }
}

pub struct Question {
    name: String,
    r#type: QuestionType,
    class: QuestionClass,
}

impl Question {
    /// * msg must point to the very first byte of the message.
    fn parse<'a>(msg: &'a [u8], unparsed: &mut &'a [u8]) -> anyhow::Result<Self> {
        let name = name::parse(msg, unparsed)?;
        let r#type = QuestionType::parse(unparsed)?;
        let class = QuestionClass::parse(unparsed)?;

        let question = Question {
            name,
            r#type,
            class,
        };
        Ok(question)
    }

    fn serialize(&self) -> anyhow::Result<Vec<u8>> {
        let mut buf = Vec::new();
        // * The question section holds the first name in the message, so it can't be compressed.
        let mut name = name::serialize(&self.name, None)?;
        buf.append(&mut name);
        buf.put_u16(self.r#type.serialize());
        buf.put_u16(self.class.serialize());

        Ok(buf)
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum QuestionType {
    RrType(rr::Type),
    Afxr,
    Mailb,
    Maila,
    All,
}

impl QuestionType {
    fn parse(unparsed: &mut &[u8]) -> anyhow::Result<Self> {
        use crate::rr;
        use QuestionType::*;

        if unparsed.remaining() < 2 {
            anyhow::bail!("incomplete question type");
        }

        // Attempt to parse a base resource record type first by peeking at the value.
        let mut peek = *unparsed;
        let question_type = match rr::Type::parse(&mut peek) {
            Ok(rr_type) => {
                // Base resource record type. Advance past the peeked at value.
                unparsed.advance(2);
                RrType(rr_type)
            }
            Err(_) => {
                // Not a base resource record type. Check the remaining possibilities.
                match unparsed.get_u16() {
                    252 => Afxr,
                    253 => Mailb,
                    254 => Maila,
                    255 => All,
                    n => anyhow::bail!("undefined question type {n}"),
                }
            }
        };

        Ok(question_type)
    }

    fn serialize(&self) -> u16 {
        use QuestionType::*;

        match self {
            RrType(rr_type) => rr_type.serialize(),
            Afxr => 252,
            Mailb => 253,
            Maila => 254,
            All => 255,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum QuestionClass {
    RrClass(rr::Class),
    Any,
}

impl QuestionClass {
    fn parse(unparsed: &mut &[u8]) -> anyhow::Result<Self> {
        use crate::rr;
        use QuestionClass::*;

        if unparsed.remaining() < 2 {
            anyhow::bail!("incomplete question class");
        }

        // Attempt to parse a base resource record class first by peeking at the value.
        let mut peek = *unparsed;
        let question_class = match rr::Class::parse(&mut peek) {
            Ok(rr_class) => {
                // Base resource record class. Advance past the peeked at value.
                unparsed.advance(2);
                RrClass(rr_class)
            }
            Err(_) => {
                // Not a base resource record class. Check the remaining possibilities.
                match unparsed.get_u16() {
                    255 => Any,
                    n => anyhow::bail!("undefined question class {n}"),
                }
            }
        };

        Ok(question_class)
    }

    fn serialize(&self) -> u16 {
        use QuestionClass::*;

        match self {
            RrClass(rr_class) => rr_class.serialize(),
            Any => 255,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use bytes::BufMut;

    #[test]
    fn parse_opcode() -> anyhow::Result<()> {
        assert_eq!(Opcode::parse(Opcode::StandardQuery.serialize())?, Opcode::StandardQuery);
        assert_eq!(Opcode::parse(Opcode::InverseQuery.serialize())?, Opcode::InverseQuery);
        assert_eq!(Opcode::parse(Opcode::ServerStatusRequest.serialize())?, Opcode::ServerStatusRequest);

        let bitfields = 3 << 11;
        assert!(Opcode::parse(bitfields).is_err());

        Ok(())
    }

    #[test]
    fn parse_response_code() -> anyhow::Result<()> {
        assert_eq!(ResponseCode::parse(ResponseCode::NoError.serialize())?, ResponseCode::NoError);
        assert_eq!(ResponseCode::parse(ResponseCode::FormatError.serialize())?, ResponseCode::FormatError);
        assert_eq!(ResponseCode::parse(ResponseCode::ServerFailure.serialize())?, ResponseCode::ServerFailure);
        assert_eq!(ResponseCode::parse(ResponseCode::NameError.serialize())?, ResponseCode::NameError);
        assert_eq!(
            ResponseCode::parse(ResponseCode::NotImplemented.serialize())?,
            ResponseCode::NotImplemented
        );
        assert_eq!(ResponseCode::parse(ResponseCode::Refused.serialize())?, ResponseCode::Refused);

        let bitfields = 6;
        assert!(ResponseCode::parse(bitfields).is_err());

        Ok(())
    }

    #[test]
    fn parse_header() -> anyhow::Result<()> {
        let header = Header {
            id: 7,
            is_response: true,
            opcode: Opcode::StandardQuery,
            is_authoritative_answer: true,
            is_truncated: false,
            is_recursion_desired: false,
            is_recursion_available: true,
            response_code: ResponseCode::NoError,
            question_count: 2,
            answer_count: 2,
            authority_count: 2,
            additional_count: 2,
        };
        let buf = header.serialize();

        let mut unparsed = &buf[..];
        let parsed_hdr = Header::parse(&mut unparsed)?;

        assert_eq!(parsed_hdr.id, header.id);
        assert_eq!(parsed_hdr.is_response, header.is_response);
        assert_eq!(parsed_hdr.opcode, header.opcode);
        assert_eq!(parsed_hdr.is_authoritative_answer, header.is_authoritative_answer);
        assert_eq!(parsed_hdr.is_truncated, header.is_truncated);
        assert_eq!(parsed_hdr.is_recursion_desired, header.is_recursion_desired);
        assert_eq!(parsed_hdr.is_recursion_available, header.is_recursion_available);
        assert_eq!(parsed_hdr.response_code, header.response_code);
        assert_eq!(parsed_hdr.question_count, header.question_count);
        assert_eq!(parsed_hdr.answer_count, header.answer_count);
        assert_eq!(parsed_hdr.authority_count, header.authority_count);
        assert_eq!(parsed_hdr.additional_count, header.additional_count);
        assert_eq!(unsafe { unparsed.as_ptr().offset_from(buf.as_ptr()) as usize }, buf.len());

        let mut unparsed = &buf[..1];
        assert!(Header::parse(&mut unparsed).is_err());
        let mut unparsed = &buf[..3];
        assert!(Header::parse(&mut unparsed).is_err());
        let mut unparsed = &buf[..5];
        assert!(Header::parse(&mut unparsed).is_err());
        let mut unparsed = &buf[..7];
        assert!(Header::parse(&mut unparsed).is_err());
        let mut unparsed = &buf[..9];
        assert!(Header::parse(&mut unparsed).is_err());
        let mut unparsed = &buf[..11];
        assert!(Header::parse(&mut unparsed).is_err());

        Ok(())
    }

    #[test]
    fn parse_question_type() -> anyhow::Result<()> {
        let mut buf = Vec::new();
        buf.put_u16(rr::Type::CNAME.serialize());
        let mut unparsed = &buf[..];
        assert_eq!(
            QuestionType::parse(&mut unparsed)?,
            QuestionType::RrType(rr::Type::CNAME)
        );
        assert_eq!(unsafe { unparsed.as_ptr().offset_from(buf.as_ptr()) }, 2);

        use QuestionType::*;
        macro_rules! test_qtype {
            ($result:expr) => {
                let mut buf = Vec::new();
                buf.put_u16($result.serialize());
                let mut unparsed = &buf[..];
                assert_eq!(QuestionType::parse(&mut unparsed)?, $result);
                assert_eq!(unsafe { unparsed.as_ptr().offset_from(buf.as_ptr()) as usize }, buf.len());
            };
        }

        test_qtype!(Afxr);
        test_qtype!(Mailb);
        test_qtype!(Maila);
        test_qtype!(All);

        let mut buf = Vec::new();
        buf.put_u16(256);
        let mut unparsed = &buf[..];
        assert!(QuestionType::parse(&mut unparsed).is_err());

        let mut buf = Vec::new();
        buf.put_u8(252);
        let mut unparsed = &buf[..];
        assert!(QuestionType::parse(&mut unparsed).is_err());

        Ok(())
    }

    #[test]
    fn parse_question_class() -> anyhow::Result<()> {
        let mut buf = Vec::new();
        buf.put_u16(rr::Class::IN.serialize());
        let mut unparsed = &buf[..];
        assert_eq!(
            QuestionClass::parse(&mut unparsed)?,
            QuestionClass::RrClass(rr::Class::IN)
        );
        assert_eq!(unsafe { unparsed.as_ptr().offset_from(buf.as_ptr()) as usize }, buf.len());

        let mut buf = Vec::new();
        buf.put_u16(QuestionClass::Any.serialize());
        let mut unparsed = &buf[..];
        assert_eq!(QuestionClass::parse(&mut unparsed)?, QuestionClass::Any);
        assert_eq!(unsafe { unparsed.as_ptr().offset_from(buf.as_ptr()) as usize }, buf.len());

        let mut buf = Vec::new();
        buf.put_u16(256);
        let mut unparsed = &buf[..];
        assert!(QuestionClass::parse(&mut unparsed).is_err());

        let mut buf = Vec::new();
        buf.put_u8(255);
        let mut unparsed = &buf[..];
        assert!(QuestionClass::parse(&mut unparsed).is_err());

        Ok(())
    }

    #[test]
    fn parse_question() -> anyhow::Result<()> {
        let question = Question {
            name: "google.com.".to_string(),
            r#type: QuestionType::RrType(rr::Type::CNAME),
            class: QuestionClass::RrClass(rr::Class::IN),
        };
        let buf = question.serialize()?;

        let mut unparsed = &buf[..];
        let question_parsed = Question::parse(&buf[..], &mut unparsed)?;

        assert_eq!(question_parsed.name, question.name);
        assert_eq!(question_parsed.r#type, question.r#type);
        assert_eq!(question_parsed.class, question.class);
        assert_eq!(unsafe { unparsed.as_ptr().offset_from(buf.as_ptr()) as usize }, buf.len());

        Ok(())
    }

    #[test]
    fn parse_message() -> anyhow::Result<()> {
        todo!("finish this test after writing serialize test");
        let mut buf = Vec::new();

        let header = Header {
            id: 7,
            is_response: true,
            opcode: Opcode::StandardQuery,
            is_authoritative_answer: true,
            is_truncated: false,
            is_recursion_desired: false,
            is_recursion_available: true,
            response_code: ResponseCode::NoError,
            question_count: 2,
            answer_count: 2,
            authority_count: 2,
            additional_count: 2,
        };
        buf.append(&mut header.serialize());

        let question1 = Question {
            name: "google.com.".to_string(),
            r#type: QuestionType::RrType(rr::Type::A),
            class: QuestionClass::RrClass(rr::Class::IN)
        };
        let question2 = Question {
            name: "amazon.com.".to_string(),
            r#type: QuestionType::RrType(rr::Type::A),
            class: QuestionClass::RrClass(rr::Class::IN)
        };
        buf.append(&mut question1.serialize()?);
        buf.append(&mut question2.serialize()?);

        let answer1 = rr::ResourceRecord {
            name: "google.com.".to_string(),
            r#type: rr::Type::A,
            class: rr::Class::IN,
            ttl: 100,
            data: Some(vec![113, 234, 56, 89]),
        };
        let answer2 = rr::ResourceRecord {
            name: "amazon.com.".to_string(),
            r#type: rr::Type::A,
            class: rr::Class::IN,
            ttl: 100,
            data: Some(vec![85, 107, 21, 77]),
        };
        buf.append(&mut answer1.serialize()?);
        
        Ok(())
    }

    #[test]
    fn serialize_opcode() {
        assert_eq!(Opcode::StandardQuery.serialize(), 0);
        assert_eq!(Opcode::InverseQuery.serialize(), 1);
        assert_eq!(Opcode::ServerStatusRequest.serialize(), 2);
    }

    #[test]
    fn serialize_response_code() {
        assert_eq!(ResponseCode::NoError.serialize(), 0);
        assert_eq!(ResponseCode::FormatError.serialize(), 1);
        assert_eq!(ResponseCode::ServerFailure.serialize(), 2);
        assert_eq!(ResponseCode::NameError.serialize(), 3);
        assert_eq!(ResponseCode::NotImplemented.serialize(), 4);
        assert_eq!(ResponseCode::Refused.serialize(), 5);
    }

    #[test]
    fn serialize_header() {
        let header = Header {
            id: 7,
            is_response: true,
            opcode: Opcode::StandardQuery,
            is_authoritative_answer: true,
            is_truncated: false,
            is_recursion_desired: false,
            is_recursion_available: true,
            response_code: ResponseCode::NoError,
            question_count: 2,
            answer_count: 2,
            authority_count: 2,
            additional_count: 2,
        };
        let buf = header.serialize();

        let mut cursor = buf.as_slice();
        assert_eq!(cursor.get_u16(), header.id);
        let bitfields = cursor.get_u16();
        assert_eq!((bitfields >> 15) & 1 != 0, header.is_response);
        assert_eq!((bitfields >> 11) & 0xf, header.opcode.serialize());
        assert_eq!((bitfields >> 10) & 1 != 0, header.is_authoritative_answer);
        assert_eq!((bitfields >> 9) & 1 != 0, header.is_truncated);
        assert_eq!((bitfields >> 8) & 1 != 0, header.is_recursion_desired);
        assert_eq!((bitfields >> 7) & 1 != 0, header.is_recursion_available);
        assert_eq!((bitfields >> 4) & 7, 0);
        assert_eq!(bitfields & 0xf, header.response_code.serialize());
        assert_eq!(cursor.get_u16(), header.question_count as u16);
        assert_eq!(cursor.get_u16(), header.answer_count as u16);
        assert_eq!(cursor.get_u16(), header.authority_count as u16);
        assert_eq!(cursor.get_u16(), header.additional_count as u16);
    }

    #[test]
    fn serialize_question_type() {
        assert_eq!(QuestionType::RrType(rr::Type::CNAME).serialize(), 5);
        assert_eq!(QuestionType::Afxr.serialize(), 252);
        assert_eq!(QuestionType::Mailb.serialize(), 253);
        assert_eq!(QuestionType::Maila.serialize(), 254);
        assert_eq!(QuestionType::All.serialize(), 255);
    }

    #[test]
    fn serialize_question_class() {
        assert_eq!(QuestionClass::RrClass(rr::Class::IN).serialize(), 1);
        assert_eq!(QuestionClass::Any.serialize(), 255);
    }

    #[test]
    fn serialize_question() -> anyhow::Result<()> {
        let question = Question {
            name: "google.com.".to_string(),
            r#type: QuestionType::RrType(rr::Type::CNAME),
            class: QuestionClass::RrClass(rr::Class::IN),
        };
        let buf = question.serialize()?;
        // * The question section holds the first name in the message, so it can't be compressed.
        let name_ser = name::serialize(&question.name, None)?;
        assert_eq!(&buf[..name_ser.len()], name_ser);
        let mut cursor = &buf[name_ser.len()..];
        assert_eq!(cursor.get_u16(), question.r#type.serialize());
        assert_eq!(cursor.get_u16(), question.class.serialize());

        Ok(())
    }

    #[test]
    fn serialize_message() {
        todo!("write this test first");
    }
}
