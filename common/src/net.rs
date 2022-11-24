use async_trait::async_trait;
use futures::{SinkExt, TryStreamExt};
use serde::{Serialize, de::DeserializeOwned};
use serde_json::Value;
use tokio::net::TcpStream;

use tokio::net as tn;
use tokio_serde as ts;
use tokio_util::codec as tuc;

/// Helper type alias
type FramedStream<T, C> = ts::SymmetricallyFramed<tuc::Framed<tn::TcpStream, tuc::LengthDelimitedCodec>, T, C>;

/// Result type used throughout the module
pub type ConnectionResult<T> = anyhow::Result<T>;

/// Framed stream connection
pub struct Connection<T, C> {
    stream: FramedStream<T, C>
}

/// Connection with Json codec
pub type JsonConnection = Connection<Value, tokio_serde::formats::SymmetricalJson<Value>>;

/// Connection with Bincode codec
/// See https://blog.logrocket.com/rust-serialization-whats-ready-for-production-today/
pub type BincodeConnection = Connection<Vec<u8>, tokio_serde::formats::SymmetricalBincode<Vec<u8>>>;

/// Allows a connection to send requests
#[async_trait]
pub trait Requester {

    /// Sends a request and returns a response
    async fn request<Req, Res>(&mut self, req: Req) -> ConnectionResult<Option<Res>>
    where
        Req: Send + Serialize,
        Res: DeserializeOwned;
}

/// Allows a connection to act as a listener
#[async_trait]
pub trait Listener {

    /// Sends a message without waiting for a reply
    async fn respond<Res>(&mut self, res: Res) -> ConnectionResult<()>
    where
        Res: Send + Serialize;

    /// Awaits requests and returns them
    async fn listen<Req>(&mut self) -> ConnectionResult<Option<Req>>
    where
        Req: DeserializeOwned;
}

impl<T, C> Connection<T, C>
where
    C: tokio_serde::Serializer<T> + tokio_serde::Deserializer<T>
{
    /// Creates a connection with a given TCP socket
    pub fn from_socket(socket: TcpStream) -> Self
    where C: Default
    {
        let length_delimited = tuc::Framed::new(socket, tuc::LengthDelimitedCodec::new());
        let stream = tokio_serde::SymmetricallyFramed::new(length_delimited, C::default());

        Self {
            stream,
        }
    }

    /// Creates a connection with a given IP address
    pub async fn from_address(address: std::net::SocketAddr) -> ConnectionResult<Self>
    where C: Default
    {
        let socket = TcpStream::connect(address).await?;
        Ok(Self::from_socket(socket))
    }
}

/// Requester implementation for the JSON codec
#[async_trait]
impl Requester for JsonConnection {
    async fn request<Req, Res>(&mut self, req: Req) -> ConnectionResult<Option<Res>>
    where
        Req: Send + Serialize,
        Res: DeserializeOwned,
    {
        // write
        let req = serde_json::to_value(req)?;
        self.stream.send(req).await?;

        // read
        let res = self.stream
            .try_next()
            .await?
            .and_then(|r| serde_json::from_value::<Res>(r).ok());

        Ok(res)
    }
}

/// Listener implementation for the JSON codec
#[async_trait]
impl Listener for JsonConnection {
    async fn respond<Res>(&mut self, res: Res) -> ConnectionResult<()>
    where
        Res: Send + Serialize
    {
        let res = serde_json::to_value(res)?;
        Ok(self.stream.send(res).await?)
    }

    async fn listen<Req>(&mut self) -> ConnectionResult<Option<Req>>
    where
        Req: DeserializeOwned
    {
        let req = self.stream
            .try_next()
            .await?
            .and_then(|r| serde_json::from_value::<Req>(r).ok());
        
        Ok(req)
    }
}

/// Requester implementation for the Bincode codec
#[async_trait]
impl Requester for BincodeConnection {
    async fn request<Req, Res>(&mut self, req: Req) -> ConnectionResult<Option<Res>>
    where
        Req: Send + Serialize,
        Res: DeserializeOwned,
    {
        // read
        let req = bincode::serialize(&req)?;
        self.stream.send(req).await?;

        // write
        let res = self.stream
            .try_next()
            .await?
            .and_then(|r| bincode::deserialize(&r).ok());

        Ok(res)
    }
}

/// Listener implementation for the Bincode codec
#[async_trait]
impl Listener for BincodeConnection {
    async fn respond<Res>(&mut self, res: Res) -> ConnectionResult<()>
    where
        Res: Send + Serialize
    {
        let res = bincode::serialize(&res)?;
        Ok(self.stream.send(res).await?)
    }

    async fn listen<Req>(&mut self) -> ConnectionResult<Option<Req>>
    where
        Req: DeserializeOwned
    {
        let req = self.stream
            .try_next()
            .await?
            .and_then(|r| bincode::deserialize(&r).ok());
        
        Ok(req)
    }
}
