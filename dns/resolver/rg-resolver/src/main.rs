use client::Client;
use std::net::{Ipv4Addr, SocketAddrV4};
use tokio::net::{TcpListener, TcpStream};
use tracing::{error, info};
use tracing_subscriber;

mod client;
mod message;
mod name;
mod net;
mod process;
mod rr;

// Example run: RUST_LOG=info cargo run.
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    const PORT: u16 = 6789;
    info!("Listening for clients on TCP port {PORT}...");
    let listener = TcpListener::bind(SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, PORT)).await?;
    loop {
        let (stream, _) = listener.accept().await?;
        tokio::spawn(async move {
            if let Err(e) = handle_client(stream).await {
                error!("failed to handle client: {e}");
            }
        });
    }

    async fn handle_client(stream: TcpStream) -> anyhow::Result<()> {
        let mut client = Client::new(stream).await?;
        info!("Accepted new client: [{}]", client.name());
        while let Some(domain_name) = client.next_request().await? {
            // TODO: Each request needs a unique ID in the request from the client.
            println!("Processing request for {domain_name}");
        }
    }

    #[allow(unreachable_code)]
    anyhow::Result::<()>::Ok(())
}
