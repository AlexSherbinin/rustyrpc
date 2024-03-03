use rkyv::{ser::serializers::AllocSerializer, with::RefAsBox, Archive, Fallible, Serialize};

use crate::{
    format::{
        rkyv::{RkyvDeserializationError, RkyvFormat},
        Decode, DecodeZeroCopy, DecodeZeroCopyFallible, Encode,
    },
    impl_decode_zero_copy, protocol,
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

#[derive(Serialize, Archive)]
#[archive(check_bytes)]
pub enum ServiceCallRequestResult<'a> {
    Ok(#[with(RefAsBox)] &'a [u32]),
    Err(ServiceCallRequestError),
}

impl_decode_zero_copy!(ServiceCallRequestResult<'_> as ArchivedServiceCallRequestResult<'_>);

impl<'a> From<&protocol::ServiceCallRequestResult<'a>> for ServiceCallRequestResult<'a> {
    fn from(value: &protocol::ServiceCallRequestResult<'a>) -> Self {
        match value {
            Ok(part_sizes) => Self::Ok(part_sizes),
            Err(err) => Self::Err(err.into()),
        }
    }
}

impl<'a> From<&'a ArchivedServiceCallRequestResult<'a>> for protocol::ServiceCallRequestResult<'a> {
    fn from(value: &'a ArchivedServiceCallRequestResult) -> Self {
        match value {
            ArchivedServiceCallRequestResult::Ok(part_sizes) => Ok(part_sizes),
            ArchivedServiceCallRequestResult::Err(err) => Err(err.into()),
        }
    }
}

impl<'a> Encode<RkyvFormat> for protocol::ServiceCallRequestResult<'a> {
    type Error = <ServiceCallRequestResult<'a> as Encode<RkyvFormat>>::Error;

    fn encode(&self) -> Result<Vec<u8>, Self::Error> {
        rkyv::to_bytes::<ServiceCallRequestResult, 0>(&self.into()).map(|buffer| buffer.to_vec())
    }
}

impl<'a>
    DecodeZeroCopy<
        'a,
        RkyvFormat,
        <protocol::ServiceCallRequestResult<'_> as DecodeZeroCopyFallible<RkyvFormat>>::Error,
    > for protocol::ServiceCallRequestResult<'a>
{
    fn decode_zero_copy(
        buffer: &'a [u8],
    ) -> Result<
        Self,
        <protocol::ServiceCallRequestResult<'_> as DecodeZeroCopyFallible<RkyvFormat>>::Error,
    > {
        let result: &ArchivedServiceCallRequestResult = DecodeZeroCopy::decode_zero_copy(buffer)?;
        Ok(result.into())
    }
}

impl<'a> DecodeZeroCopyFallible<RkyvFormat> for protocol::ServiceCallRequestResult<'a> {
    type Error =
        <&'a ArchivedServiceCallRequestResult<'a> as DecodeZeroCopyFallible<RkyvFormat>>::Error;
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
