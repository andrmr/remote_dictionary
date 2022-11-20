use std::{net::SocketAddr, str::FromStr, sync::Arc};
use clap::Parser;
use tokio::{net::TcpListener, sync::mpsc::{unbounded_channel, UnboundedSender}};

use common::dto::{Request, Response};
use server::Db;

const OK_GET: u8 = 0;
const BAD_GET: u8 = 1;


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

    let dict = Arc::new(Db::<String, String>::open_or_create("dict".to_owned())?);
    let stats = Arc::new(Db::<u8, u64>::open_or_create("stats".to_owned())?);

    let listener = TcpListener::bind(address).await?;
    println!("Listening on {:?}", listener.local_addr());

    // task req: count get reqs: total, ok, nok
    // on every get broadcast true or false, depending on the result
    let stats_producer = make_stats_handler(stats.clone());
    
    loop {
        let (socket, address) = listener.accept().await?;
        println!("Accepted client with address {:?}", address);

        let mut connection = common::net::Connection::from_socket(socket);
        let dict = dict.clone();
        let s = stats.clone();
        let stats_producer = stats_producer.clone();

        tokio::spawn(async move {
            while let Ok(Some(msg)) = connection.read().await {
                if let Ok(req) = serde_json::from_value::<Request>(msg) {
                    println!("Processing request {:?}", req);
                    let res = match req {
                        Request::Get { key } => {
                            match dict.get(&key).await {
                                Ok(Some(val)) => {
                                    _ = stats_producer.send(true)
                                        .map_err(|e| eprintln!("Unable to upload stats. Error: {}", e));
                                    
                                        Response::Get { ok: true, val: Some(val), err: None }
                                },
                                Ok(None) => {
                                    _ = stats_producer.send(false)
                                        .map_err(|e| eprintln!("Unable to upload stats. Error: {}", e));

                                    Response::Get { ok: false, val: None, err: Some(String::from("not found")) }
                                },
                                Err(e) => {
                                    _ = stats_producer.send(false)
                                        .map_err(|e| eprintln!("Unable to upload stats. Error: {}", e));

                                    Response::Get { ok: false, val: None, err: Some(e.to_string()) }
                                }
                            }
                        },
                        Request::Set { key, val } => {
                            match dict.set(&key, &val).await {
                                Ok(()) => Response::Set { ok: true, err: None },
                                Err(e) => Response::Set { ok: false, err: Some(e.to_string()) }
                            }
                        },
                        Request::Stats => {
                            let mut response = Response::Stats { ok: false, total: None, good: None, bad: None };

                            // NOTE: this is actually a bug in an async environment
                            // the stats DB needs a locking/transaction mechanism
                            // to retrieve both values in a single lock
                            if let Ok(Some(good_count)) = s.get(&OK_GET).await {
                                if let Ok(Some(bad_count)) = s.get(&BAD_GET).await {
                                    response = Response::Stats { ok: true, total: Some(good_count + bad_count), good: Some(good_count), bad: Some(bad_count) };
                                }
                            };

                            response
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

fn make_stats_handler(s: Arc<Db<u8, u64>>) -> UnboundedSender<bool> {
    let (stats_producer, mut stats_recorder) = unbounded_channel::<bool>();

    // let s = stats.clone();
    tokio::spawn(async move {
        while let Some(stat) = stats_recorder.recv().await {
            match stat {
                // NOTE: this is a bug
                // needs locking/transaction       
                true => {
                    println!("Counting a succesful Get");
                    match s.get(&OK_GET).await {
                        Ok(count) => {
                            println!("Current count {:?}", count);
                            let count = count.map_or(1, |c| c + 1);
                            _ = s
                                .set(&OK_GET, &count)
                                .await
                                .map_err(|e| eprintln!("Unable to store stat update. Error: {}", e));
                        },
                        Err(e) => println!("Db get err {:?}", e)
                    }
                },
                false => {
                    println!("Counting a failed Get");
                    if let Ok(count) = s.get(&BAD_GET).await {
                        let count = count.map_or(1, |c| c + 1);
                        _ = s
                            .set(&BAD_GET, &count)
                            .await
                            .map_err(|e| eprintln!("Unable to store stat update. Error: {}", e));
                    }
                }
            }
        }
    });

    stats_producer
}
