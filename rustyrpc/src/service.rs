use alloc::{borrow::Cow, sync::Arc};

use async_trait::async_trait;

use crate::{
    client::Client,
    format::EncodingFormat,
    protocol::{ServiceCallRequestError, ServiceKind},
    server::PrivateServiceAllocator,
    transport,
};

/// Service client for interaction with specific remote service.
pub trait ServiceClient<Connection: transport::ClientConnection, Format: EncodingFormat>
where
    Self: Sized,
{
    /// Service name that client corresponds to
    const SERVICE_NAME: &'static str;
    /// Service checksum that client corresponds to
    const SERVICE_CHECKSUM: &'static [u8];

    /// Create new service client from service kind, id and RPC client
    fn new(
        service_kind: ServiceKind,
        service_id: usize,
        rpc_client: Arc<Client<Connection, Format>>,
    ) -> Self;
}

/// Service wrapper that wraps implementor of specific service trait like `AuthService` to implement [`Service`].
pub trait ServiceWrapper<T, Format: EncodingFormat>: ServiceMetadata<Format> {
    /// Wrap specific service trait implementor.
    fn wrap(to_wrap: T) -> Self;
}

/// Metadata of service.
pub trait ServiceMetadata<Format: EncodingFormat>: Service<Format> {
    /// Service name.
    const NAME: &'static str;
    /// Service checksum.
    const CHECKSUM: &'static [u8];
}

/// Service that can be called remotely
#[async_trait]
pub trait Service<Format: EncodingFormat>: Send + Sync {
    /// Returns checksum of service.
    fn checksum(&self) -> Cow<'static, [u8]>;

    /// Call service.
    async fn call(
        &self,
        service_allocator: Arc<PrivateServiceAllocator<Format>>,
        function_id: u32,
        args: Vec<u8>,
    ) -> Result<Vec<u8>, ServiceCallRequestError>;
}

/// An implementor of specific service trait that can be converted to [`Service`] with specified wrapper.
pub trait IntoService<Format: EncodingFormat>
where
    Self: Sized,
{
    /// Wrapper for converting to service
    type Wrapper: ServiceWrapper<Self, Format>;

    /// Converts to service via [`Wrapper`][IntoService::Wrapper]
    fn into_service(self) -> Self::Wrapper {
        Self::Wrapper::wrap(self)
    }
}
