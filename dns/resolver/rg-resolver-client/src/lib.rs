use std::net::IpAddr;

use serde::{Deserialize, Serialize};

mod request {
    use std::net::IpAddr;

    use rg_resolver_common::{QCLASS, QNAME, QTYPE};
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Serialize, Deserialize)]
    pub struct HostNameToAddress {
        hostname: String,
    }

    impl HostNameToAddress {
        pub fn new(hostname: String) -> HostNameToAddress {
            HostNameToAddress { hostname }
        }
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct AddressToHostName<T> {
        address: T,
    }

    impl<T> AddressToHostName<T>
    where
        T: Into<IpAddr>,
    {
        pub fn new(address: T) -> AddressToHostName<T> {
            AddressToHostName { address }
        }
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct General {
        qname: QNAME,
        qtype: QTYPE,
        qclass: QCLASS,
    }

    impl General {
        pub fn new(qname: QNAME, qtype: QTYPE, qclass: QCLASS) -> General {
            General { qname, qtype, qclass }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
