use crate::rr;

pub struct Message {
    header: Header,
    question: Vec<Question>,
    answer: Option<Vec<rr::ResourceRecord>>,
    authority: Option<Vec<rr::ResourceRecord>>,
    additional: Option<Vec<rr::ResourceRecord>>,
}
pub struct Header {
    id: u16,
    r#type: MessageType,
    opcode: Opcode,
    is_authoritative_answer: bool,
    is_truncated: bool,
    is_recursion_desired: bool,
    is_recursion_available: bool,
    response_code: ResponseCode,
}

enum MessageType {
    Query,
    Response,
}

enum Opcode {
    StandardQuery,
    InverseQuery,
    ServerStatusRequest,
    Reserved(u8),
}

enum ResponseCode {
    NoError,
    FormatError,
    ServerFailure,
    NameError,
    NotImplemented,
    Refused,
    Reserver(u8),
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

pub struct Answer {

}

pub struct Authority {

}

pub struct Additional {

}