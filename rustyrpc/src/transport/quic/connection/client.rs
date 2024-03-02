use alloc::sync::Arc;
use core::net::SocketAddr;
use std::io;

use quinn::{ClientConfig, Endpoint, VarInt};

use crate::transport;

use super::stream_pool::{PooledStream, StreamPool};

/// Connection via QUIC protocol used on client side.
pub struct ClientConnection {
    stream_pool: Arc<StreamPool>,
    connection: quinn::Connection,
}

impl ClientConnection {
    /// Creates new `ClientConnection` but instead of initiating connection just wraps specified connection from `quinn`
    #[must_use]
    pub fn new(connection: quinn::Connection, stream_pool_size: usize) -> Self {
        Self {
            stream_pool: StreamPool::new(connection.clone(), stream_pool_size).into(),
            connection,
        }
    }

    /// Establishes connection to server via QUIC protocol.
    ///
    /// # Errors
    /// Returns error on fail of connection establishment.
    pub async fn connect(
        client_config: ClientConfig,
        local_address: SocketAddr,
        address: SocketAddr,
        server_name: &str,
        stream_pool_size: usize,
    ) -> io::Result<Self> {
        let mut endpoint = Endpoint::client(local_address)?;
        endpoint.set_default_client_config(client_config);

        let connection = endpoint
            .connect(address, server_name)
            .map_err(|err| io::Error::new(io::ErrorKind::Other, err))?
            .await?;

        Ok(Self::new(connection, stream_pool_size))
    }
}

impl transport::Connection for ClientConnection {
    async fn close(self) -> io::Result<()> {
        self.connection.close(VarInt::from_u32(0), b"");
        Ok(())
    }
}

impl transport::ClientConnection for ClientConnection {
    type Stream = PooledStream;

    async fn new_stream(&mut self) -> io::Result<Self::Stream> {
        Ok(self.stream_pool.get().await?)
    }
}
