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
            Err(e) => return Err(anyhow::anyhow!(e)),
        };
        let mut buf = vec![0; len];
        self.stream.read_exact(&mut buf[..]).await?;
        let domain_name = String::from_utf8(buf)?;
        Ok(Some(domain_name))
    }
}
