use std::net::{Ipv4Addr, SocketAddrV4};
use tokio::net::TcpListener;
use tokio::signal::ctrl_c;

pub type Error = Box<dyn std::error::Error + Sync + Send + 'static>;
pub type Result<T> = std::result::Result<T, Error>;

pub async fn run() -> Result<()> {
    let config = parse_command_line();
    let bind_addr = SocketAddrV4::new(Ipv4Addr::LOCALHOST, config.port);
    let listener = TcpListener::bind(bind_addr).await?;

    tokio::select! {
        val = listener.accept() => {
            
        }
    }

    Ok(())
}

pub struct Config {
    pub port: u16,
}

pub fn parse_command_line() -> Config {
    let matches = clap::command!()
        .arg(
            clap::Arg::new("port")
                .short('p')
                .long("port")
                .value_parser(clap::value_parser!(u16))
                .default_value("17553")
                .value_name("PORT")
                .help("The TCP port number to listen on for client connections."),
        )
        .get_matches();
    
    let &port = matches.get_one("port").unwrap();
    Config { port }
}