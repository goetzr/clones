use std::io;
use std::net::{Ipv4Addr, SocketAddrV4};
use std::time::Duration;
use tokio::net::{TcpListener, TcpStream};
use tokio::signal;
use tokio::time::sleep;

pub type Error = Box<dyn std::error::Error + Sync + Send + 'static>;
pub type Result<T> = std::result::Result<T, Error>;

pub async fn run() -> Result<()> {
    let subscriber = tracing_subscriber::FmtSubscriber::new();
    tracing::subscriber::set_global_default(subscriber)?;

    let config = parse_command_line();
    let listener = bind_listener(config.port).await?;

    tokio::select! {
        res = async {
            loop {
                let (socket, _) = listener.accept().await?;
                tokio::spawn(async {
                    process(socket).await;
                });
            }
            #[allow(unused)]
            Ok::<_, io::Error>(())
        } => {
            res?
        }
        _ = signal::ctrl_c() => {}
    }

    Ok(())
}

async fn process(socket: TcpStream) {
    // A request is a JSON object
    // Cache responses
}

async fn bind_listener(port: u16) -> io::Result<TcpListener> {
    let bind_addr = SocketAddrV4::new(Ipv4Addr::LOCALHOST, port);
    TcpListener::bind(bind_addr).await.into()
}

struct Config {
    port: u16,
}

fn parse_command_line() -> Config {
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
