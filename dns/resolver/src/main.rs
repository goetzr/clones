use bytes::buf::{Buf, BufMut};
use std::io::Read;
use std::io::Write;
use std::net::TcpStream;

fn encode_qname(name: &str) -> Vec<u8> {
    let mut qname = Vec::new();

    name.split('.').for_each(|label| {
        qname.put_u8(label.len() as u8);
        qname.put_slice(label.as_bytes());
    });
    qname.put_u8(0);

    qname
}

fn build_request(hostname: &str) -> Vec<u8> {
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

    req.put_u16(0);  // place holder for length
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
    req.put_slice(&encode_qname(hostname));
    let qtype = 1; // A (host address)
    let qclass = 1; // IN (internet)
    req.put_u16(qtype);
    req.put_u16(qclass);

    let reqn: u16 = req.len() as u16 - 2;
    (&mut req[0..2]).write(&reqn.to_be_bytes()).unwrap();

    println!("");
    println!("Request:");
    display_buffer(&req);

    req
}

fn parse_name(msg: &mut [u8], index: &mut usize) -> String {
    let mut name = String::new();
    let mut new_idx = *index;

    let mut append_to_name = |part: String| {
        if !name.is_empty() {
            name.push('.');
        }
        name.push_str(part.as_str());
    };
    
    loop {
        let len = &msg[new_idx..new_idx+1];
        let len = u8::from_be_bytes(len.try_into().unwrap()) as usize;
        if len & 0xc0 == 0xc0 {
            // Pointer. 
            // The next byte contains the low 8 bits of the 14-bit index
            // of the pointed-to name.
            let low_byte = u8::from_be_bytes(msg[new_idx+1..new_idx+2].try_into().unwrap());
            new_idx += 2;
            let mut pointee_idx = (len & 0x3f) << 8 | low_byte as usize;
            // Parse the pointed to name from the message.
            let subname = parse_name(msg, &mut pointee_idx);
            append_to_name(subname);
            break;
        }
        new_idx += 1;
        if len == 0 {
            break;
        }
        let label = &mut msg[new_idx..new_idx+len];
        let label = String::from_utf8(label.to_vec()).unwrap();
        append_to_name(label);
        new_idx += len;
    }

    *index = new_idx;
    name
}

fn parse_response(sock: &mut TcpStream) -> String {
    println!("");
    println!("Waiting for response...");
    let mut size = [0u8; 2];
    sock.read_exact(&mut size).unwrap();
    let size = u16::from_be_bytes(size) as usize;
    println!("Received {size} byte response:");

    let mut msg = vec![0u8; size];
    sock.read_exact(&mut msg).unwrap();
    display_buffer(&msg);
    let mut index : usize = 0;

    let mut header = &msg[0..12];
    index += 12;

    // Header
    println!("");
    println!("Header:");
    let id = header.get_u16();
    println!("id = {id}");
    let mut word2 = header.get_u16();
    let rcode = word2 & 0xf;
    word2 >>= 4;
    println!("rcode = {rcode}");
    word2 >>= 3;    // Discard zero bits
    let ra = word2 & 1;
    word2 >>= 1;
    println!("ra = {ra}");
    let rd = word2 & 1;
    word2 >>= 1;
    println!("rd = {rd}");
    let tc = word2 & 1;
    word2 >>= 1;
    println!("tc = {tc}");
    let aa = word2 & 1;
    word2 >>= 1;
    println!("aa = {aa}");
    let opcode = word2 & 0xf;
    word2 >>= 4;
    println!("opcode = {opcode}");
    let qr = word2 & 1;
    println!("qr = {qr}");
    let qdcount = header.get_u16();
    println!("qdcount = {qdcount}");
    let ancount = header.get_u16();
    println!("ancount = {ancount}");
    assert!(ancount > 0, "Expected at least one answer");
    let nscount = header.get_u16();
    println!("nscount = {nscount}");
    let arcount = header.get_u16();
    println!("arcount = {arcount}");

    // Question
    if qdcount == 1 {
        println!("");
        println!("Question:");

        let qname = parse_name(&mut msg, &mut index);
        println!("qname = {qname}");

        let qcode = &msg[index..index+2];
        let qcode = u16::from_be_bytes(qcode[0..2].try_into().unwrap());
        index += 2;
        println!("qcode = {qcode}");

        let qclass = &msg[index..index+2];
        let qclass = u16::from_be_bytes(qclass[0..2].try_into().unwrap());
        index += 2;
        println!("qclass = {qclass}");
    }

    // Answer
    println!("");
    println!("Answer:");

    let name = parse_name(&mut msg, &mut index);
    println!("name = {name}");

    let r#type = &msg[index..index+2];
    index += 2;
    let r#type = u16::from_be_bytes(r#type[0..2].try_into().unwrap());
    println!("type = {}", r#type);
    assert_eq!(r#type, 1, "Type must be A");

    let class = &msg[index..index+2];
    index += 2;
    let class = u16::from_be_bytes(class[0..2].try_into().unwrap());
    println!("class = {class}");
    assert_eq!(class, 1, "Class must be IN");

    let ttl = &msg[index..index+4];
    index += 4;
    let ttl = u32::from_be_bytes(ttl[0..4].try_into().unwrap());
    println!("ttl = {ttl}");

    let rdlength = &msg[index..index+2];
    index += 2;
    let rdlength = u16::from_be_bytes(rdlength[0..2].try_into().unwrap());
    println!("rdlength = {rdlength}");

    let rdata = &msg[index..index+4];
    //index += 4;
    let rdata = u32::from_be_bytes(rdata[0..4].try_into().unwrap());

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

fn display_buffer(buf: &[u8]) {
    fn display_line(line: &mut String, bytes: &mut Vec<u8>) {
        *line += "| ";
        for &b in bytes.iter() {
            let c = if b.is_ascii_graphic() { b as char } else { '.' };
            let ascii = format!("{}", c);
            *line += &ascii;
        }
        println!("{line}");
        line.clear();
        bytes.clear();
    }

    let mut line = String::new();
    let mut bytes = Vec::new();
    for (idx, &byte) in buf.into_iter().enumerate() {
        bytes.push(byte);
        if idx % 16 == 0 {
            let address = format!("{idx:02X}: ");
            line += &address;
        }
        let hex = format!("{byte:02X} ");
        line += &hex;
        if (idx + 1) % 16 == 0 {
            display_line(&mut line, &mut bytes);
        }
    }
    if !line.is_empty() {
        for _ in 0..16 - bytes.len() {
            line += "   ";
        }
        display_line(&mut line, &mut bytes);
    }
}

fn main() {
    use std::env;

    // Fedora command to get nameserver IP address:
    //     nmcli dev show | grep 'IP4.DNS'
    let args: Vec<_> = env::args().collect();
    let resolver_ip = args[1].clone();
    let hostname = args[2].clone();

    // Resolve a hostname to an IP address:
    // 1. Connect to nameserver
    // 2. Build a type A request
    // 3. Send the request to the server
    // 4. Receive and parse the response
    println!("Resolving IP address of {hostname}...");
    let mut ns_sock = TcpStream::connect((resolver_ip.as_str(), 53)).unwrap();
    let req = build_request(&hostname);
    ns_sock.write(&req).unwrap();
    let ip_addr = parse_response(&mut ns_sock);

    println!("");
    println!("IP address for {hostname} is {ip_addr}");
}
