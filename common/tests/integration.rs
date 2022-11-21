use std::{net::SocketAddr, str::FromStr};
use serde_json::json;
use tokio::net::TcpListener;

use common::net::Connection;

#[tokio::test]
async fn test_write_read() {
    let address = "127.0.0.1:8123";

    let val = json!({"test": true});
    tokio::spawn(async move {
        let listener = TcpListener::bind(address)
            .await
            .expect("Unable to start test listener");
        
        let (socket, _) = listener.accept()
            .await
            .expect("Unable to accept client");

        let mut s = Connection::from_socket(socket);
        
        match s.read().await {
            Ok(Some(req)) => assert!(req == val),
            Ok(None) => panic!("Empty request"),
            Err(e) => panic!("Error on read {:?}", e),
        }
    });

    let val = json!({"test": true});
    let mut c = Connection::from_address(SocketAddr::from_str(&address).unwrap())
        .await
        .expect("Unable to open test socket");

    c.write(val)
        .await
        .expect("Failed to write through client socket");
}
