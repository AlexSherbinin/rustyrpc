use alloc::{borrow::Cow, sync::Arc};

use crate::{client::Client, format::EncodingFormat, protocol::ServiceKind, service, transport};

/// Reference to private service in `PrivateServiceAllocator`
pub struct ServiceRef {
    /// Private service id
    pub service_id: usize,
    /// Private service checksum
    pub service_checksum: Cow<'static, [u8]>,
}

impl ServiceRef {
    /// Creates service client from reference and [rpc client][Client]
    pub fn into_client<
        ServiceClient: service::ServiceClient<Connection, Format>,
        Connection: transport::Connection,
        Format: EncodingFormat,
    >(
        self,
        rpc_client: Arc<Client<Connection, Format>>,
    ) -> Option<ServiceClient> {
        if *ServiceClient::SERVICE_CHECKSUM != *self.service_checksum {
            return None;
        }

        Some(ServiceClient::new(
            ServiceKind::Private,
            self.service_id,
            rpc_client,
        ))
    }
}
