use crate::{
    format::{
        DecodeZeroCopy, DecodeZeroCopyFallible, Encode, EncodingFormat, ZeroCopyEncodingFormat,
    },
    multipart::{MultipartReceived, MultipartSendable},
    protocol::{
        InvalidPrivateServiceIdError, PrivateServiceDeallocateRequestResult,
        RemoteServiceIdRequestError, RequestKind, ServiceCallRequestError,
        ServiceCallRequestResult, ServiceFound, ServiceIdRequestResult, ServiceKind,
    },
    transport::{self, StreamExt},
};
use core::{future::Future, marker::PhantomData};
use std::io;

pub(crate) trait CallHandler {
    fn handle_call(
        &self,
        kind: ServiceKind,
        service_id: u32,
        function_id: u32,
        args: MultipartReceived,
    ) -> impl Future<Output = Result<MultipartSendable, ServiceCallRequestError>> + Send;

    fn handle_service_request(
        &self,
        name: &str,
        checksum: &[u8],
    ) -> impl Future<Output = Result<u32, RemoteServiceIdRequestError>> + Send;

    fn handle_private_service_deallocation(
        &self,
        service_id: u32,
    ) -> impl Future<Output = Result<(), InvalidPrivateServiceIdError>> + Send;
}

pub(crate) struct CallStream<Stream: transport::Stream, Format: EncodingFormat> {
    stream: Stream,
    _format: PhantomData<Format>,
}

impl<Stream: transport::Stream, Format: ZeroCopyEncodingFormat> CallStream<Stream, Format>
where
    for<'b, 'c> RequestKind<'b>:
        DecodeZeroCopy<'b, Format, <RequestKind<'c> as DecodeZeroCopyFallible<Format>>::Error>,
    ServiceIdRequestResult: Encode<Format>,
    for<'a> ServiceCallRequestResult<'a>: Encode<Format>,
    PrivateServiceDeallocateRequestResult: Encode<Format>,
{
    pub(crate) async fn handle_call<H>(mut self, handler: &H) -> io::Result<()>
    where
        H: CallHandler,
    {
        loop {
            let request = self.stream.receive().await?;
            let request = RequestKind::decode_zero_copy(&request)
                .map_err(|err| io::Error::new(io::ErrorKind::Other, err))?;

            match request {
                RequestKind::ServiceId { name, checksum } => {
                    self.handle_service_id_request(handler, name, checksum)
                        .await?;
                }
                RequestKind::ServiceCall {
                    kind,
                    id,
                    function_id,
                    part_sizes,
                } => {
                    let args = MultipartReceived::receive_from_stream(&mut self.stream, part_sizes)
                        .await?;

                    self.handle_service_call_request(handler, kind, id, function_id, args)
                        .await?;
                }
                RequestKind::DeallocatePrivateService { id } => {
                    let response = handler.handle_private_service_deallocation(id).await;
                    self.stream.send_encodable(&response).await?;
                }
            }

            self.stream.flush().await?;
        }
    }

    async fn handle_service_id_request<H: CallHandler>(
        &mut self,
        handler: &H,
        name: &str,
        checksum: &[u8],
    ) -> io::Result<()> {
        let response = handler
            .handle_service_request(name, checksum)
            .await
            .map(ServiceFound);

        self.stream.send_encodable(&response).await
    }

    async fn handle_service_call_request<H: CallHandler>(
        &mut self,
        handler: &H,
        kind: ServiceKind,
        service_id: u32,
        function_id: u32,
        args: MultipartReceived,
    ) -> io::Result<()> {
        match handler
            .handle_call(kind, service_id, function_id, args)
            .await
        {
            Ok(returns) => {
                let part_sizes: Vec<u32> = returns
                    .iter()
                    .map(|part| {
                        part.len()
                            .try_into()
                            .map_err(|err| io::Error::new(io::ErrorKind::Other, err))
                    })
                    .try_collect()?;

                self.stream
                    .send_encodable::<ServiceCallRequestResult, _>(&Ok(&part_sizes))
                    .await?;
                self.stream.send_multipart(&returns).await?;
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
