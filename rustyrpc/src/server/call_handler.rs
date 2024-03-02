use super::{call_stream::CallHandler, PrivateServiceAllocator, Server};
use crate::{
    format::EncodingFormat,
    protocol::{
        InvalidPrivateServiceIdError, RemoteServiceIdRequestError, ServiceCallRequestError,
        ServiceKind,
    },
    service::Service,
    transport,
};
use alloc::sync::Arc;
use derive_where::derive_where;
use log::trace;

#[derive_where(Clone)]
pub(super) struct ServerCallHandler<Listener: transport::ConnectionListener, Format: EncodingFormat>
{
    server: Arc<Server<Listener, Format>>,
    private_service_allocator: Arc<PrivateServiceAllocator<Format>>,
}

impl<Listener: transport::ConnectionListener, Format: EncodingFormat>
    ServerCallHandler<Listener, Format>
{
    pub(super) fn new_for_connection(server: Arc<Server<Listener, Format>>) -> Self {
        Self {
            server,
            private_service_allocator: Arc::default(),
        }
    }

    async fn handle_private_service_call(
        self,
        service: super::private_service::ServiceRefLock<'_, Format>,
        function_id: u32,
        args: Vec<u8>,
    ) -> Result<Vec<u8>, ServiceCallRequestError> {
        service
            .call(
                Arc::clone(&self.private_service_allocator),
                function_id,
                args,
            )
            .await
    }

    async fn handle_public_service_call(
        self,
        service: &dyn Service<Format>,
        function_id: u32,
        args: Vec<u8>,
    ) -> Result<Vec<u8>, ServiceCallRequestError> {
        service
            .call(
                Arc::clone(&self.private_service_allocator),
                function_id,
                args,
            )
            .await
    }
}

impl<Listener: transport::ConnectionListener, Format: EncodingFormat> CallHandler
    for ServerCallHandler<Listener, Format>
{
    async fn handle_call(
        &self,
        kind: ServiceKind,
        service_id: u32,
        function_id: u32,
        args: Vec<u8>,
    ) -> Result<Vec<u8>, ServiceCallRequestError> {
        trace!("Received service call. Kind: {kind:?}, service id: {service_id}, function_id: {function_id}");

        #[allow(clippy::map_err_ignore)]
        let service_id: usize = service_id
            .try_into()
            .map_err(|_| ServiceCallRequestError::InvalidServiceId)?;

        match kind {
            ServiceKind::Public if let Some(service) = self.server.services.get(service_id) => {
                self.clone()
                    .handle_public_service_call(service.as_ref(), function_id, args)
                    .await
            }
            ServiceKind::Private
                if let Some(service) = self.private_service_allocator.get(service_id).await =>
            {
                self.clone()
                    .handle_private_service_call(service, function_id, args)
                    .await
            }
            ServiceKind::Public | ServiceKind::Private => {
                Err(ServiceCallRequestError::InvalidServiceId)
            }
        }
    }

    async fn handle_service_request(
        &self,
        name: &str,
        checksum: &[u8],
    ) -> Result<u32, RemoteServiceIdRequestError> {
        trace!("Received service request. Service name: {name}, checksum: {checksum:?}");

        let (expected_checksum, service_id) = self
            .server
            .service_map
            .get(name)
            .ok_or(RemoteServiceIdRequestError::ServiceNotFound)?;
        if checksum == &**expected_checksum {
            Ok(*service_id)
        } else {
            Err(RemoteServiceIdRequestError::InvalidChecksum)
        }
    }

    #[allow(clippy::map_err_ignore)]
    async fn handle_private_service_deallocation(
        &self,
        service_id: u32,
    ) -> Result<(), InvalidPrivateServiceIdError> {
        trace!("Received private service deallocation request. Service id: {service_id}");

        let service_id = service_id
            .try_into()
            .map_err(|_| InvalidPrivateServiceIdError)?;
        if self
            .private_service_allocator
            .deallocate_by_id(service_id)
            .await
            .is_some()
        {
            Ok(())
        } else {
            Err(InvalidPrivateServiceIdError)
        }
    }
}
