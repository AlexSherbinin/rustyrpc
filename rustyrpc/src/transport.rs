use crate::format::{Decode, Encode, EncodingFormat};
use core::future::Future;
use extension_traits::extension;
use std::io;

/// Provides transport implementation via QUIC protocol.
pub mod quic;

/// Transport specific connection's stream.
pub trait Stream: Send {
    /// Send a message on the stream.
    fn send(&mut self, message: Vec<u8>) -> impl Future<Output = io::Result<()>> + Send;
    /// Receive a message from stream.
    fn receive(&mut self) -> impl Future<Output = io::Result<Vec<u8>>> + Send;
    /// Wait until stream is closed by other side of connection.
    fn stopped(self) -> impl Future<Output = io::Result<()>> + Send;
    /// Close stream.
    fn close(self) -> impl Future<Output = io::Result<()>> + Send;
}

#[extension(pub(crate) trait StreamExt)]
impl<T: Stream> T {
    async fn send_encodable<M: Encode<Format>, Format: EncodingFormat>(
        &mut self,
        message: &M,
    ) -> io::Result<()> {
        let encoded = message
            .encode()
            .map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidData, err))?;
        self.send(encoded).await
    }

    async fn receive_decodable<M: Decode<Format>, Format: EncodingFormat>(
        &mut self,
    ) -> io::Result<M> {
        let message = self.receive().await?;
        M::decode(&message).map_err(|err| std::io::Error::new(std::io::ErrorKind::Other, err))
    }
}

/// Transport specific incoming connection.
pub trait Connection: Send + 'static {
    /// Stream produced by connection.
    type Stream: Stream + 'static;

    /// Create new stream and notify other side of connection about it.
    fn new_stream(&mut self) -> impl Future<Output = io::Result<Self::Stream>> + Send;
    /// Accept new stream created by other side of connection.
    fn accept_stream(&mut self) -> impl Future<Output = io::Result<Self::Stream>> + Send;
    /// Close connection.
    fn close(self) -> impl Future<Output = io::Result<()>> + Send;
}

/// Transport specific incoming connections listener like a [`TcpListener`][`std::net::TcpListener`] or others
pub trait ConnectionListener: Send {
    /// Connection produced by listener
    type Connection: Connection;

    /// Accepts a new connection
    fn accept_connection(&mut self) -> impl Future<Output = io::Result<Self::Connection>>;
}
