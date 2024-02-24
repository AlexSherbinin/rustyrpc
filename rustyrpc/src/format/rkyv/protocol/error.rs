use rkyv::{Archive, Deserialize, Serialize};

use crate::protocol;

#[derive(Serialize, Deserialize, Archive)]
#[archive(check_bytes)]
pub enum RemoteServiceIdRequestError {
    ServiceNotFound,
    InvalidChecksum,
}

impl From<RemoteServiceIdRequestError> for protocol::RemoteServiceIdRequestError {
    fn from(error: RemoteServiceIdRequestError) -> Self {
        match error {
            RemoteServiceIdRequestError::ServiceNotFound => Self::ServiceNotFound,
            RemoteServiceIdRequestError::InvalidChecksum => Self::InvalidChecksum,
        }
    }
}

impl From<&protocol::RemoteServiceIdRequestError> for RemoteServiceIdRequestError {
    fn from(error: &protocol::RemoteServiceIdRequestError) -> Self {
        match error {
            protocol::RemoteServiceIdRequestError::ServiceNotFound => Self::ServiceNotFound,
            protocol::RemoteServiceIdRequestError::InvalidChecksum => Self::InvalidChecksum,
        }
    }
}

#[derive(Serialize, Deserialize, Archive)]
#[archive(check_bytes)]
pub enum ServiceCallRequestError {
    InvalidServiceId,
    InvalidFunctionId,
    ArgsDecode,
    ReturnsDecode,
}

impl From<ServiceCallRequestError> for protocol::ServiceCallRequestError {
    fn from(error: ServiceCallRequestError) -> Self {
        match error {
            ServiceCallRequestError::InvalidServiceId => Self::InvalidServiceId,
            ServiceCallRequestError::InvalidFunctionId => Self::InvalidFunctionId,
            ServiceCallRequestError::ArgsDecode => Self::ArgsDecode,
            ServiceCallRequestError::ReturnsDecode => Self::ReturnsEncode,
        }
    }
}

impl From<&protocol::ServiceCallRequestError> for ServiceCallRequestError {
    fn from(error: &protocol::ServiceCallRequestError) -> Self {
        match error {
            protocol::ServiceCallRequestError::InvalidServiceId => Self::InvalidServiceId,
            protocol::ServiceCallRequestError::InvalidFunctionId => Self::InvalidFunctionId,
            protocol::ServiceCallRequestError::ArgsDecode => Self::ArgsDecode,
            protocol::ServiceCallRequestError::ReturnsEncode => Self::ReturnsDecode,
        }
    }
}
