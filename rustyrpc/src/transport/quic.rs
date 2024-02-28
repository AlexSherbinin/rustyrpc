mod listener;

use core::{net::SocketAddr, num::TryFromIntError};
use quinn::{ClientConfig, Endpoint, RecvStream, SendStream, StoppedError, VarInt};
use std::io;
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufReader, BufWriter};

pub use listener::ConnectionListener;
use thiserror::Error;

/// Connection via QUIC protocol.
pub struct Connection(quinn::Connection);

/// Stream via QUIC protocol.
pub struct Stream {
    send_stream: BufWriter<SendStream>,
    receive_stream: BufReader<RecvStream>,
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
    async fn send(&mut self, message: Vec<u8>) -> io::Result<()> {
        let length_prefix: u32 = message.len().try_into().map_err(|err| {
            std::io::Error::new(
                std::io::ErrorKind::Other,
                SendingInvalidLengthPrefixError(err),
            )
        })?;
        let length_prefix = length_prefix.to_be_bytes();

        self.send_stream.write_all(&length_prefix).await?;
        self.send_stream.write_all(&message).await?;
        Ok(())
    }

    async fn receive(&mut self) -> io::Result<Vec<u8>> {
        let mut length_prefix_buffer = [0u8; 4];
        self.receive_stream
            .read_exact(&mut length_prefix_buffer)
            .await?;
        let length_prefix: usize = u32::from_be_bytes(length_prefix_buffer)
            .try_into()
            .map_err(|err| {
                std::io::Error::new(
                    std::io::ErrorKind::Other,
                    InvalidLengthPrefixReceivedError(err),
                )
            })?;

        let mut message_buffer = vec![0u8; length_prefix];
        self.receive_stream.read_exact(&mut message_buffer).await?;

        Ok(message_buffer)
    }

    async fn flush(&mut self) -> io::Result<()> {
        self.send_stream.flush().await
    }

    async fn stopped(self) -> io::Result<()> {
        if let Err(err) = self.send_stream.into_inner().stopped().await {
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

    async fn close(mut self) -> io::Result<()> {
        self.send_stream.shutdown().await
    }
}

impl From<(SendStream, RecvStream)> for Stream {
    fn from((send_stream, receive_stream): (SendStream, RecvStream)) -> Self {
        Self {
            send_stream: BufWriter::new(send_stream),
            receive_stream: BufReader::new(receive_stream),
        }
    }
}

impl super::Connection for Connection {
    type Stream = Stream;

    async fn new_stream(&mut self) -> io::Result<Self::Stream> {
        Ok(self.0.open_bi().await?.into())
    }

    async fn accept_stream(&mut self) -> io::Result<Self::Stream> {
        Ok(self.0.accept_bi().await?.into())
    }

    async fn close(self) -> io::Result<()> {
        self.0.close(VarInt::from_u32(0), b"Client is closed");
        Ok(())
    }
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
    ) -> io::Result<Self> {
        let mut endpoint = Endpoint::client(local_address)?;
        endpoint.set_default_client_config(client_config);

        let connection = endpoint
            .connect(address, server_name)
            .map_err(|err| io::Error::new(io::ErrorKind::Other, err))?
            .await?;

        Ok(connection.into())
    }
}

impl From<quinn::Connection> for Connection {
    fn from(connection: quinn::Connection) -> Self {
        Self(connection)
    }
}
