use serde::{Serialize, Deserialize};
use std::fmt::{self, Display};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    Qname(QnameError),
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use Error::*;
        match self {
            Qname(e) => write!(f, "invalid QNAME: {}", e),
        }
    }
}

impl std::error::Error for Error {}

#[derive(Debug)]
pub enum QnameError {
    NotAscii,
    LabelTooLong(String),
    NameTooLong,
}

impl Display for QnameError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use QnameError::*;
        match self {
            NotAscii => f.write_str("not ASCII"),
            LabelTooLong(label) => write!(f, "label '{}' exceeded max label length of {} characters", label, Qname::MAX_LABEL_LENGTH),
            NameTooLong => write!(f, "exceeded max length of {} characters", Qname::MAX_LENGTH),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Qname {
    labels: Vec<String>,
}

impl Qname {
    const MAX_LENGTH: usize = 255;
    const MAX_LABEL_LENGTH: usize = 63;

    pub fn new(name: String) -> Result<Qname> {
        // TODO: Need to consider who is responsible for resolving relative domain names.
        if name.len() > Qname::MAX_LENGTH {
            return Err(Error::Qname(QnameError::NameTooLong));
        }
        if !name.is_ascii() {
            return Err(Error::Qname(QnameError::NotAscii));
        }
        let labels = name.split('.').map(String::from).collect::<Vec<_>>();
        for label in &labels {
            if label.len() > Qname::MAX_LABEL_LENGTH {
                return Err(Error::Qname(QnameError::LabelTooLong(label.clone())));
            }
        }
        Ok(Qname { labels })
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Qtype;

#[derive(Debug, Serialize, Deserialize)]
pub struct Qclass;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn qname_too_long() {
        let name = "abcdefghij".repeat(30);
        let qname = Qname::new(name);
        assert!(
            qname.is_err() &&
            matches!(qname, Err(Error::Qname(QnameError::NameTooLong)))
        )
    }

    #[test]
    fn qname_not_ascii() {
        let mut name = String::from("test");
        name.push(char::from_u32(0x99).unwrap());
        name.push_str(".google.com");
        let qname = Qname::new(name);
        assert!(
            qname.is_err() &&
            matches!(qname, Err(Error::Qname(QnameError::NotAscii)))
        )
    }

    #[test]
    fn qname_label_too_long() {
        let bad_label = "abcdefghij".repeat(7);
        let name = format!("test.{}.google.com", bad_label);
        let qname = Qname::new(name);
        assert!(
            qname.is_err() &&
            match qname {
                Err(Error::Qname(QnameError::LabelTooLong(label))) => label == bad_label,
                _ => false
            }
        )
    }
}
