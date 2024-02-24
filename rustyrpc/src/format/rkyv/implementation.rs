use rkyv::{
    de::deserializers::SharedDeserializeMap, ser::serializers::AllocSerializer,
    validation::validators::DefaultValidator, Archive, CheckBytes, Deserialize, Fallible,
    Serialize,
};

use crate::{
    format::{Decode, Encode},
    protocol,
    server::ServiceRef,
};

use super::{RkyvDeserializationError, RkyvFormat};

auto trait DefaultEncode {}

#[allow(suspicious_auto_trait_impls)]
impl !DefaultEncode for protocol::ServiceIdRequestResult {}
#[allow(suspicious_auto_trait_impls)]
impl !DefaultEncode for protocol::ServiceCallRequestResult {}
#[allow(suspicious_auto_trait_impls)]
impl !DefaultEncode for Option<ServiceRef> {}

impl<T> Encode<RkyvFormat> for T
where
    Self: DefaultEncode + Serialize<AllocSerializer<0>>,
{
    type Error = <AllocSerializer<0> as Fallible>::Error;

    fn encode(&self) -> Result<Vec<u8>, Self::Error> {
        Ok(rkyv::to_bytes(self)?.to_vec())
    }
}

impl<T> Decode<RkyvFormat> for T
where
    T: Archive,
    T::Archived: for<'a> CheckBytes<DefaultValidator<'a>> + Deserialize<T, SharedDeserializeMap>,
    Self: DefaultEncode,
{
    type Error = RkyvDeserializationError;

    fn decode(buffer: &[u8]) -> Result<Self, Self::Error> {
        rkyv::from_bytes(buffer).map_err(|err| RkyvDeserializationError(err.to_string()))
    }
}

/// Implements the `DecodeZeroCopy` and `DecodeZeroCopyFallible` traits for a data structures that implement [`Archive`][rkyv::Archive]
#[macro_export]
macro_rules! impl_decode_zero_copy {
    ($root: ty as $archived: ty) => {
        impl<'a> DecodeZeroCopy<'a, RkyvFormat, RkyvDeserializationError> for &'a $archived {
            fn decode_zero_copy(
                buffer: &'a [u8],
            ) -> Result<Self, <Self as DecodeZeroCopyFallible<RkyvFormat>>::Error> {
                rkyv::check_archived_root::<$root>(buffer)
                    .map_err(|err| RkyvDeserializationError(err.to_string()))
            }
        }

        impl DecodeZeroCopyFallible<RkyvFormat> for &$archived {
            type Error = RkyvDeserializationError;
        }
    };
}
