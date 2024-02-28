use core::net::SocketAddr;
use std::io;

use quinn::{ClientConfig, Endpoint, VarInt};

use crate::transport;

use super::Stream;

/// Connection via QUIC protocol.
pub struct Connection(quinn::Connection);

impl transport::Connection for Connection {
    async fn close(self) -> io::Result<()> {
        self.0.close(VarInt::from_u32(0), b"Client is closed");
        Ok(())
    }
}

impl transport::ServerConnection for Connection {
    type Stream = Stream;

    async fn accept_stream(&mut self) -> io::Result<Self::Stream> {
        Ok(self.0.accept_bi().await?.into())
    }
}

impl transport::ClientConnection for Connection {
    type Stream = Stream;

    async fn new_stream(&mut self) -> io::Result<Self::Stream> {
        Ok(self.0.open_bi().await?.into())
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
