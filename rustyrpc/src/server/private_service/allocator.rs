use derive_where::derive_where;

use crate::{format::EncodingFormat, service::Service};

use super::{service_ref::ServiceRef, PrivateServices, ServiceRefLock};

/// Allocator for private service refs.
#[derive_where(Default)]
pub struct PrivateServiceAllocator<Format: EncodingFormat>(PrivateServices<Format>);

impl<Format: EncodingFormat> PrivateServiceAllocator<Format> {
    /// Allocate service ref.
    pub async fn allocate(&self, service: Box<dyn Service<Format>>) -> ServiceRef {
        let checksum = service.checksum();
        let service_id = self.0.push(service).await;

        ServiceRef {
            service_id,
            service_checksum: checksum,
        }
    }

    /// Deallocate service ref.
    pub async fn deallocate(&self, service_ref: ServiceRef) -> Option<Box<dyn Service<Format>>> {
        self.deallocate_by_id(service_ref.service_id).await
    }

    /// Deallocate service ref by service id.
    pub async fn deallocate_by_id(&self, id: usize) -> Option<Box<dyn Service<Format>>> {
        self.0.remove(id).await
    }

    pub(crate) async fn get(&self, service_id: usize) -> Option<ServiceRefLock<Format>> {
        self.0.get(service_id).await
    }
}
