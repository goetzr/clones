use bytes::buf::{Buf, BufMut};
use bytes::Bytes;
use std::io::Read;
use std::io::Write;
use std::net::TcpStream;

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

fn parse_qname(sock: &mut TcpStream) -> String {
    let mut name = String::new();

    let mut buf = [0u8; 1];
    let mut label_buf;
    loop {
        sock.read_exact(&mut buf[0..1]).unwrap();
        let len = u8::from_be_bytes(buf.try_into().unwrap());
        if len == 0 {
            break;
        }
        if !name.is_empty() {
            name.push('.');
        }
        label_buf = vec![0; len as usize];
        sock.read_exact(&mut label_buf).unwrap();
        name.push_str(&String::from_utf8(label_buf).unwrap());
    }

    name
}

fn parse_response(sock: &mut TcpStream) -> String {
    let mut header = [0u8; 12];
    sock.read_exact(&mut header).unwrap();
    let mut header = Bytes::from(header.to_vec());

    // Header
    let _id = header.get_u16();
    let word2 = header.get_u16();
    let _rcode = (word2 >> 12) & 0xf;
    let _ra = (word2 >> 8) & 1;
    let _rd = (word2 >> 7) & 1;
    let _tc = (word2 >> 6) & 1;
    let _aa = (word2 >> 5) & 1;
    let _opcode = (word2 >> 1) & 0xf;
    let _qr = word2 & 1;
    let qdcount = header.get_u16();
    assert_ne!(qdcount, 0, "Question(s) in header");
    let ancount = header.get_u16();
    assert!(ancount > 0, "Expected at least one answer");
    let _nscount = header.get_u16();
    let _arcount = header.get_u16();

    // Answer
    let _name = parse_qname(&mut sock);
    let mut buf = [0u8; 4];
    sock.read_exact(&mut buf[0..2]).unwrap();
    let r#type = u16::from_be_bytes(buf[0..2].try_into().unwrap());
    assert_eq!(r#type, 1, "Type must be A");
    sock.read_exact(&mut buf[0..2]).unwrap();
    let class = u16::from_be_bytes(buf[0..2].try_into().unwrap());
    assert_eq!(class, 1, "Class must be IN");
    sock.read_exact(&mut buf[0..2]).unwrap();
    let _ttl = u16::from_be_bytes(buf[0..2].try_into().unwrap());
    sock.read_exact(&mut buf[0..2]).unwrap();
    let _rdlength = u16::from_be_bytes(buf[0..2].try_into().unwrap());
    sock.read_exact(&mut buf[0..4]).unwrap();
    let rdata = u32::from_be_bytes(buf[0..2].try_into().unwrap());

    let octets: [u8; 4] = [
        ((rdata >> 24) & 0xff) as u8,
        ((rdata >> 16) & 0xff) as u8,
        ((rdata >> 8) & 0xff) as u8,
        (rdata & 0xff) as u8,
    ];
    let octets = octets
        .into_iter()
        .map(|b| b.to_string())
        .collect::<Vec<_>>();
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
    let _ip_addr = parse_response(&mut ns_sock);

    println!("Hello from the resolver");
}
