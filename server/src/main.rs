use std::{net::SocketAddr, str::FromStr, sync::{Arc, Mutex}};
use clap::Parser;
use tokio::{net::TcpListener};

use common::dto::{Request, Response};

mod db;
use db::*;

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
    
    let db_path = std::path::Path::new("./storage.db");
    
    let db = if db_path.exists() {
        println!("Opening storage");
        Db::open(db_path.to_str().unwrap())?
    } else {
        println!("Creating storage");
        Db::create(db_path.to_str().unwrap())?
    };

    let db = Arc::new(db);

    let listener = TcpListener::bind(address).await?;
    println!("Listening on {:?}", listener.local_addr());

    loop {
        let (socket, address) = listener.accept().await?;
        println!("Accepted client with address {:?}", address);

        let mut connection = common::net::Connection::from_socket(socket);
        let db = db.clone();

        tokio::spawn(async move {
            while let Ok(Some(msg)) = connection.read().await {
                if let Ok(req) = serde_json::from_value::<Request>(msg) {
                    println!("Processing request {:?}", req);
                    let res = match req {
                        Request::Get { key } => {
                            match db.get(&key) {
                                Ok(Some(val)) => Response::Get { ok: true, val: Some(val), err: None },
                                Ok(None) => Response::Get { ok: false, val: None, err: Some(String::from("not found")) },
                                Err(e) => Response::Get { ok: false, val: None, err: Some(e.to_string()) }
                            }
                        },
                        Request::Set { key, val } => {
                            match db.set(&key, &val) {
                                Ok(()) => Response::Set { ok: true, err: None },
                                Err(e) => Response::Set { ok: false, err: Some(e.to_string()) }
                            }
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
