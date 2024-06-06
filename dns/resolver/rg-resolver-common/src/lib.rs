use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct QNAME;

#[derive(Debug, Serialize, Deserialize)]
pub struct QTYPE;

#[derive(Debug, Serialize, Deserialize)]
pub struct QCLASS;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
