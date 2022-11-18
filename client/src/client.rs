use std::net::SocketAddr;

use common::dto::{Request, Response};
use common::net::Connection;

pub type ClientResult<T> = anyhow::Result<T>;

pub struct Client {
    address: SocketAddr,
    connection: Option<Connection>,
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

    pub async fn send_request(&mut self, request: Request) -> ClientResult<Response> {
        if self.connection.is_none() {
            self.connect().await?;
        }

        self.connection.as_mut().unwrap()
            .write(serde_json::to_value(request)?)
            .await?;
        
        let response = self.connection.as_mut().unwrap()
            .read()
            .await?
            .unwrap_or_default();

        Ok(serde_json::from_value::<Response>(response)?)
    }
}
