use core::net::SocketAddr;
use std::io;

use quinn::{Endpoint, ServerConfig};

use super::connection::ServerConnection;

/// Listener for incoming connections via QUIC protocol.
pub struct ConnectionListener(quinn::Endpoint);

impl crate::transport::ConnectionListener for ConnectionListener {
    type Connection = ServerConnection;

    async fn accept_connection(&mut self) -> io::Result<Self::Connection> {
        Ok(self
            .0
            .accept()
            .await
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotConnected, "Endpoint is closed"))?
            .await
            .map(Into::into)?)
    }
}

impl ConnectionListener {
    /// Creates new listener from [`ServerConfig`] and [`SocketAddr`]
    ///
    /// # Errors
    /// Returns if connection was failed to be accepted.
    pub fn new(server_config: ServerConfig, addr: SocketAddr) -> Result<Self, std::io::Error> {
        Ok(Self(Endpoint::server(server_config, addr)?))
    }
}
