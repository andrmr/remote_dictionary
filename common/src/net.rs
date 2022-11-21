use futures::{TryStreamExt, SinkExt};
use serde_json::Value;
use tokio::net::TcpStream;
use tokio_serde::formats::{SymmetricalJson, Json};
use tokio_util::codec as tu_codec;
use tokio_serde as ts;

// Wrapper over a framed JSON stream
// TODO: make this generic and allow other encoding schemas
pub struct Connection {
    stream: ts::Framed<tu_codec::Framed<TcpStream, tu_codec::LengthDelimitedCodec>, Value, Value, Json<Value, Value>>
}

pub type ConnectionResult<T> = anyhow::Result<T>;

impl Connection {
    pub fn from_socket(socket: TcpStream) -> Self {
        let length_delimited = tu_codec::Framed::new(socket, tu_codec::LengthDelimitedCodec::new());
        let stream = ts::SymmetricallyFramed::new(length_delimited, SymmetricalJson::default());

        Self {
            stream
        }
    }

    pub async fn from_address(address: std::net::SocketAddr) -> ConnectionResult<Self> {
        let socket = TcpStream::connect(address).await?;
        Ok(Self::from_socket(socket))
    }

    pub async fn read(&mut self) -> ConnectionResult<Option<Value>> {
        Ok(self.stream.try_next().await?)
    }

    pub async fn write(&mut self, val: Value) -> ConnectionResult<()> {
        Ok(self.stream.send(val).await?)
    }
}
