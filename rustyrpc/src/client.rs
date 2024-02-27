use alloc::sync::Arc;
use core::marker::PhantomData;
use core::ops::DerefMut;
use std::io;
use tokio::sync::Mutex;

use crate::{
    format::{self, Decode, Encode, EncodingFormat},
    protocol::{
        PrivateServiceDeallocateRequestResult, RequestKind, ServiceCallRequestResult,
        ServiceIdRequestResult, ServiceKind,
    },
    service::ServiceClient,
    transport::{self, Stream, StreamExt},
    utils::{ConnectionCloseOnDrop, DropOwned},
};

/// RPC client for calling remote services.
pub struct Client<Connection: transport::Connection, Format: format::EncodingFormat> {
    connection: Mutex<DropOwned<ConnectionCloseOnDrop<Connection>>>,
    _format: PhantomData<Format>,
}

impl<Connection: transport::Connection, Format: format::EncodingFormat> Client<Connection, Format>
where
    for<'a> RequestKind<'a>: Encode<Format>,
{
    async fn open_new_stream(&self) -> io::Result<Connection::Stream> {
        let mut transport_connection = self.connection.lock().await;
        transport_connection.deref_mut().0.new_stream().await
    }

    /// Retrieves a service specified by service client.
    ///
    /// # Errors
    /// Returns an error if service request fails.
    pub async fn get_service_client<T>(self: Arc<Self>) -> io::Result<T>
    where
        ServiceIdRequestResult: Decode<Format>,
        T: ServiceClient<Connection, Format>,
    {
        let service_id = self
            .request_service(T::SERVICE_NAME, T::SERVICE_CHECKSUM)
            .await?;
        let service_id = service_id
            .try_into()
            .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))?;

        Ok(T::new(ServiceKind::Public, service_id, self))
    }

    /// Retrieves a service. It's different from [`get_service_client`][Client::get_service_client], because just returns received service id except client
    ///
    /// # Errors
    /// Returns an error if service request fails.
    pub async fn request_service<'a>(&self, name: &'a str, checksum: &'a [u8]) -> io::Result<u32>
    where
        ServiceIdRequestResult: Decode<Format>,
    {
        let mut request_stream = self.open_new_stream().await?;

        let request = RequestKind::ServiceId { name, checksum };
        request_stream.send_encodable(&request).await?;

        let service_id = request_stream
            .receive_decodable::<ServiceIdRequestResult, _>()
            .await??;
        request_stream.close().await?;
        Ok(service_id.0)
    }

    /// Calls a remote service.
    ///
    /// # Errors
    /// Returns an error if service call fails.
    pub async fn call_service<Args, Returns>(
        &self,
        kind: ServiceKind,
        id: u32,
        function_id: u32,
        args: &Args,
    ) -> io::Result<Returns>
    where
        Args: Encode<Format>,
        Returns: Decode<Format>,
        ServiceCallRequestResult: Decode<Format>,
    {
        let mut request_stream = self.open_new_stream().await?;

        let request = RequestKind::ServiceCall {
            kind,
            id,
            function_id,
        };
        request_stream.send_encodable(&request).await?;
        request_stream.send_encodable(args).await?;

        request_stream
            .receive_decodable::<ServiceCallRequestResult, _>()
            .await??;

        let result = request_stream.receive_decodable().await?;
        request_stream.close().await?;
        Ok(result)
    }

    /// Deallocate private service previously returned from public service.
    ///
    /// # Errors
    /// Returns an error if service deallocation fails.
    pub async fn deallocate_private_service(&self, id: u32) -> io::Result<()>
    where
        PrivateServiceDeallocateRequestResult: Decode<Format>,
    {
        let mut request_stream = self.open_new_stream().await?;

        let request = RequestKind::DeallocatePrivateService { id };
        request_stream.send_encodable(&request).await?;

        request_stream
            .receive_decodable::<PrivateServiceDeallocateRequestResult, _>()
            .await??;
        request_stream.close().await?;
        Ok(())
    }
}

impl<Connection: transport::Connection, Format: EncodingFormat> From<Connection>
    for Client<Connection, Format>
{
    fn from(connection: Connection) -> Self {
        Self {
            connection: Mutex::new(ConnectionCloseOnDrop(connection).into()),
            _format: PhantomData,
        }
    }
}
