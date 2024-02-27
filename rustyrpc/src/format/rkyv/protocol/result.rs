use rkyv::{ser::serializers::AllocSerializer, Fallible};

use crate::{
    format::{
        rkyv::{RkyvDeserializationError, RkyvFormat},
        Decode, Encode,
    },
    protocol::{self},
};

use super::error::{
    InvalidPrivateServiceIdError, RemoteServiceIdRequestError, ServiceCallRequestError,
};

impl Encode<RkyvFormat> for protocol::ServiceIdRequestResult {
    type Error = <AllocSerializer<0> as Fallible>::Error;

    fn encode(&self) -> Result<Vec<u8>, Self::Error> {
        let result = self.as_ref().map(|result| result.0).map_err(Into::into);
        rkyv::to_bytes::<Result<u32, RemoteServiceIdRequestError>, 0>(&result)
            .map(|buffer| buffer.to_vec())
    }
}

impl Decode<RkyvFormat> for protocol::ServiceIdRequestResult {
    type Error = RkyvDeserializationError;

    fn decode(buffer: &[u8]) -> Result<Self, Self::Error> {
        Ok(
            rkyv::from_bytes::<Result<u32, RemoteServiceIdRequestError>>(buffer)
                .map_err(|err| RkyvDeserializationError(err.to_string()))?
                .map(protocol::ServiceFound)
                .map_err(Into::into),
        )
    }
}

impl Encode<RkyvFormat> for protocol::ServiceCallRequestResult {
    type Error = <AllocSerializer<0> as Fallible>::Error;

    fn encode(&self) -> Result<Vec<u8>, Self::Error> {
        let result = self.as_ref().copied().map_err(Into::into);
        rkyv::to_bytes::<Result<(), ServiceCallRequestError>, 0>(&result)
            .map(|buffer| buffer.to_vec())
    }
}

impl Decode<RkyvFormat> for protocol::ServiceCallRequestResult {
    type Error = RkyvDeserializationError;

    fn decode(buffer: &[u8]) -> Result<Self, Self::Error> {
        Ok(
            rkyv::from_bytes::<Result<(), ServiceCallRequestError>>(buffer)
                .map_err(|err| RkyvDeserializationError(err.to_string()))?
                .map_err(Into::into),
        )
    }
}

impl Encode<RkyvFormat> for protocol::PrivateServiceDeallocateRequestResult {
    type Error = <AllocSerializer<0> as Fallible>::Error;

    fn encode(&self) -> Result<Vec<u8>, Self::Error> {
        let result = self.as_ref().copied().map_err(Into::into);
        rkyv::to_bytes::<Result<(), InvalidPrivateServiceIdError>, 0>(&result)
            .map(|buffer| buffer.to_vec())
    }
}

impl Decode<RkyvFormat> for protocol::PrivateServiceDeallocateRequestResult {
    type Error = RkyvDeserializationError;

    fn decode(buffer: &[u8]) -> Result<Self, Self::Error> {
        Ok(
            rkyv::from_bytes::<Result<(), InvalidPrivateServiceIdError>>(buffer)
                .map_err(|err| RkyvDeserializationError(err.to_string()))?
                .map_err(Into::into),
        )
    }
}
