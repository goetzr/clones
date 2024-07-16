use crate::message::Message;
use std::net::{Ipv4Addr, SocketAddrV4, UdpSocket};

const UDP_PORT: u16 = 53;

pub fn tx_then_rx_udp(msg: &Message) -> anyhow::Result<Message> {
    // ! Eventually implement a check for the message size being <= 512 bytes.
    // ! If the size is > 512 bytes, truncate the message and set the
    // ! TC bit in the header.

    let sock = UdpSocket::bind(SocketAddrV4::new(Ipv4Addr::LOCALHOST, UDP_PORT))?;
    sock.connect(get_nameserver_addr()?)?;
    let _ = sock.send(msg.serialize()?.as_slice())?;
    let mut buf = Vec::new();
    let _ = sock.recv(buf.as_mut_slice())?;
    let mut buf = &buf[..];
    Ok(Message::parse(&mut buf)?)
}

fn get_nameserver_addr() -> anyhow::Result<SocketAddrV4> {
    // TODO: Need to run a command or something to determine this dynamically.
    // TODO: I ran scutil --dns
    Ok(SocketAddrV4::new("192.168.50.1".parse()?, UDP_PORT))
}
