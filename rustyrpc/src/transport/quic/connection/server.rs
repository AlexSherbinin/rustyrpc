use std::io;

use quinn::VarInt;

use crate::transport::{self, quic::stream::Stream};

/// Connection via QUIC protocol used on server side.
pub struct ServerConnection(quinn::Connection);

impl transport::Connection for ServerConnection {
    async fn close(self) -> io::Result<()> {
        self.0.close(VarInt::from_u32(0), b"");
        Ok(())
    }
}

impl transport::ServerConnection for ServerConnection {
    type Stream = Stream;

    async fn accept_stream(&mut self) -> io::Result<Self::Stream> {
        Ok(self.0.accept_bi().await?.into())
    }
}

impl From<quinn::Connection> for ServerConnection {
    fn from(connection: quinn::Connection) -> Self {
        Self(connection)
    }
}
