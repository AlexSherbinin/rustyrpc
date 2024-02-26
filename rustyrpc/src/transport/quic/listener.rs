use core::net::SocketAddr;
use std::io;

use quinn::{Endpoint, ServerConfig};

use super::Connection;

/// Listener for incoming connections via QUIC protocol.
pub struct ConnectionListener(quinn::Endpoint);

impl crate::transport::ConnectionListener for ConnectionListener {
    type Connection = Connection;

    async fn accept_connection(&mut self) -> io::Result<Self::Connection> {
        Ok(self
            .0
            .accept()
            .await
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotConnected, "Endpoint is closed"))?
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
