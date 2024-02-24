use core::net::SocketAddr;

use quinn::{ConnectionError, Endpoint, ServerConfig};
use thiserror::Error;

use super::Connection;

#[derive(Error, Debug)]
pub enum AcceptError {
    #[error("Endpoint is closed")]
    EndpointIsClosed,
    #[error(transparent)]
    Connection(#[from] ConnectionError),
}

/// Listener for incoming connections via QUIC protocol.
pub struct ConnectionListener(quinn::Endpoint);

impl crate::transport::ConnectionListener for ConnectionListener {
    type Connection = Connection;
    type Error = AcceptError;

    async fn accept_connection(&mut self) -> Result<Self::Connection, Self::Error> {
        Ok(self
            .0
            .accept()
            .await
            .ok_or(AcceptError::EndpointIsClosed)?
            .await
            .map(Connection)?)
    }
}

impl ConnectionListener {
    /// Creates new listener from [`ServerConfig`] and [`SocketAddr`]
    ///
    /// # Errors
    /// Returns error by many reasons represented in [`ConnectionError`]
    pub fn new(server_config: ServerConfig, addr: SocketAddr) -> Result<Self, std::io::Error> {
        Ok(Self(Endpoint::server(server_config, addr)?))
    }
}
