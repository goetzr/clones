use bytes::buf::{Buf, BufMut};
use std::net::TcpStream;
use std::io::Write;

const NAMESERVER_IP: &'static str = "172.20.10.1";

fn encode_qname(name: &str) -> Vec<u8> {
    let mut qname = Vec::new();

    name.split('.').for_each(|label| {
        qname.put_u8(label.len() as u8);
        qname.put_slice(label.as_bytes());
    });
    qname.put_u8(0);

    qname
}

fn build_request() -> Vec<u8> {
    let mut req = Vec::new();

    // Header
    let id = 1;
    let qr: u16 = 0; // query
    let opcode: u16 = 0; // standard query
    let aa: u16 = 0; // authoritative answer: ignored in query
    let tc: u16 = 0; // truncation: ignore in query
    let rd: u16 = 0; // recursion not desired
    let ra: u16 = 0; // recursion available: ignore in query
    let z: u16 = 0; // must be zero
    let rcode: u16 = 0; // response code: ignore in query
    let qdcount = 1; // number of entries in question section
    let ancount = 0; // number of entries in answer section: ignored in query
    let nscount = 0; // number of entries in authority section: ignored in query
    let arcount = 0; // number of entries in additional record section: ignored in query

    req.put_u16(id);
    let word2 = (rcode & 0xf) << 12
        | (z & 0x7) << 9
        | (ra & 1) << 8
        | (rd & 1) << 7
        | (tc & 1) << 6
        | (aa & 1) << 5
        | (opcode & 0xf) << 1
        | (qr & 1);
    req.put_u16(word2);
    req.put_u16(qdcount);
    req.put_u16(ancount);
    req.put_u16(nscount);
    req.put_u16(arcount);

    // Question
    req.put_slice(&encode_qname("google.com"));
    let qtype = 1; // A (host address)
    let qclass = 1; // IN (internet)
    req.put_u16(qtype);
    req.put_u16(qclass);

    req
}

fn parse_qname(qname: &[u8]) -> String {

    String::from("blah")
}

fn parse_response(response: &[u8]) -> String {
    // Header
    let id = response.get_u16();
    let word2 = response.get_u16();
    let rcode = (word2 >> 12) & 0xf;
    let ra = (word2 >> 8) & 1;
    let rd = (word2 >> 7) & 1;
    let tc = (word2 >> 6) & 1;
    let aa = (word2 >> 5) & 1;
    let opcode = (word2 >> 1) & 0xf;
    let qr = word2 & 1;
    let qdcount = response.get_u16();
    let ancount = response.get_u16();
    let nscount = response.get_u16();
    let arcount = response.get_u16();

    if qdcount != 0 {
        panic!("Question(s) in response");
    }

    // Answer
    // TODO: Need to pass reference to current location in response
    let name = parse_qname(
    
    String::from("0.0.0.0")
}

fn main() {
    // Resolve a hostname to an IP address:
    // 1. Connect to nameserver
    // 2. Build a type A request
    // 3. Send the request to the server
    // 4. Receive and parse the response
    let mut ns_sock = TcpStream::connect((NAMESERVER_IP, 53)).unwrap();
    let req = build_request();
    ns_sock.write(&req).unwrap();


    println!("Hello from the resolver");
}
