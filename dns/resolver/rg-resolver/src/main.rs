use connection::Connection;
use std::net::{Ipv4Addr, SocketAddrV4};
use tokio::net::TcpListener;
use tracing::info;
use tracing_subscriber;

mod connection;
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
        let mut connection = Connection::new(listener.accept().await?.0);
        info!("Accepted new client");
        tokio::spawn(async move {
            while let Some(domain_name) = connection.next_request().await? {
                println!("Processing request for {domain_name}");
            }

            anyhow::Result::<()>::Ok(())
        });
    }

    #[allow(unreachable_code)]
    anyhow::Result::<()>::Ok(())
}
