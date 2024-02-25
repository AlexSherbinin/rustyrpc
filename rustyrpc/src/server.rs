mod builder;
mod call_handler;
mod call_stream;
mod client_connection;
mod private_service;
mod task_pool;

use self::{client_connection::ClientConnection, task_pool::TaskPool};
use crate::{
    format::{
        DecodeZeroCopy, DecodeZeroCopyFallible, Encode, EncodingFormat, ZeroCopyEncodingFormat,
    },
    protocol::{RequestKind, ServiceCallRequestResult, ServiceIdRequestResult},
    server::call_handler::ServerCallHandler,
    service::Service,
    transport,
};
use alloc::sync::Arc;
use core::marker::PhantomData;
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
                .spawn_task(Arc::clone(&self).handle_connection(connection));
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

        let call_handler = ServerCallHandler::new_for_connection(Arc::clone(&self));

        loop {
            let call_stream = connection.accept_call_stream().await.unwrap();
            let call_handler = call_handler.clone();

            self.tasks.spawn_task(async move {
                call_stream.handle_call(call_handler).await.unwrap();
            });
        }
    }
}
