use core::marker::PhantomData;

use crate::{
    format::EncodingFormat,
    transport::{self},
};

use super::call_stream::CallStream;

pub(crate) struct ClientConnection<Connection: transport::Connection, Format: EncodingFormat> {
    connection: Connection,
    _format: PhantomData<Format>,
}

impl<Connection: transport::Connection, Format: EncodingFormat>
    ClientConnection<Connection, Format>
{
    pub(crate) async fn accept_call_stream(
        &mut self,
    ) -> Result<CallStream<Connection::Stream, Format>, Connection::Error> {
        Ok(self.connection.accept_stream().await?.into())
    }
}

impl<Connection: transport::Connection, Format: EncodingFormat> From<Connection>
    for ClientConnection<Connection, Format>
{
    fn from(connection: Connection) -> Self {
        Self {
            connection,
            _format: PhantomData,
        }
    }
}
