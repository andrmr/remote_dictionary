use std::{net::SocketAddr, str::FromStr};
use serde::{Serialize, Deserialize};
use tokio::net::TcpListener;

use common::net::{JsonConnection, BincodeConnection, Listener, Requester};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct TestThing {
    pub test_bool: bool,
    pub test_string: String,
}

#[tokio::test]
async fn test_json_write_read() {
    let address = "127.0.0.1:8128";

    let response = TestThing { test_bool: false, test_string: "test_res".to_owned() };
    let request = TestThing { test_bool: true, test_string: "test_req".to_owned() };

    let res = response.clone();
    let req = request.clone();
    tokio::spawn(async move {
        let listener = TcpListener::bind(address)
            .await
            .expect("failed bind");
        
        let (socket, _) = listener.accept()
            .await
            .expect("failed accept");

        let mut s = JsonConnection::from_socket(socket);

        let r = s.listen::<TestThing>()
            .await
            .expect("no request")
            .expect("empty request");

        assert_eq!(req, r, "bad request");
        
        s.respond(res)
            .await
            .expect("failed response")
    });

    let mut c = JsonConnection::from_address(SocketAddr::from_str(&address).unwrap())
        .await
        .expect("socket failure");
    
    let res = response.clone();
    let req = request.clone();
    let r = c.request::<TestThing, TestThing>(req)
        .await
        .expect("failed request")
        .expect("empty response");

    assert_eq!(r, res, "bad response");
}

#[tokio::test]
async fn test_bincode_write_read() {
    let address = "127.0.0.1:8129";

    let response = TestThing { test_bool: false, test_string: "test_res".to_owned() };
    let request = TestThing { test_bool: true, test_string: "test_req".to_owned() };

    let res = response.clone();
    let req = request.clone();
    tokio::spawn(async move {
        let listener = TcpListener::bind(address)
            .await
            .expect("failed bind");
        
        let (socket, _) = listener.accept()
            .await
            .expect("failed accept");

        let mut s = BincodeConnection::from_socket(socket);

        let r = s.listen::<TestThing>()
            .await
            .expect("no request")
            .expect("empty request");

        assert_eq!(req, r, "bad request");
        
        s.respond(res)
            .await
            .expect("failed response")
    });

    let mut c = BincodeConnection::from_address(SocketAddr::from_str(&address).unwrap())
        .await
        .expect("socket failure");
    
    let res = response.clone();
    let req = request.clone();
    let r = c.request::<TestThing, TestThing>(req)
        .await
        .expect("failed request")
        .expect("empty response");

    assert_eq!(r, res, "bad response");
}
