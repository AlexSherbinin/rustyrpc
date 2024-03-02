//! # Remote service id request
//! ```markdown
//! RequestKind::ServiceIdRequest --> Server
//! Client <-- ServiceRequestResult
//! ```
//!
//! # Remote call
//! ```markdown
//! RequestKind::ServiceCallRequest --> Server
//! Args --> Server
//! Client <-- ServiceCallRequestResult
//! Client <-- Returns
//! ```

use std::io;

use thiserror::Error;

/// Response on service id request
pub type ServiceIdRequestResult = Result<ServiceFound, RemoteServiceIdRequestError>;
/// Response on service call request
pub type ServiceCallRequestResult = Result<(), ServiceCallRequestError>;
/// Response on private service deallocation request
pub type PrivateServiceDeallocateRequestResult = Result<(), InvalidPrivateServiceIdError>;

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
    /// Request to deallocate private service
    DeallocatePrivateService {
        /// Private service id
        id: u32,
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

impl From<RemoteServiceIdRequestError> for io::Error {
    fn from(error: RemoteServiceIdRequestError) -> Self {
        let kind = match error {
            RemoteServiceIdRequestError::ServiceNotFound => io::ErrorKind::NotFound,
            RemoteServiceIdRequestError::InvalidChecksum => io::ErrorKind::InvalidInput,
        };

        io::Error::new(kind, error)
    }
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
    /// Indicates a failure caused by internal server errors.
    #[error("Unexpected error caused by server")]
    ServerInternal,
}

impl From<ServiceCallRequestError> for io::Error {
    fn from(error: ServiceCallRequestError) -> Self {
        let kind = if let ServiceCallRequestError::ServerInternal = error {
            io::ErrorKind::Other
        } else {
            io::ErrorKind::InvalidInput
        };

        io::Error::new(kind, error)
    }
}

/// Error that may occur while trying to deallocate private service.
#[derive(Error, Debug)]
#[error("Invalid private service id")]
pub struct InvalidPrivateServiceIdError;

impl From<InvalidPrivateServiceIdError> for io::Error {
    fn from(error: InvalidPrivateServiceIdError) -> Self {
        io::Error::new(io::ErrorKind::InvalidInput, error)
    }
}
