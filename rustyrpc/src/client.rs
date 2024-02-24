use alloc::sync::Arc;
use core::marker::PhantomData;

use futures::lock::Mutex;
use log::warn;

use crate::{
    error::{ServiceCallError, ServiceRequestError},
    format::{self, Decode, Encode, EncodingFormat},
    protocol::{RequestKind, ServiceCallRequestResult, ServiceIdRequestResult, ServiceKind},
    service::ServiceClient,
    transport::{self, Stream, StreamExt},
};

/// RPC client for calling remote services.
pub struct Client<Connection: transport::Connection, Format: format::EncodingFormat> {
    connection: Mutex<Connection>,
    not_closed_warn: ClientNotClosedWarn,
    _format: PhantomData<Format>,
}

impl<Connection: transport::Connection, Format: format::EncodingFormat> Client<Connection, Format> {
    async fn open_new_stream(&self) -> Result<Connection::Stream, Connection::Error> {
        let mut transport_connection = self.connection.lock().await;
        transport_connection.new_stream().await
    }

    /// Retrieves a service specified by service client.
    ///
    /// # Errors
    /// Returns an error if service request fails.
    pub async fn get_service_client<T>(
        self: Arc<Self>,
    ) -> Result<T, ServiceRequestError<'static, Connection, Format>>
    where
        for<'b> RequestKind<'b>: Encode<Format>,
        ServiceIdRequestResult: Decode<Format>,
        T: ServiceClient<Connection, Format>,
    {
        self.request_service(T::SERVICE_NAME, T::SERVICE_CHECKSUM)
            .await
            .map(|service_id| T::new(ServiceKind::Public, service_id, self))
    }

    /// Retrieves a service. It's different from [`get_service_client`][Client::get_service_client], because just returns received service id except client
    ///
    /// # Errors
    /// Returns an error if service request fails.
    pub async fn request_service<'a>(
        &self,
        name: &'a str,
        checksum: &'a [u8],
    ) -> Result<usize, ServiceRequestError<'a, Connection, Format>>
    where
        for<'b> RequestKind<'b>: Encode<Format>,
        ServiceIdRequestResult: Decode<Format>,
    {
        let mut request_stream = self
            .open_new_stream()
            .await
            .map_err(ServiceRequestError::StreamManagement)?;

        let request = RequestKind::ServiceId { name, checksum };
        request_stream
            .send_encodable(&request)
            .await
            .map_err(ServiceRequestError::RequestEncode)?;

        let service_id = request_stream
            .receive_decodable::<ServiceIdRequestResult, _>()
            .await??
            .0;
        request_stream
            .close()
            .await
            .map_err(ServiceRequestError::StreamIO)?;
        service_id
            .try_into()
            .map_err(ServiceRequestError::InvalidServiceId)
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
    ) -> Result<Returns, ServiceCallError<Connection, Format, Args, Returns>>
    where
        RequestKind<'static>: Encode<Format>,
        Args: Encode<Format>,
        Returns: Decode<Format>,
        ServiceCallRequestResult: Decode<Format>,
    {
        let mut request_stream = self
            .open_new_stream()
            .await
            .map_err(ServiceCallError::StreamManagement)?;

        let request = RequestKind::ServiceCall {
            kind,
            id,
            function_id,
        };
        request_stream
            .send_encodable(&request)
            .await
            .map_err(ServiceCallError::RequestEncode)?;
        request_stream
            .send_encodable(args)
            .await
            .map_err(ServiceCallError::ArgsEncode)?;

        request_stream
            .receive_decodable::<ServiceCallRequestResult, _>()
            .await
            .map_err(ServiceCallError::ResponseDecode)?
            .map_err(ServiceCallError::Remote)?;

        let result = request_stream
            .receive_decodable()
            .await
            .map_err(ServiceCallError::ReturnsDecode)?;
        request_stream
            .close()
            .await
            .map_err(ServiceCallError::StreamIO)?;
        Ok(result)
    }

    /// Closes underlying connection
    ///
    /// # Errors
    /// Returns an error if underlying connection returned error while close.
    pub async fn close(self) -> Result<(), Connection::Error> {
        self.connection.into_inner().close().await?;
        self.not_closed_warn.defuse();
        Ok(())
    }
}

impl<Connection: transport::Connection, Format: EncodingFormat> From<Connection>
    for Client<Connection, Format>
{
    fn from(connection: Connection) -> Self {
        Self {
            connection: connection.into(),
            not_closed_warn: ClientNotClosedWarn::default(),
            _format: PhantomData,
        }
    }
}

#[derive(Default)]
struct ClientNotClosedWarn(bool);

impl ClientNotClosedWarn {
    fn defuse(mut self) {
        self.0 = true;
    }
}

impl Drop for ClientNotClosedWarn {
    fn drop(&mut self) {
        if !self.0 {
            warn!("Client is dropped but not closed asynchronously");
        }
    }
}
