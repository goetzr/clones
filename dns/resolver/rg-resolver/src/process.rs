use crate::message::Message;
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
        // 3. Handle CNAME.
        // 4. Handle NS.
        self.sock.send(query.serialize()?.as_slice())?;
        let mut resp_buf = [0; 512];
        self.sock.recv(&mut resp_buf)?;
        let response = Message::parse(&resp_buf[..])?;

        Ok(())
    }
}
