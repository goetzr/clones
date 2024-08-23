use anyhow::Context;
use bytes::Buf;
use tokio::io::AsyncReadExt;
use tokio::io::BufReader;
use tokio::io::ErrorKind;
use tokio::net::TcpStream;

pub struct Client {
    reader: BufReader<TcpStream>,
    name: String,
}

impl Client {
    pub async fn new(stream: TcpStream) -> anyhow::Result<Self> {
        let mut reader = BufReader::new(stream);
        let name = Client::recv_name(&mut reader).await?;
        Ok(Client { reader, name })
    }

    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    async fn recv_name(reader: &mut BufReader<TcpStream>) -> anyhow::Result<String> {
        let len = match Client::recv_length_byte_impl(reader).await {
            Ok(Some(len)) => len,
            Ok(None) => {
                return Err(anyhow::anyhow!(
                    "connection closed before client name could be received"
                ))
            }
            Err(e) => {
                return Err(anyhow::anyhow!(e))
                    .with_context(|| "failed to receive the length of the client name")
            }
        };

        let mut buf = vec![0_u8; len];
        reader
            .read_exact(buf.as_mut_slice())
            .await
            .with_context(|| "failed to receive the client name")?;

        let name = String::from_utf8(buf)
            .with_context(|| "received client name contains invalid UTF-8")?;
        Ok(name)
    }

    pub async fn next_request(&mut self) -> anyhow::Result<Option<ClientRequest>> {
        let Some(len) = self.recv_length_byte().await? else {
            // Client closed his end of the connection, indicating no more requests will be sent.
            return Ok(None);
        };
        let mut buf = vec![0; len];
        self.reader
            .read_exact(buf.as_mut_slice())
            .await
            .with_context(|| "failed to receive the next request: failed to receive the payload")?;
        let mut buf = &buf[..];

        let id = buf.get_u8();
        // The remainder of the payload contains the domain name.
        let name = String::from_utf8(buf[..].to_vec())?;

        Ok(Some(ClientRequest::new(id, name)))
    }

    async fn recv_length_byte(&mut self) -> anyhow::Result<Option<usize>> {
        Client::recv_length_byte_impl(&mut self.reader).await
    }

    async fn recv_length_byte_impl(
        reader: &mut BufReader<TcpStream>,
    ) -> anyhow::Result<Option<usize>> {
        match reader.read_u8().await {
            Ok(len) => Ok(Some(len as usize)),
            Err(e) if e.kind() == ErrorKind::UnexpectedEof => Ok(None),
            Err(e) => Err(anyhow::anyhow!(e)),
        }
    }
}

pub struct ClientRequest {
    id: u8,
    name: String,
}

impl ClientRequest {
    pub fn new(id: u8, name: String) -> Self {
        Self { id, name }
    }

    pub fn id(&self) -> u8 {
        self.id
    }

    pub fn name(&self) -> &str {
        self.name.as_str()
    }
}
