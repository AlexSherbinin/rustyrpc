use core::{future::Future, marker::PhantomData};

use derive_where::derive_where;
use thiserror::Error;

use crate::{
    error::SendEncodableError,
    format::{
        DecodeZeroCopy, DecodeZeroCopyFallible, Encode, EncodingFormat, ZeroCopyEncodingFormat,
    },
    protocol::{
        RemoteServiceIdRequestError, RequestKind, ServiceCallRequestError,
        ServiceCallRequestResult, ServiceFound, ServiceIdRequestResult, ServiceKind,
    },
    transport::{self, StreamExt},
};

pub(crate) trait CallHandler {
    fn handle_call(
        self,
        kind: ServiceKind,
        service_id: u32,
        function_id: u32,
        args: Vec<u8>,
    ) -> impl Future<Output = Result<Vec<u8>, ServiceCallRequestError>> + Send;

    fn handle_service_request(
        self,
        name: &str,
        checksum: &[u8],
    ) -> impl Future<Output = Result<u32, RemoteServiceIdRequestError>>;
}

#[derive(Error)]
#[derive_where(Debug)]
pub(crate) enum CallHandleError<
    Stream: transport::Stream,
    Format: EncodingFormat,
    DecodeError: std::error::Error,
> where
    ServiceIdRequestResult: Encode<Format>,
    ServiceCallRequestResult: Encode<Format>,
{
    #[error("Failed to receive request")]
    RequestReceive(#[source] Stream::Error),
    #[error("Failed to decode call request")]
    RequestDecode(#[source] DecodeError),
    #[error("Failed to send response on service id request")]
    SendServiceIdResponse(
        #[from] SendEncodableError<Stream, <ServiceIdRequestResult as Encode<Format>>::Error>,
    ),
    #[error("Failed to receive arguments of service call")]
    ServiceCallArgumentsReceive(#[source] Stream::Error),
    #[error("Failed to send service call response")]
    SendServiceCallResponse(#[from] ServiceCallResponseSendError<Stream, Format>),
}

#[derive(Error)]
#[derive_where(Debug)]
pub(crate) enum ServiceCallResponseSendError<Stream: transport::Stream, Format: EncodingFormat>
where
    ServiceCallRequestResult: Encode<Format>,
{
    #[error("Failed to send call result")]
    Result(#[from] SendEncodableError<Stream, <ServiceCallRequestResult as Encode<Format>>::Error>),
    #[error("Failed to send what service returned")]
    ServiceReturn(#[source] Stream::Error),
}

pub(crate) struct CallStream<Stream: transport::Stream, Format: EncodingFormat> {
    stream: Stream,
    _format: PhantomData<Format>,
}

impl<Stream: transport::Stream, Format: ZeroCopyEncodingFormat> CallStream<Stream, Format>
where
    ServiceIdRequestResult: Encode<Format>,
{
    pub(crate) async fn handle_call<'a, H>(
        mut self,
        handler: H,
    ) -> Result<
        (),
        CallHandleError<Stream, Format, <RequestKind<'a> as DecodeZeroCopyFallible<Format>>::Error>,
    >
    where
        for<'b, 'c> RequestKind<'b>:
            DecodeZeroCopy<'b, Format, <RequestKind<'c> as DecodeZeroCopyFallible<Format>>::Error>,
        ServiceIdRequestResult: Encode<Format>,
        ServiceCallRequestResult: Encode<Format>,
        H: CallHandler,
    {
        let request = self
            .stream
            .receive()
            .await
            .map_err(CallHandleError::RequestReceive)?;
        let request =
            RequestKind::decode_zero_copy(&request).map_err(CallHandleError::RequestDecode)?;

        match request {
            RequestKind::ServiceId { name, checksum } => {
                self.handle_service_id_request(handler, name, checksum)
                    .await?;
            }
            RequestKind::ServiceCall {
                kind,
                id,
                function_id,
            } => {
                let args = self
                    .stream
                    .receive()
                    .await
                    .map_err(CallHandleError::ServiceCallArgumentsReceive)?;

                self.handle_service_call_request(handler, kind, id, function_id, args)
                    .await?;
            }
        }

        self.stream
            .stopped()
            .await
            .map_err(CallHandleError::RequestReceive)?;

        Ok(())
    }

    async fn handle_service_id_request<H: CallHandler>(
        &mut self,
        handler: H,
        name: &str,
        checksum: &[u8],
    ) -> Result<(), SendEncodableError<Stream, <ServiceIdRequestResult as Encode<Format>>::Error>>
    {
        let response = handler
            .handle_service_request(name, checksum)
            .await
            .map(ServiceFound);

        self.stream.send_encodable(&response).await
    }

    async fn handle_service_call_request<H: CallHandler>(
        &mut self,
        handler: H,
        kind: ServiceKind,
        service_id: u32,
        function_id: u32,
        args: Vec<u8>,
    ) -> Result<(), ServiceCallResponseSendError<Stream, Format>>
    where
        ServiceCallRequestResult: Encode<Format>,
    {
        match handler
            .handle_call(kind, service_id, function_id, args)
            .await
        {
            Ok(returns) => {
                self.stream
                    .send_encodable::<ServiceCallRequestResult, _>(&Ok(()))
                    .await?;
                self.stream
                    .send(returns)
                    .await
                    .map_err(ServiceCallResponseSendError::ServiceReturn)?;
            }
            Err(err) => {
                self.stream
                    .send_encodable::<ServiceCallRequestResult, _>(&Err(err))
                    .await?;
            }
        }

        Ok(())
    }
}

impl<Stream: transport::Stream, Format: EncodingFormat> From<Stream>
    for CallStream<Stream, Format>
{
    fn from(stream: Stream) -> Self {
        Self {
            stream,
            _format: PhantomData,
        }
    }
}
