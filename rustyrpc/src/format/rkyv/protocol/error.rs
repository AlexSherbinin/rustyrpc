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

impl From<&ArchivedServiceCallRequestError> for protocol::ServiceCallRequestError {
    fn from(error: &ArchivedServiceCallRequestError) -> Self {
        match error {
            ArchivedServiceCallRequestError::InvalidServiceId => Self::InvalidServiceId,
            ArchivedServiceCallRequestError::InvalidFunctionId => Self::InvalidFunctionId,
            ArchivedServiceCallRequestError::ArgsDecode => Self::ArgsDecode,
            ArchivedServiceCallRequestError::ReturnsDecode => Self::ServerInternal,
        }
    }
}

impl From<&protocol::ServiceCallRequestError> for ServiceCallRequestError {
    fn from(error: &protocol::ServiceCallRequestError) -> Self {
        match error {
            protocol::ServiceCallRequestError::InvalidServiceId => Self::InvalidServiceId,
            protocol::ServiceCallRequestError::InvalidFunctionId => Self::InvalidFunctionId,
            protocol::ServiceCallRequestError::ArgsDecode => Self::ArgsDecode,
            protocol::ServiceCallRequestError::ServerInternal => Self::ReturnsDecode,
        }
    }
}

#[derive(Serialize, Deserialize, Archive)]
#[archive(check_bytes)]
pub struct InvalidPrivateServiceIdError;

impl From<InvalidPrivateServiceIdError> for protocol::InvalidPrivateServiceIdError {
    fn from(_error: InvalidPrivateServiceIdError) -> Self {
        Self
    }
}

impl From<&protocol::InvalidPrivateServiceIdError> for InvalidPrivateServiceIdError {
    fn from(_error: &protocol::InvalidPrivateServiceIdError) -> Self {
        Self
    }
}
