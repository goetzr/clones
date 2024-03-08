use bytes::buf::{Buf, BufMut};
use bytes::Bytes;
use std::io::Write;
use std::net::TcpStream;
use std::io::Read;

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
    let mut qname = Bytes::from(qname.to_vec());
    let mut name = String::new();

    let mut len = qname.get_u8();
    let mut buf = vec![0; len as usize];
    qname.copy_to_slice(&mut buf);
    name.push_str(&String::from_utf8(buf).unwrap());
    while !qname.is_empty() {
        len = qname.get_u8();
        if len == 0 {
            break;
        }
        name.push('.');
        buf = vec![0; len as usize];
        qname.copy_to_slice(&mut buf);
        name.push_str(&String::from_utf8(buf).unwrap());
    }

    name
}

fn parse_response(sock: &TcpStream) -> String {
    // TODO: Read from socket
    // TODO: Read big-endian

    let mut header = [0u8; 12];
    sock.read_exact(&mut header);
    sock.
    let mut response = Bytes::from(response.to_vec());

    // Header
    let _id = response.get_u16();
    let word2 = response.get_u16();
    let _rcode = (word2 >> 12) & 0xf;
    let _ra = (word2 >> 8) & 1;
    let _rd = (word2 >> 7) & 1;
    let _tc = (word2 >> 6) & 1;
    let _aa = (word2 >> 5) & 1;
    let _opcode = (word2 >> 1) & 0xf;
    let _qr = word2 & 1;
    let qdcount = response.get_u16();
    assert_ne!(qdcount, 0, "Question(s) in response");
    let _ancount = response.get_u16();
    let _nscount = response.get_u16();
    let _arcount = response.get_u16();

    // Answer
    let _name = parse_qname(&response[..]);
    let r#type = response.get_u16();
    assert_eq!(r#type, 1, "Type must be A");
    let class = response.get_u16();
    assert_eq!(class, 1, "Class must be IN");
    let _ttl = response.get_u32();
    let _rdlength = response.get_u16();
    let rdata = response.get_u32();

    let octets: [u8; 4] = [
        ((rdata >> 24) & 0xff) as u8,
        ((rdata >> 16) & 0xff) as u8,
        ((rdata >> 8) & 0xff) as u8,
        (rdata & 0xff) as u8,
    ];
    let octets = octets.into_iter().map(|b| b.to_string()).collect::<Vec<_>>();
    octets.join(".")
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
    let ip_addr = parse_response(&ns_sock);

    println!("Hello from the resolver");
}
