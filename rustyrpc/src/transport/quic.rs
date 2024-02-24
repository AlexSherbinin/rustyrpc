mod listener;

use core::{net::SocketAddr, num::TryFromIntError};
use futures::io::{AsyncReadExt, AsyncWriteExt};
use quinn::{
    ClientConfig, ConnectError, ConnectionError, Endpoint, RecvStream, SendStream, StoppedError,
    VarInt,
};

pub use listener::ConnectionListener;
use thiserror::Error;

/// Connection via QUIC protocol.
pub struct Connection(quinn::Connection);

/// Stream via QUIC protocol.
pub struct Stream {
    send_stream: SendStream,
    receive_stream: RecvStream,
}

/// Error that may occur when sending invalid length prefix.
#[derive(Error, Debug)]
#[error("Trying to send invalid length prefix")]
pub struct SendingInvalidLengthPrefixError(#[from] TryFromIntError);

/// Error that may occur while receiving message if length prefix is invalid
#[derive(Error, Debug)]
#[error("Invalid length prefix received")]
pub struct InvalidLengthPrefixReceivedError(#[from] TryFromIntError);

impl super::Stream for Stream {
    type Error = std::io::Error;

    async fn send(&mut self, message: Vec<u8>) -> Result<(), Self::Error> {
        let length_prefix: u32 = message.len().try_into().map_err(|err| {
            std::io::Error::new(
                std::io::ErrorKind::Other,
                SendingInvalidLengthPrefixError(err),
            )
        })?;
        let length_prefix = length_prefix.to_be_bytes();

        // Tokio's AsyncWriteExt trait conflicts with same trait in futures-io, so we need to write it in this "ugly" form.
        // Same in the receive method
        AsyncWriteExt::write_all(&mut self.send_stream, &length_prefix).await?;
        AsyncWriteExt::write_all(&mut self.send_stream, &message).await?;

        Ok(())
    }

    async fn receive(&mut self) -> Result<Vec<u8>, Self::Error> {
        let mut length_prefix_buffer = [0u8; 4];
        AsyncReadExt::read_exact(&mut self.receive_stream, &mut length_prefix_buffer).await?;
        let length_prefix: usize = u32::from_be_bytes(length_prefix_buffer)
            .try_into()
            .map_err(|err| {
                std::io::Error::new(
                    std::io::ErrorKind::Other,
                    InvalidLengthPrefixReceivedError(err),
                )
            })?;

        let mut message_buffer = vec![0u8; length_prefix];
        AsyncReadExt::read_exact(&mut self.receive_stream, &mut message_buffer).await?;

        Ok(message_buffer)
    }

    async fn stopped(mut self) -> Result<(), Self::Error> {
        if let Err(err) = self.send_stream.stopped().await {
            match err {
                StoppedError::ConnectionLost(err) => Err(err.into()),
                err @ (StoppedError::UnknownStream | StoppedError::ZeroRttRejected) => {
                    Err(std::io::Error::new(std::io::ErrorKind::Other, err))
                }
            }
        } else {
            Ok(())
        }
    }

    async fn close(mut self) -> Result<(), Self::Error> {
        self.send_stream.close().await
    }
}

impl super::Connection for Connection {
    type Stream = Stream;
    type Error = std::io::Error;

    async fn new_stream(&mut self) -> Result<Self::Stream, Self::Error> {
        let (send_stream, receive_stream) = self.0.open_bi().await?;
        Ok(Stream {
            send_stream,
            receive_stream,
        })
    }

    async fn accept_stream(&mut self) -> Result<Self::Stream, Self::Error> {
        let (send_stream, receive_stream) = self.0.accept_bi().await?;
        Ok(Stream {
            send_stream,
            receive_stream,
        })
    }

    async fn close(self) -> Result<(), Self::Error> {
        self.0.close(VarInt::from_u32(0), b"Client is closed");
        Ok(())
    }
}

/// Errors that may occur while connecting to server via QUIC protocol.
#[derive(Error, Debug)]
pub enum ConnectingError {
    /// Indicates errors related to invalid configuration and others. See [`ConnectError`].
    #[error(transparent)]
    Configuration(#[from] ConnectError),
    /// Connection establishment error like version mismatch and others.
    #[error(transparent)]
    ConnectionEstablishment(#[from] ConnectionError),
    /// Indicates IO errors.
    #[error(transparent)]
    IO(#[from] std::io::Error),
}

impl Connection {
    /// Establishes connection to server via QUIC protocol.
    ///
    /// # Errors
    /// Returns error on fail of connection establishment.
    pub async fn connect(
        client_config: ClientConfig,
        local_address: SocketAddr,
        address: SocketAddr,
        server_name: &str,
    ) -> Result<Self, ConnectingError> {
        let mut endpoint = Endpoint::client(local_address)?;
        endpoint.set_default_client_config(client_config);

        let connection = endpoint.connect(address, server_name)?.await?;

        Ok(Self(connection))
    }
}

impl From<quinn::Connection> for Connection {
    fn from(connection: quinn::Connection) -> Self {
        Self(connection)
    }
}
