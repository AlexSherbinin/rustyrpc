use std::{borrow::Cow, future::Future, marker::PhantomData, sync::Arc};

use derive_where::derive_where;
use rustyrpc::{
    client::Client,
    error::ServiceCallError,
    format::{Decode, Encode, EncodingFormat},
    protocol::{RequestKind, ServiceCallRequestError, ServiceCallRequestResult, ServiceKind},
    server::PrivateServiceAllocator,
    service::{IntoService, Service, ServiceClient, ServiceMetadata, ServiceWrapper},
    transport,
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
        _args: Vec<u8>,
    ) -> Result<Vec<u8>, ServiceCallRequestError> {
        if function_id != 0 {
            return Err(ServiceCallRequestError::InvalidFunctionId);
        }

        self.0
            .hello()
            .await
            .encode()
            .map_err(|_| ServiceCallRequestError::ReturnsEncode)
    }
}

#[derive_where(Clone)]
pub struct HelloServiceClient<Connection: transport::Connection, Format: EncodingFormat> {
    service_kind: ServiceKind,
    service_id: usize,
    rpc_client: Arc<Client<Connection, Format>>,
}

impl<Connection: transport::Connection, Format: EncodingFormat> ServiceClient<Connection, Format>
    for HelloServiceClient<Connection, Format>
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

impl<Connection: transport::Connection, Format: EncodingFormat>
    HelloServiceClient<Connection, Format>
{
    pub async fn hello(&self) -> Result<String, ServiceCallError<Connection, Format, (), String>>
    where
        (): Encode<Format>,
        String: Decode<Format>,
        RequestKind<'static>: Encode<Format>,
        ServiceCallRequestResult: Decode<Format>,
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
