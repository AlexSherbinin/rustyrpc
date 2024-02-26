mod allocator;
mod service_ref;

use crate::{format::EncodingFormat, service::Service};
use core::ops::Deref;
use derive_where::derive_where;
use tokio::sync::{Mutex, RwLock, RwLockReadGuard};

pub use allocator::PrivateServiceAllocator;
pub use service_ref::ServiceRef;

pub(crate) struct ServiceRefLock<'a, Format: EncodingFormat>(
    RwLockReadGuard<'a, Option<Box<dyn Service<Format>>>>,
);

impl<Format: EncodingFormat> Deref for ServiceRefLock<'_, Format> {
    type Target = Box<dyn Service<Format>>;

    #[allow(clippy::undocumented_unsafe_blocks)]
    fn deref(&self) -> &Self::Target {
        unsafe { self.0.as_ref().unwrap_unchecked() }
    }
}

#[derive_where(Default)]
pub(super) struct PrivateServices<Format: EncodingFormat> {
    services: boxcar::Vec<RwLock<Option<Box<dyn Service<Format>>>>>,
    free_ids: Mutex<Vec<usize>>,
}

impl<Format: EncodingFormat> PrivateServices<Format> {
    pub(super) async fn push(&self, service: Box<dyn Service<Format>>) -> usize {
        if let Some(service_id) = self.free_ids.lock().await.pop() {
            #[allow(clippy::unwrap_used)]
            let mut service_entry = self.services.get(service_id).unwrap().write().await;
            *service_entry = Some(service);

            service_id
        } else {
            self.services.push(Some(service).into())
        }
    }

    pub(super) async fn get(&self, id: usize) -> Option<ServiceRefLock<Format>> {
        let service = self.services.get(id)?.read().await;
        service.as_ref()?;

        Some(ServiceRefLock(service))
    }

    pub(super) async fn remove(&self, id: usize) -> Option<Box<dyn Service<Format>>> {
        let mut service_entry = self.services.get(id)?.write().await;

        if service_entry.is_some() {
            self.free_ids.lock().await.push(id);
        }

        service_entry.take()
    }
}
