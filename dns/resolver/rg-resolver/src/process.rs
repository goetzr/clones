use crate::message::{Message, Question};
use std::net::{Ipv4Addr, SocketAddrV4};
use tokio::net::UdpSocket;

struct QueryProcessor {
    sock: UdpSocket,
}

impl QueryProcessor {
    const PORT: u16 = 53;

    pub fn new() -> anyhow::Result<Self> {
        let sock = UdpSocket::bind(SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, Self::PORT))?;
        let ns_addr = SocketAddrV4::new(Ipv4Addr::new(192, 168, 1, 50), Self::PORT);
        sock.connect(ns_addr)?;
        Ok(Self { sock })
    }

    pub fn process(&self, query: Message) -> anyhow::Result<()> {
        // Responses to QCLASS = * queries can never be authoritative.
        // Responses to QTYPE = * must be authoritative.
        // Don't cache RR if TTL == 0.
        self.sock.send(query.serialize()?.as_slice())?;
        let mut resp_buf = [0; 512];
        self.sock.recv(&mut resp_buf)?;
        let response = Message::parse(&resp_buf[..])?;

        Ok(())
    }
}

struct Request {
    /// SNAME, STYPE, SCLASS.
    question: Question,
    timestamp: 
}

// TODO: Load SBELT from configuration file.
struct NameServerList {
    /// Zone name equivalent.
    /// Number of labels from the root down which SNAME has in common with the zone being queried.
    /// Used as a measure of how "close" the resolver is to SNAME.
    match_count: i32,
    name_servers: Vec<NameServer>,
}

struct NameServer {
    name: String,
    addresses: Vec<NameServerAddress>,
}

struct NameServerAddress {
    address: Ipv4Addr,
    /// Weighted average for response time.
    /// Batting average.
    history: u32,
}
