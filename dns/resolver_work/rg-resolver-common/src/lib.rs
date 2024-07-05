use serde::{Deserialize, Serialize};
use std::fmt::{self, Display};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    DomainName(DomainNameError),
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use Error::*;
        match self {
            DomainName(e) => write!(f, "invalid QNAME: {}", e),
        }
    }
}

impl std::error::Error for Error {}

#[derive(Debug)]
pub enum DomainNameError {
    Empty,
    FirstLabelMissing,
    InteriorLabelMissing,
    LabelTooLong(String),
    LabelNotAscii(String),
    NameTooLong,
}

impl Display for DomainNameError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use DomainNameError::*;
        match self {
            Empty => f.write_str("empty"),
            FirstLabelMissing => f.write_str("must start with a label"),
            InteriorLabelMissing => f.write_str("interior label missing"),
            LabelTooLong(label) => write!(
                f,
                "label '{}' exceeded max label length of {} characters",
                label,
                DomainName::MAX_LABEL_LENGTH
            ),
            LabelNotAscii(label) => write!(f, "label '{}' was not ASCII", label),
            NameTooLong => write!(
                f,
                "exceeded max length of {} characters",
                DomainName::MAX_LENGTH
            ),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DomainName {
    labels: Vec<String>,
}

impl DomainName {
    const MAX_LENGTH: usize = 255;
    const MAX_LABEL_LENGTH: usize = 63;

    pub fn new(name: String) -> Result<DomainName> {
        // TODO: Move this check to the resolver.
        if name.len() > DomainName::MAX_LENGTH {
            return Err(Error::DomainName(DomainNameError::NameTooLong));
        }
        let labels = name
            .split('.')
            .map(|lbl| lbl.trim())
            .map(String::from)
            .collect::<Vec<_>>();
        if labels.is_empty() {
            return Err(Error::DomainName(DomainNameError::Empty));
        }
        // A domain name must start with a label.
        if labels.first().unwrap().is_empty() {
            return Err(Error::DomainName(DomainNameError::FirstLabelMissing));
        }
        for (idx, label) in labels.iter().enumerate() {
            // An empty label not at the end means two '.' were found back-to-back without a label between them,
            // meaning an interior label is missing.
            // An empty label at the end is OK. It means the domain name ends in '.', which means
            // the domain name ends at the null root label.
            if idx + 1 < labels.len() && label.is_empty() {
                return Err(Error::DomainName(DomainNameError::InteriorLabelMissing));
            }
            if label.len() > DomainName::MAX_LABEL_LENGTH {
                return Err(Error::DomainName(DomainNameError::LabelTooLong(
                    label.clone(),
                )));
            }
            if !label.is_ascii() {
                return Err(Error::DomainName(DomainNameError::LabelNotAscii(
                    label.clone(),
                )));
            }
        }
        Ok(DomainName { labels })
    }

    pub fn is_absolute(&self) -> bool {
        self.labels.last().unwrap().len() == 0
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
        let qname = DomainName::new(name);
        assert!(
            qname.is_err() && matches!(qname, Err(Error::DomainName(DomainNameError::NameTooLong)))
        )
    }

    #[test]
    fn qname_not_ascii() {
        let mut name = String::from("test");
        name.push(char::from_u32(0x99).unwrap());
        name.push_str(".google.com");
        let qname = DomainName::new(name);
        assert!(
            qname.is_err() && matches!(qname, Err(Error::DomainName(DomainNameError::NotAscii)))
        )
    }

    #[test]
    fn qname_label_too_long() {
        let bad_label = "abcdefghij".repeat(7);
        let name = format!("test.{}.google.com", bad_label);
        let qname = DomainName::new(name);
        assert!(
            qname.is_err()
                && match qname {
                    Err(Error::DomainName(DomainNameError::LabelTooLong(label))) =>
                        label == bad_label,
                    _ => false,
                }
        )
    }
}
