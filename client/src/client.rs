use std::net::SocketAddr;

use common::dto::{Request, Response};
use common::net::{Connection, BincodeConnection, Requester};

pub type ClientResult<T> = anyhow::Result<T>;

// Client library used to connect to Remote Dictionary server
// TODO: add inner mutability for the connection field, so the client object can be used immutably
pub struct Client {
    address: SocketAddr,
    connection: Option<BincodeConnection>,
}

impl Client {
    pub fn new(address: SocketAddr) -> Self {
        Self {
            address,
            connection: None,
        }
    }

    pub async fn connect(&mut self) -> ClientResult<()> {
        self.connection = Some(Connection::from_address(self.address).await?);
        Ok(())
    }

    // Sends a request to the server and waits for a response
    pub async fn send_request(&mut self, request: Request) -> ClientResult<Response> {
        if self.connection.is_none() {
            self.connect().await?;
        }

        let response = self.connection.as_mut().unwrap()
            .request(request)
            .await?
            .unwrap_or_default();

        Ok(response)
    }
}
