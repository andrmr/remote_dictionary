use std::{net::SocketAddr, str::FromStr};
use clap::{Parser, Subcommand};

use client::Client;

#[derive(Parser)]
#[command(about="Dictionary client, used to call dictionary server endpoints", long_about=None)]
#[command(author, version, propagate_version=true)]
struct Cli {
    #[arg(short, long, value_name="ADDRESS")]
    #[arg(help="Server address; alternatively, set the DICT_ADDR env var")]
    address: Option<String>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    #[command(about = "Get value")]
    Get {
        #[arg(short, long, value_name="KEY")]
        #[arg(help="Key")]
        key: String
    },

    #[command(about = "Set key/value pair")]
    Set {
        #[arg(short, long, value_name="KEY")]
        #[arg(help="Key")]
        key: String,

        #[arg(short, long, value_name="VAL")]
        #[arg(help="Value")]
        val: String,
    },

    #[command(about = "Get stats")]
    Stats,
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

    let mut client = Client::new(address);
    client.connect().await?;
    
    let req = match &cli.command {
        Commands::Get { key } => common::dto::Request::Get { key: key.clone() },
        Commands::Set { key, val } => common::dto::Request::Set { key: key.clone(), val: val.clone() },
        Commands::Stats => common::dto::Request::Stats,
    };

    match client.send_request(req).await {
        Ok(res) => println!("Received response {:?}", res),
        Err(e) => eprintln!("Error {:?}", e),
    }

    Ok(())
}
