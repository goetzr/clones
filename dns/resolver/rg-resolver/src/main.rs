#[tokio::main]
async fn main() {
    if let Err(e) = rg_resolver::run().await {
        eprintln!("ERROR: {}", e);
    }
}
