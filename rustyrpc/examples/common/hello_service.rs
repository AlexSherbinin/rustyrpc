use std::{borrow::Cow, future::Future, io, marker::PhantomData, sync::Arc};

use rustyrpc::{
    format::{
        Decode, DecodeZeroCopy, DecodeZeroCopyFallible, Encode, EncodingFormat,
        ZeroCopyEncodingFormat,
    },
    multipart::{MultipartReceived, MultipartSendable},
    protocol::{
        PrivateServiceDeallocateRequestResult, RequestKind, ServiceCallRequestError,
        ServiceCallRequestResult, ServiceKind,
    },
    server::PrivateServiceAllocator,
    service::{IntoService, Service, ServiceClient, ServiceMetadata, ServiceWrapper},
    transport, Client,
};

const SERVICE_NAME: &str = "Hello";
const SERVICE_CHECKSUM: &[u8] = &[];

pub trait HelloService<Format: EncodingFormat>: IntoService<Format> + Send + Sync {
    fn hello(&self) -> impl Future<Output = String> + Send;
}

pub struct HelloServiceWrapper<T: HelloService<Format>, Format: EncodingFormat>(
    T,
    PhantomData<Format>,
);

impl<T, Format> ServiceWrapper<T, Format> for HelloServiceWrapper<T, Format>
where
    T: HelloService<Format>,
    Format: EncodingFormat,
{
    fn wrap(to_wrap: T) -> Self {
        Self(to_wrap, PhantomData)
    }
}

impl<T, Format> ServiceMetadata<Format> for HelloServiceWrapper<T, Format>
where
    T: HelloService<Format> + Send + Sync,
    Format: EncodingFormat,
{
    const NAME: &'static str = SERVICE_NAME;
    const CHECKSUM: &'static [u8] = SERVICE_CHECKSUM;
}

#[async_trait::async_trait]
impl<T, Format> Service<Format> for HelloServiceWrapper<T, Format>
where
    T: HelloService<Format> + Send + Sync,
    Format: EncodingFormat,
{
    fn checksum(&self) -> Cow<'static, [u8]> {
        Cow::Borrowed(SERVICE_CHECKSUM)
    }

    async fn call(
        &self,
        _service_allocator: Arc<PrivateServiceAllocator<Format>>,
        function_id: u32,
        _args: MultipartReceived,
    ) -> Result<MultipartSendable, ServiceCallRequestError> {
        if function_id != 0 {
            return Err(ServiceCallRequestError::InvalidFunctionId);
        }

        let message = self
            .0
            .hello()
            .await
            .encode()
            .map_err(|_| ServiceCallRequestError::ServerInternal)?;

        Ok(MultipartSendable::from([message]))
    }
}

pub struct HelloServiceClient<
    Connection: transport::ClientConnection,
    Format: ZeroCopyEncodingFormat,
> where
    for<'a> RequestKind<'a>: Encode<Format>,
    for<'a> ServiceCallRequestResult<'a>: DecodeZeroCopy<
        'a,
        Format,
        <ServiceCallRequestResult<'a> as DecodeZeroCopyFallible<Format>>::Error,
    >,
    PrivateServiceDeallocateRequestResult: Decode<Format>,
{
    service_kind: ServiceKind,
    service_id: usize,
    rpc_client: Arc<Client<Connection, Format>>,
}

impl<Connection: transport::ClientConnection, Format: ZeroCopyEncodingFormat>
    ServiceClient<Connection, Format> for HelloServiceClient<Connection, Format>
where
    for<'a> RequestKind<'a>: Encode<Format>,
    for<'a> ServiceCallRequestResult<'a>: DecodeZeroCopy<
        'a,
        Format,
        <ServiceCallRequestResult<'a> as DecodeZeroCopyFallible<Format>>::Error,
    >,
    PrivateServiceDeallocateRequestResult: Decode<Format>,
{
    const SERVICE_NAME: &'static str = SERVICE_NAME;
    const SERVICE_CHECKSUM: &'static [u8] = SERVICE_CHECKSUM;

    fn new(
        service_kind: ServiceKind,
        service_id: usize,
        rpc_client: Arc<Client<Connection, Format>>,
    ) -> Self {
        Self {
            service_kind,
            service_id,
            rpc_client,
        }
    }
}

impl<Connection: transport::ClientConnection, Format: ZeroCopyEncodingFormat>
    HelloServiceClient<Connection, Format>
where
    for<'a> RequestKind<'a>: Encode<Format>,
    for<'a> ServiceCallRequestResult<'a>: DecodeZeroCopy<
        'a,
        Format,
        <ServiceCallRequestResult<'a> as DecodeZeroCopyFallible<Format>>::Error,
    >,
    PrivateServiceDeallocateRequestResult: Decode<Format>,
{
    pub async fn hello(&self) -> io::Result<String>
    where
        (): Encode<Format>,
        String: Decode<Format>,
    {
        self.rpc_client
            .call_service(
                self.service_kind,
                self.service_id.try_into().unwrap(),
                0,
                &(),
            )
            .await
    }
}

impl<Connection: transport::ClientConnection, Format: ZeroCopyEncodingFormat> Drop
    for HelloServiceClient<Connection, Format>
where
    for<'a> RequestKind<'a>: Encode<Format>,
    for<'a> ServiceCallRequestResult<'a>: DecodeZeroCopy<
        'a,
        Format,
        <ServiceCallRequestResult<'a> as DecodeZeroCopyFallible<Format>>::Error,
    >,
    PrivateServiceDeallocateRequestResult: Decode<Format>,
{
    fn drop(&mut self) {
        if let ServiceKind::Private = self.service_kind {
            let rpc_client = Arc::clone(&self.rpc_client);
            let service_id = self.service_id.try_into().unwrap();
            tokio::spawn(async move {
                rpc_client
                    .deallocate_private_service(service_id)
                    .await
                    .unwrap();
            });
        }
    }
}
