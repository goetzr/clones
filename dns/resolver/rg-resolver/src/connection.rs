use anyhow::Context;
use bytes::Buf;
use tokio::io::AsyncReadExt;
use tokio::io::ErrorKind;
use tokio::net::TcpStream;

pub struct Connection {
    stream: TcpStream,
}

impl Connection {
    pub fn new(stream: TcpStream) -> Self {
        Self { stream }
    }

    pub async fn next_request(&mut self) -> anyhow::Result<Option<String>> {
        let len = match self.stream.read_u8().await {
            Ok(n) => n as usize,
            Err(e) if e.kind() == ErrorKind::UnexpectedEof => return Ok(None),
            Err(e) => {
                return Err(anyhow::anyhow!(e)).with_context(|| "failed to receive request length")
            }
        };
        let mut buf = vec![0; len];
        if let Err(e) = self.stream.read_exact(&mut buf[..]).await {
            if e.kind() == ErrorKind::UnexpectedEof {
                anyhow::bail!("failed to read the next request: not enough data sent");
            }
            return Err(anyhow::anyhow!(e)).with_context(|| "failed to read the next request");
        }
        let mut buf = &buf[..];
        let id = buf.get_u8();
        let domain_name = String::from_utf8(&buf[..])?;
        Ok(Some(domain_name))
    }
}

pub struct ClientRequest {
    id: u32,
    name: String,
}

impl ClientRequest {
    pub fn new(name: String) -> Self {}
}
