use alloc::sync::Arc;
use core::marker::PhantomData;
use core::ops::DerefMut;
use std::io;
use tokio::sync::Mutex;

use crate::{
    format::{self, Decode, DecodeZeroCopy, DecodeZeroCopyFallible, Encode, EncodingFormat},
    multipart::{MultipartReceived, MultipartSendable},
    protocol::{
        PrivateServiceDeallocateRequestResult, RequestKind, ServiceCallRequestResult,
        ServiceIdRequestResult, ServiceKind,
    },
    service::ServiceClient,
    transport::{self, Stream, StreamExt},
    utils::{ConnectionCloseOnDrop, DropOwned},
};

/// RPC client for calling remote services.
pub struct Client<Connection: transport::ClientConnection, Format: format::EncodingFormat> {
    connection: Mutex<DropOwned<ConnectionCloseOnDrop<Connection>>>,
    _format: PhantomData<Format>,
}

impl<Connection: transport::ClientConnection, Format: format::ZeroCopyEncodingFormat>
    Client<Connection, Format>
where
    for<'a> RequestKind<'a>: Encode<Format>,
    for<'a> ServiceCallRequestResult<'a>: DecodeZeroCopy<
        'a,
        Format,
        <ServiceCallRequestResult<'a> as DecodeZeroCopyFallible<Format>>::Error,
    >,
{
    async fn new_stream(&self) -> io::Result<Connection::Stream> {
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
        let mut request_stream = self.new_stream().await?;

        let request = RequestKind::ServiceId { name, checksum };
        request_stream.send_encodable(&request).await?;
        request_stream.flush().await?;

        let service_id = request_stream
            .receive_decodable::<ServiceIdRequestResult, _>()
            .await??;
        Ok(service_id.0)
    }

    /// Call a remote service with multipart as arguments.
    /// # Errors
    /// Returns an error if service call fails.
    pub async fn call_service_multipart(
        &self,
        kind: ServiceKind,
        id: u32,
        function_id: u32,
        args: &MultipartSendable,
    ) -> io::Result<MultipartReceived> {
        let mut request_stream = self.new_stream().await?;

        let part_sizes: Vec<u32> = args
            .iter()
            .map(|part| part.len().try_into())
            .try_collect()
            .map_err(|err| io::Error::new(io::ErrorKind::InvalidInput, err))?;

        let request = RequestKind::ServiceCall {
            kind,
            id,
            function_id,
            part_sizes: &part_sizes,
        };
        request_stream.send_encodable(&request).await?;
        request_stream.send_multipart(args).await?;
        request_stream.flush().await?;

        let service_call_result = request_stream.receive().await?;
        let response_part_sizes = ServiceCallRequestResult::decode_zero_copy(&service_call_result)
            .map_err(|err| io::Error::new(io::ErrorKind::Other, err))??;
        MultipartReceived::receive_from_stream(&mut request_stream, response_part_sizes).await
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
    {
        let args_encoded = args
            .encode()
            .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))?;
        let request_multipart = MultipartSendable::from([args_encoded]);

        let response_multipart = self
            .call_service_multipart(kind, id, function_id, &request_multipart)
            .await?;

        let response = response_multipart.iter().next().ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::Other,
                "Server sent no multipart when expected at least one",
            )
        })?;
        Returns::decode(response).map_err(|err| io::Error::new(io::ErrorKind::Other, err))
    }

    /// Deallocate private service previously returned from public service.
    ///
    /// # Errors
    /// Returns an error if service deallocation fails.
    pub async fn deallocate_private_service(&self, id: u32) -> io::Result<()>
    where
        PrivateServiceDeallocateRequestResult: Decode<Format>,
    {
        let mut request_stream = self.new_stream().await?;

        let request = RequestKind::DeallocatePrivateService { id };
        request_stream.send_encodable(&request).await?;
        request_stream.flush().await?;

        request_stream
            .receive_decodable::<PrivateServiceDeallocateRequestResult, _>()
            .await??;
        Ok(())
    }
}

impl<Connection: transport::ClientConnection, Format: EncodingFormat> From<Connection>
    for Client<Connection, Format>
{
    fn from(connection: Connection) -> Self {
        Self {
            connection: Mutex::new(ConnectionCloseOnDrop(connection).into()),
            _format: PhantomData,
        }
    }
}
