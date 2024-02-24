//! # Remote callable id request
//! ```markdown
//! **Client opens new stream**
//! RequestKind::ServiceIdRequest --> Server
//! Client <-- ServiceRequestResult
//! **Client closes stream**
//! ```
//!
//! # Remote call
//! ```markdown
//! **Client opens new stream**
//! RequestKind::ServiceCallRequest --> Server
//! Args --> Server
//! Client <-- ServiceCallRequestResult
//! Client <-- Returns
//! **Client closes stream**
//! ```

use thiserror::Error;

/// Response on service id request
pub type ServiceIdRequestResult = Result<ServiceFound, RemoteServiceIdRequestError>;
/// Response on service call request
pub type ServiceCallRequestResult = Result<(), ServiceCallRequestError>;

/// Requests that can be made.
pub enum RequestKind<'a> {
    /// Request to retrieve service
    ServiceId {
        /// Name of service
        name: &'a str,
        /// Checksum of service
        checksum: &'a [u8],
    },
    /// Request to call service's function
    ServiceCall {
        /// Kind of service
        kind: ServiceKind,
        /// Service id
        id: u32,
        /// Service's function id
        function_id: u32,
    },
}

/// Kind of service.
#[derive(Debug, Clone, Copy)]
pub enum ServiceKind {
    /// Represents service that can be accessed with [`service id request`][RequestKind::ServiceId]
    Public,
    /// Represents service that can be accessed only by `ServiceRef`
    Private,
}

/// Successful result of finding a service, containing its service id.
pub struct ServiceFound(
    /// Service id
    pub u32,
);

/// Errors that may occur on remote host while executing service id request.
#[derive(Debug, Error)]
pub enum RemoteServiceIdRequestError {
    /// Indicates that the requested service was not found.
    #[error("Service not found")]
    ServiceNotFound,
    /// Indicates that service found but checksum doesn't match.
    #[error("Invalid service checksum")]
    InvalidChecksum,
}

/// Errors that may occur on remote host while executing service call.
#[derive(Debug, Error)]
pub enum ServiceCallRequestError {
    /// Indicates that the service call was invoked with an invalid service ID.
    #[error("Call invoked with invalid service id")]
    InvalidServiceId,
    /// Indicates that the service call was invoked with an invalid function ID.
    #[error("Call invoked with invalid function id")]
    InvalidFunctionId,
    /// Indicates a failure in decoding the function arguments.
    #[error("Failed to decode args")]
    ArgsDecode,
    /// Indicates a failure in encoding the value returned by the function.
    #[error("Failed to encode what function returned")]
    ReturnsEncode,
}
