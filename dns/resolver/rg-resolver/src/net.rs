use crate::message::Message;
use std::net::{Ipv4Addr, SocketAddrV4, UdpSocket};
use tracing::info;

const UDP_PORT: u16 = 53;

pub fn tx_then_rx_udp(msg: &Message) -> anyhow::Result<Message> {
    let sock = UdpSocket::bind(SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, 0))?;
    info!("Socket bound");
    sock.connect(get_nameserver_addr()?)?;
    info!("Socket connected");
    let _ = sock.send(msg.serialize()?.as_slice())?;
    info!("Data sent");
    let mut buf = [0_u8; 512];
    let size = sock.recv(&mut buf)?;
    info!("Received {size} byte response");
    let mut buf = &buf[..];
    Ok(Message::parse(&mut buf)?)
}

fn get_nameserver_addr() -> anyhow::Result<SocketAddrV4> {
    // TODO: Need to run a command or something to determine this dynamically.
    // TODO: I ran scutil --dns
    Ok(SocketAddrV4::new("192.168.50.1".parse()?, UDP_PORT))
}
