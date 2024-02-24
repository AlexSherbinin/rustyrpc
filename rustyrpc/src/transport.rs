use crate::{
    error::{ReceiveDecodableError, SendEncodableError},
    format::{Decode, Encode, EncodingFormat},
};
use core::future::Future;
use extension_traits::extension;

/// Provides transport implementation via QUIC protocol.
pub mod quic;

/// Transport specific connection's stream.
pub trait Stream: Send {
    /// Error that can occur while any operation on connection.
    type Error: std::error::Error + 'static;

    /// Send a message on the stream.
    fn send(&mut self, message: Vec<u8>) -> impl Future<Output = Result<(), Self::Error>> + Send;
    /// Receive a message from stream.
    fn receive(&mut self) -> impl Future<Output = Result<Vec<u8>, Self::Error>> + Send;
    /// Wait until stream is closed by other side of connection.
    fn stopped(self) -> impl Future<Output = Result<(), Self::Error>> + Send;
    /// Close stream.
    fn close(self) -> impl Future<Output = Result<(), Self::Error>> + Send;
}

#[extension(pub(crate) trait StreamExt)]
impl<T: Stream> T {
    async fn send_encodable<M: Encode<Format>, Format: EncodingFormat>(
        &mut self,
        message: &M,
    ) -> Result<(), SendEncodableError<T, M::Error>> {
        let encoded = message.encode().map_err(SendEncodableError::Encode)?;
        self.send(encoded).await.map_err(SendEncodableError::Send)
    }

    async fn receive_decodable<M: Decode<Format>, Format: EncodingFormat>(
        &mut self,
    ) -> Result<M, ReceiveDecodableError<T, M::Error>> {
        let message = self
            .receive()
            .await
            .map_err(ReceiveDecodableError::Receive)?;
        M::decode(&message).map_err(ReceiveDecodableError::Decode)
    }
}

/// Transport specific incoming connection.
pub trait Connection: Send {
    /// Stream produced by connection.
    type Stream: Stream + 'static;
    /// Error that can occur while any operation on connection.
    type Error: std::error::Error + 'static;

    /// Create new stream and notify other side of connection about it.
    fn new_stream(&mut self) -> impl Future<Output = Result<Self::Stream, Self::Error>> + Send;
    /// Accept new stream created by other side of connection.
    fn accept_stream(&mut self) -> impl Future<Output = Result<Self::Stream, Self::Error>> + Send;
    /// Close connection.
    fn close(self) -> impl Future<Output = Result<(), Self::Error>> + Send;
}

/// Transport specific incoming connections listener like a [`TcpListener`][`std::net::TcpListener`] or others
pub trait ConnectionListener: Send {
    /// Connection produced by listener
    type Connection: Connection;
    /// Error that can occur while accepting incoming connection.
    type Error: std::error::Error + 'static;

    /// Accepts a new connection
    fn accept_connection(&mut self) -> impl Future<Output = Result<Self::Connection, Self::Error>>;
}
