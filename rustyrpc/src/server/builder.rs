use core::marker::PhantomData;
use std::collections::HashMap;

use derive_where::derive_where;

use crate::{
    format::EncodingFormat,
    service::{IntoService, Service, ServiceMetadata},
    transport,
};

use super::{task_pool::TaskPool, Server};

/// Builder for [`Server`][Server]
#[derive_where(Default)]
pub struct ServerBuilder<Listener: transport::ConnectionListener, Format: EncodingFormat> {
    service_map: HashMap<Box<str>, (Box<[u8]>, u32)>,
    services: Vec<Box<dyn Service<Format>>>,
    _phantom: PhantomData<(Listener, Format)>,
}

impl<Listener: transport::ConnectionListener, Format: EncodingFormat>
    ServerBuilder<Listener, Format>
{
    /// Adds service to server.
    /// # Panics
    /// Panics if service count overflows u32
    #[must_use]
    pub fn with_service<S>(self, service: S) -> Self
    where
        S: IntoService<Format> + 'static,
    {
        let service = service.into_service();

        self.with_boxed_service(
            <S::Wrapper as ServiceMetadata<Format>>::NAME
                .to_owned()
                .into_boxed_str(),
            <S::Wrapper as ServiceMetadata<Format>>::CHECKSUM
                .to_vec()
                .into_boxed_slice(),
            Box::new(service),
        )
    }

    /// Adds boxed service to server.
    ///
    /// # Panics
    /// Panics if service count overflows u32
    #[must_use]
    pub fn with_boxed_service(
        mut self,
        name: Box<str>,
        checksum: Box<[u8]>,
        service: Box<dyn Service<Format>>,
    ) -> Self {
        #[allow(clippy::expect_used)]
        let service_id = self
            .services
            .len()
            .try_into()
            .expect("Too much services. Service count overflows u32!");
        self.services.push(service);
        self.service_map.insert(name, (checksum, service_id));

        self
    }

    /// Builds server from builder.
    pub fn build(self, listener: Listener) -> Server<Listener, Format> {
        Server {
            listener: listener.into(),
            tasks: TaskPool::default(),
            service_map: self.service_map,
            services: self.services.into_boxed_slice(),
            _format: PhantomData,
        }
    }
}
