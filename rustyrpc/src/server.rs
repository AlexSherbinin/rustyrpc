mod builder;
mod call_stream;
mod client_connection;
mod private_service;
mod task_pool;

use self::{call_stream::CallHandler, client_connection::ClientConnection, task_pool::TaskPool};
use crate::{
    format::{
        DecodeZeroCopy, DecodeZeroCopyFallible, Encode, EncodingFormat, ZeroCopyEncodingFormat,
    },
    protocol::{
        RemoteServiceIdRequestError, RequestKind, ServiceCallRequestError,
        ServiceCallRequestResult, ServiceIdRequestResult, ServiceKind,
    },
    service::Service,
    transport,
};
use alloc::sync::Arc;
use core::marker::PhantomData;
use derive_where::derive_where;
use futures::lock::Mutex;
use log::trace;
use std::collections::HashMap;

pub use builder::ServerBuilder;
pub use private_service::{PrivateServiceAllocator, ServiceRef};

/// Server for handling incoming connections and managing service calls.
pub struct Server<Listener: transport::ConnectionListener, Format: EncodingFormat> {
    listener: Mutex<Listener>,
    tasks: TaskPool,
    service_map: HashMap<Box<str>, (Box<[u8]>, u32)>,
    services: Box<[Box<dyn Service<Format>>]>,
    _format: PhantomData<Format>,
}

// Server now supports only zero-copy formats. Non zero-copy formats coming soon.
impl<Listener: transport::ConnectionListener + 'static, Format: ZeroCopyEncodingFormat>
    Server<Listener, Format>
where
    for<'a, 'b> RequestKind<'a>:
        DecodeZeroCopy<'a, Format, <RequestKind<'b> as DecodeZeroCopyFallible<Format>>::Error>,
    ServiceIdRequestResult: Encode<Format>,
    ServiceCallRequestResult: Encode<Format>,
    u32: Encode<Format>,
{
    /// Starts listening for incoming connections and handles them.
    #[allow(clippy::unwrap_used, clippy::missing_panics_doc)]
    pub async fn listen(self: Arc<Self>) -> ! {
        loop {
            let connection = self.accept_connection().await.unwrap();
            self.tasks
                .spawn_task(Arc::clone(&self).handle_connection(connection))
                .await;
        }
    }

    async fn accept_connection(
        &self,
    ) -> Result<ClientConnection<Listener::Connection, Format>, Listener::Error> {
        self.listener
            .lock()
            .await
            .accept_connection()
            .await
            .map(Into::into)
    }

    #[allow(clippy::unwrap_used)]
    async fn handle_connection(
        self: Arc<Self>,
        mut connection: ClientConnection<Listener::Connection, Format>,
    ) {
        trace!("New connection accepted");

        let call_handler = ServerCallHandler {
            server: Arc::clone(&self),
            private_service_allocator: Arc::default(),
        };

        loop {
            let call_stream = connection.accept_call_stream().await.unwrap();
            let call_handler = call_handler.clone();

            self.tasks
                .spawn_task(async move {
                    call_stream.handle_call(call_handler).await.unwrap();
                })
                .await;
        }
    }
}

#[derive_where(Clone)]
struct ServerCallHandler<Listener: transport::ConnectionListener, Format: EncodingFormat> {
    server: Arc<Server<Listener, Format>>,
    private_service_allocator: Arc<PrivateServiceAllocator<Format>>,
}

impl<Listener: transport::ConnectionListener, Format: EncodingFormat> CallHandler
    for ServerCallHandler<Listener, Format>
{
    async fn handle_call(
        self,
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
                service
                    .call(
                        Arc::clone(&self.private_service_allocator),
                        function_id,
                        args,
                    )
                    .await
            }
            ServiceKind::Private
                if let Some(service) = self.private_service_allocator.get(service_id).await =>
            {
                service
                    .call(
                        Arc::clone(&self.private_service_allocator),
                        function_id,
                        args,
                    )
                    .await
            }
            ServiceKind::Public | ServiceKind::Private => {
                Err(ServiceCallRequestError::InvalidServiceId)
            }
        }
    }

    async fn handle_service_request(
        self,
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
}
