use crate::rr;
use bytes::Buf;

pub struct Message {
    header: Header,
    question: Vec<Question>,
    answer: Option<Vec<rr::ResourceRecord>>,
    authority: Option<Vec<rr::ResourceRecord>>,
    additional: Option<Vec<rr::ResourceRecord>>,
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
    nameserver_count: usize,
    additional_count: usize,
}

impl Header {
    fn parse(hdr: &mut &[u8]) -> anyhow::Result<Header> {
        macro_rules! check_remaining {
            ($size:expr, $field:expr) => {
                if hdr.remaining() < $size {
                    anyhow::bail!("incomplete header field: {}", $field);
                }
            };
        }
        check_remaining!(2, "id");
        let id = hdr.get_u16();

        check_remaining!(2, "bitfields");
        let bitfields = hdr.get_u16();
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
        let question_count = hdr.get_u16() as usize;
        check_remaining!(2, "answer count");
        let answer_count = hdr.get_u16() as usize;
        check_remaining!(2, "nameserver count");
        let nameserver_count = hdr.get_u16() as usize;
        check_remaining!(2, "additional count");
        let additional_count = hdr.get_u16() as usize;

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
            nameserver_count,
            additional_count,
        };
        Ok(header)
    }
}

#[derive(Debug, PartialEq)]
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

    fn serialize(&self) -> u8 {
        use Opcode::*;
        match self {
            StandardQuery => 0,
            InverseQuery => 1,
            ServerStatusRequest => 2
        }
    }
}

#[derive(Debug, PartialEq)]
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

    fn serialize(&self) -> u8 {
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

pub enum QuestionType {
    RrType(rr::Type),
    Afxr,
    Mailb,
    Maila,
    All,
}

pub enum QuestionClass {
    RrClass(rr::Class),
    Any,
}

pub struct Answer {}

pub struct Authority {}

pub struct Additional {}

#[cfg(test)]
mod test {
    use super::*;
    use bytes::BufMut;

    #[test]
    fn parse_header() -> anyhow::Result<()> {
        let mut buf = Vec::new();

        let id = 27;
        buf.put_u16(id);

        let is_response = true;
        let opcode = Opcode::StandardQuery;
        let is_authoritative_answer = true;
        let is_truncated = false;
        let is_recursion_desired = true;
        let is_recursion_available = false;
        let response_code = ResponseCode::NameError;
        let bitfields: u16 =
            (is_response as u16) << 15 |
            (opcode.serialize() as u16) << 11 |
            (is_authoritative_answer as u16) << 10 |
            (is_truncated as u16) << 9 |
            (is_recursion_desired as u16) << 8 |
            (is_recursion_available as u16) << 7 |
            response_code.serialize() as u16;
        buf.put_u16(bitfields);

        let question_count = 2;
        buf.put_u16(question_count);
        let answer_count = 4;
        buf.put_u16(answer_count);
        let nameserver_count = 1;
        buf.put_u16(nameserver_count);
        let additional_count = 3;
        buf.put_u16(additional_count);

        let mut hdr = &buf[..];
        let header = Header::parse(&mut hdr)?;

        assert_eq!(header.id, id);
        assert_eq!(header.is_response, is_response);
        assert_eq!(header.opcode, opcode);
        assert_eq!(header.is_authoritative_answer, is_authoritative_answer);
        assert_eq!(header.is_truncated, is_truncated);
        assert_eq!(header.is_recursion_desired, is_recursion_desired);
        assert_eq!(header.is_recursion_available, is_recursion_available);
        assert_eq!(header.response_code, response_code);
        assert_eq!(header.question_count, question_count as usize);
        assert_eq!(header.answer_count, answer_count as usize);
        assert_eq!(header.nameserver_count, nameserver_count as usize);
        assert_eq!(header.additional_count, additional_count as usize);

        // TODO: Assert that hdr - buf is the right number of bytes.


        Ok(())
    }
}