use crate::message;
use std::net::{Ipv4Addr, SocketAddrV4, UdpSocket};

const UDP_PORT: u16 = 53;

pub fn send_message_udp(msg: &message::Message) -> anyhow::Result<()> {
    // ! Eventually implement a check for the message size being <= 512 bytes.
    // ! If the size is > 512 bytes, truncate the message and set the
    // ! TC bit in the header.

    // ! Need to think about the easiest/simplest way to send a single query and receive a single response.
    // ! Need to think about the easiest/simplest way to determine the IP address of the nameserver.
    let sock = UdpSocket::bind(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 0))?;
    Ok(())
}
