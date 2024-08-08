use std::env;
use tracing::info;
use tracing_subscriber;

mod message;
mod name;
mod net;
mod rr;

// Example run: RUST_LOG=info cargo run -- yahoo.com.
fn main() {
    if let Err(e) = run() {
        eprintln!("ERROR: {e}");
        std::process::exit(1);
    }
}

fn run() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let Some(domain_name) = env::args().skip(1).next() else {
        anyhow::bail!("must specify domain name".to_string());
    };

    info!("Querying address(es) for domain name {domain_name}...");
    let query = message::address_query(&domain_name);
    info!("Sending query {:#?}", query);
    let response = net::tx_then_rx_udp(&query)?;
    info!("Got response: {:#?}", response);

    Ok(())
}
