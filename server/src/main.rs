use std::{net::SocketAddr, str::FromStr};
use clap::Parser;
use tokio::net::TcpListener;

use common::dto::{Request, Response};

#[derive(Parser)]
#[command(about="Dictionary server", long_about=None)]
#[command(author, version, propagate_version=true)]
struct Cli {
    #[arg(short, long, value_name="ADDRESS")]
    #[arg(help="Server address; alternatively, set the DICT_ADDR env var")]
    address: Option<String>,
}

#[tokio::main]
pub async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    
    let address = match cli.address {
        Some(address) => address,
        None => std::env::var("DICT_ADDR").expect("Provide an address or set the DICT_ADDR env var"),
    };
    
    let address = SocketAddr::from_str(&address)
        .expect("Unable to parse address");

    let listener = TcpListener::bind(address).await?;
    println!("Listening on {:?}", listener.local_addr());

    loop {
        let (socket, address) = listener.accept().await?;
        println!("Accepted client with address {:?}", address);

        let mut connection = common::net::Connection::from_socket(socket);

        tokio::spawn(async move {
            while let Ok(Some(msg)) = connection.read().await {
                if let Ok(req) = serde_json::from_value::<Request>(msg) {
                    println!("Processing request {:?}", req);
                    let res = match req {
                        Request::Get { key } => {
                            Response::Get { ok: true, val: Some("dummy_val".into()), err: None }
                        },
                        _ => {
                            Response::Empty
                        },
                    };

                    println!("Responding back");
                    if let Err(e) = connection.write(serde_json::to_value(res).unwrap()).await {
                        eprintln!("Response failed. Error {:?}", e)
                    }
                }
            }
        });
    }
}
