use std::{borrow::Cow, future::Future, marker::PhantomData, sync::Arc};

use async_trait::async_trait;
use derive_where::derive_where;
use rkyv::{with::RefAsBox, Archive, Serialize};
use rustyrpc::{
    client::Client,
    format::{Decode, Encode, EncodingFormat},
    protocol::{
        PrivateServiceDeallocateRequestResult, RequestKind, ServiceCallRequestError,
        ServiceCallRequestResult, ServiceKind,
    },
    server::{PrivateServiceAllocator, ServiceRef},
    service::{IntoService, Service, ServiceClient, ServiceMetadata, ServiceWrapper},
    transport,
};

use super::hello_service::HelloServiceClient;

const SERVICE_NAME: &str = "Auth";
const SERVICE_CHECKSUM: &[u8] = &[];

#[derive(Serialize, Archive)]
#[archive(check_bytes)]
pub struct AuthRequest<'a> {
    #[with(RefAsBox)]
    username: &'a str,
    #[with(RefAsBox)]
    password: &'a str,
}

pub trait AuthService<Format>: IntoService<Format> + Send + Sync
where
    Format: EncodingFormat,
{
    fn auth(
        &self,
        username: &str,
        password: &str,
    ) -> impl Future<Output = Option<Box<dyn Service<Format>>>> + Send;
}

pub struct AuthServiceWrapper<T: AuthService<Format>, Format: EncodingFormat>(
    T,
    PhantomData<Format>,
);

impl<T, Format> ServiceWrapper<T, Format> for AuthServiceWrapper<T, Format>
where
    T: AuthService<Format>,
    Format: EncodingFormat,
    Option<ServiceRef>: Encode<Format>,
{
    fn wrap(to_wrap: T) -> Self {
        Self(to_wrap, PhantomData)
    }
}

impl<T, Format> ServiceMetadata<Format> for AuthServiceWrapper<T, Format>
where
    T: AuthService<Format>,
    Format: EncodingFormat,
    Option<ServiceRef>: Encode<Format>,
{
    const NAME: &'static str = SERVICE_NAME;
    const CHECKSUM: &'static [u8] = SERVICE_CHECKSUM;
}

#[async_trait]
impl<T, Format> Service<Format> for AuthServiceWrapper<T, Format>
where
    T: AuthService<Format>,
    Format: EncodingFormat,
    Option<ServiceRef>: Encode<Format>,
{
    fn checksum(&self) -> std::borrow::Cow<'static, [u8]> {
        Cow::Borrowed(SERVICE_CHECKSUM)
    }

    async fn call(
        &self,
        service_allocator: Arc<PrivateServiceAllocator<Format>>,
        function_id: u32,
        args: Vec<u8>,
    ) -> Result<Vec<u8>, ServiceCallRequestError> {
        if function_id != 0 {
            return Err(ServiceCallRequestError::InvalidFunctionId);
        }

        let request = rkyv::check_archived_root::<AuthRequest>(&args)
            .map_err(|_| ServiceCallRequestError::ArgsDecode)?;

        let service_response = self.0.auth(&request.username, &request.password).await;
        let service_ref = if let Some(service) = service_response {
            Some(service_allocator.allocate(service).await)
        } else {
            None
        };

        service_ref
            .encode()
            .map_err(|_| ServiceCallRequestError::ServerInternal)
    }
}

#[derive_where(Clone)]
pub struct AuthServiceClient<Connection: transport::ClientConnection, Format: EncodingFormat> {
    service_kind: ServiceKind,
    service_id: usize,
    rpc_client: Arc<Client<Connection, Format>>,
}

impl<Connection: transport::ClientConnection, Format: EncodingFormat>
    AuthServiceClient<Connection, Format>
where
    for<'a> RequestKind<'a>: Encode<Format>,
    ServiceCallRequestResult: Decode<Format>,
    PrivateServiceDeallocateRequestResult: Decode<Format>,
{
    pub async fn auth(
        &self,
        username: &str,
        password: &str,
    ) -> Option<HelloServiceClient<Connection, Format>>
    where
        for<'a> AuthRequest<'a>: Encode<Format>,
        Option<ServiceRef>: Decode<Format>,
    {
        let request = AuthRequest { username, password };
        let service_ref: Option<ServiceRef> = self
            .rpc_client
            .call_service(
                self.service_kind,
                self.service_id.try_into().unwrap(),
                0,
                &request,
            )
            .await
            .unwrap();

        service_ref.map(|service_ref| service_ref.into_client(self.rpc_client.clone()).unwrap())
    }
}

impl<Connection: transport::ClientConnection, Format: EncodingFormat>
    ServiceClient<Connection, Format> for AuthServiceClient<Connection, Format>
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
