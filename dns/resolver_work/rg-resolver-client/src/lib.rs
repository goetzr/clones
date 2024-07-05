use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU32, Ordering};

static NEXT_ID: AtomicU32 = AtomicU32::new(1);

fn next_id() -> u32 {
    NEXT_ID.fetch_add(1, Ordering::SeqCst)
}

pub fn hostname_to_address(hostname: String) -> String {
    let req = HostNameToAddress::new(next_id(), hostname);
    // Send request to server
    // Wait for response
    // Return response
}

#[derive(Serialize, Deserialize)]
struct JsonRpc {
    jsonrpc: String,
    id: u32,
    method: String,
}

impl JsonRpc {
    fn new(id: u32, method: String) -> JsonRpc {
        JsonRpc { jsonrpc: String::from("2.0"), id, method }
    }
}

#[derive(Serialize, Deserialize)]
struct HostNameToAddress {
    #[serde(flatten)]
    jsonrpc: JsonRpc,

    params: [String; 1],
}

impl HostNameToAddress {
    const METHOD_NAME: &'static str = "host_name_to_address";

    fn new(id: u32, domain_name: String) -> HostNameToAddress {
        let jsonrpc = JsonRpc::new(id, String::from(Self::METHOD_NAME));
        HostNameToAddress { jsonrpc, params: [domain_name] }
    }
}

#[derive(Serialize, Deserialize)]
struct AddressToHostname {
    #[serde(flatten)]
    jsonrpc: JsonRpc,

    params: [String; 1],
}

impl AddressToHostname {
    const METHOD_NAME: &'static str = "address_to_hostname";

    fn new(id: u32, address: String) -> AddressToHostname {
        let jsonrpc = JsonRpc::new(id, String::from(Self::METHOD_NAME));
        AddressToHostname { jsonrpc, params: [address] }
    }
}

#[derive(Serialize, Deserialize)]
struct GeneralLookup {
    #[serde(flatten)]
    jsonrpc: JsonRpc,

    #[serde(flatten)]
    params: GeneralLookupParams,
}

impl GeneralLookup {
    const METHOD_NAME: &'static str = "general_lookup";

    fn new(id: u32, qname: String, qtype: String, qclass: String) -> GeneralLookup {
        let jsonrpc = JsonRpc::new(id, String::from(Self::METHOD_NAME));
        let params = GeneralLookupParams::new(qname, qtype, qclass);
        GeneralLookup { jsonrpc, params }
    }
}

#[derive(Serialize, Deserialize)]
struct GeneralLookupParams {
    qname: String,
    qtype: String,
    qclass: String,
}

impl GeneralLookupParams {
    fn new(qname: String, qtype: String, qclass: String) -> GeneralLookupParams {
        GeneralLookupParams { qname, qtype, qclass }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
    }
}
