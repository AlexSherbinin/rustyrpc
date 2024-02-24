use core::num::TryFromIntError;

use crate::{
    format::{
        rkyv::{RkyvDeserializationError, RkyvFormat},
        Decode, DecodeZeroCopy, DecodeZeroCopyFallible, Encode,
    },
    impl_decode_zero_copy, server,
};
use alloc::borrow::Cow;
use rkyv::{
    option::ArchivedOption,
    ser::serializers::{
        AlignedSerializer, AllocScratch, CompositeSerializerError, FallbackScratch, HeapScratch,
        SharedSerializeMap,
    },
    with::RefAsBox,
    AlignedVec, Archive, Fallible, Serialize,
};
use thiserror::Error;

#[derive(Serialize, Archive)]
#[archive(check_bytes)]
struct ServiceRef<'a> {
    service_id: u32,
    #[with(RefAsBox)]
    service_checksum: &'a [u8],
}

impl_decode_zero_copy!(ServiceRef<'_> as ArchivedServiceRef<'_>);

impl<'a> TryFrom<&'a ArchivedServiceRef<'a>> for server::ServiceRef {
    type Error = TryFromIntError;

    fn try_from(service_ref: &'a ArchivedServiceRef<'a>) -> Result<Self, Self::Error> {
        Ok(Self {
            service_id: service_ref.service_id.try_into()?,
            service_checksum: Cow::Owned(service_ref.service_checksum.to_vec()),
        })
    }
}

impl<'a> TryFrom<&'a server::ServiceRef> for ServiceRef<'a> {
    type Error = TryFromIntError;

    fn try_from(service_ref: &'a server::ServiceRef) -> Result<Self, Self::Error> {
        Ok(Self {
            service_id: service_ref.service_id.try_into()?,
            service_checksum: &service_ref.service_checksum,
        })
    }
}

type EncodeError = CompositeSerializerError<
    <AlignedSerializer<AlignedVec> as Fallible>::Error,
    <FallbackScratch<HeapScratch<0>, AllocScratch> as Fallible>::Error,
    <SharedSerializeMap as Fallible>::Error,
>;

#[derive(Error, Debug)]
pub enum ServiceRefEncodeError {
    #[error("Service id is invalid")]
    InvalidServiceId(#[from] TryFromIntError),
    #[error(transparent)]
    Encode(#[from] EncodeError),
}

#[derive(Error, Debug)]
pub enum ServiceRefDecodeError {
    #[error("Service id is invalid")]
    InvalidServiceId(#[from] TryFromIntError),
    #[error(transparent)]
    Decode(#[from] RkyvDeserializationError),
}

impl Encode<RkyvFormat> for server::ServiceRef {
    type Error = ServiceRefEncodeError;

    fn encode(&self) -> Result<Vec<u8>, Self::Error> {
        let service_ref: ServiceRef = self.try_into()?;
        Ok(service_ref.encode()?)
    }
}

impl Decode<RkyvFormat> for server::ServiceRef {
    type Error = ServiceRefDecodeError;

    fn decode(buffer: &[u8]) -> Result<Self, Self::Error> {
        Ok(<&ArchivedServiceRef>::decode_zero_copy(buffer)?.try_into()?)
    }
}

impl Encode<RkyvFormat> for Option<server::ServiceRef> {
    type Error = ServiceRefEncodeError;

    fn encode(&self) -> Result<Vec<u8>, Self::Error> {
        let service_ref: Option<ServiceRef> = self
            .as_ref()
            .map_or(Ok(None), |service_ref| service_ref.try_into().map(Some))?;
        Ok(rkyv::to_bytes::<_, 0>(&service_ref)?.to_vec())
    }
}

impl Decode<RkyvFormat> for Option<server::ServiceRef> {
    type Error = ServiceRefDecodeError;

    fn decode(buffer: &[u8]) -> Result<Self, Self::Error> {
        let service_ref = rkyv::check_archived_root::<Option<ServiceRef>>(buffer)
            .map_err(|err| RkyvDeserializationError(err.to_string()))?;
        match service_ref {
            ArchivedOption::Some(service_ref) => Ok(Some(service_ref.try_into()?)),
            ArchivedOption::None => Ok(None),
        }
    }
}
