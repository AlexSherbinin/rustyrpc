use rkyv::{with::RefAsBox, Archive, Serialize};

use crate::{
    format::{
        rkyv::{RkyvDeserializationError, RkyvFormat},
        DecodeZeroCopy, DecodeZeroCopyFallible, Encode,
    },
    impl_decode_zero_copy, protocol,
};

use super::service_kind::ServiceKind;

#[derive(Serialize, Archive)]
#[archive(check_bytes)]
pub enum RequestKind<'a> {
    ServiceId {
        #[with(RefAsBox)]
        name: &'a str,
        #[with(RefAsBox)]
        checksum: &'a [u8],
    },
    ServiceCall {
        kind: ServiceKind,
        id: u32,
        function_id: u32,
    },
    DeallocatePrivateService {
        id: u32,
    },
}

impl_decode_zero_copy!(RequestKind<'_> as ArchivedRequestKind<'_>);

impl<'a> From<&protocol::RequestKind<'a>> for RequestKind<'a> {
    fn from(value: &protocol::RequestKind<'a>) -> Self {
        match value {
            protocol::RequestKind::ServiceId { name, checksum } => {
                Self::ServiceId { name, checksum }
            }
            protocol::RequestKind::ServiceCall {
                kind,
                id,
                function_id,
            } => Self::ServiceCall {
                kind: (*kind).into(),
                id: *id,
                function_id: *function_id,
            },
            protocol::RequestKind::DeallocatePrivateService { id } => {
                Self::DeallocatePrivateService { id: *id }
            }
        }
    }
}

impl<'a> From<&'a ArchivedRequestKind<'a>> for protocol::RequestKind<'a> {
    fn from(value: &'a ArchivedRequestKind) -> Self {
        match value {
            ArchivedRequestKind::ServiceId { name, checksum } => Self::ServiceId { name, checksum },
            ArchivedRequestKind::ServiceCall {
                kind,
                id,
                function_id,
            } => Self::ServiceCall {
                kind: kind.into(),
                id: *id,
                function_id: *function_id,
            },
            ArchivedRequestKind::DeallocatePrivateService { id } => {
                Self::DeallocatePrivateService { id: *id }
            }
        }
    }
}

impl<'a> Encode<RkyvFormat> for protocol::RequestKind<'a> {
    type Error = <RequestKind<'a> as Encode<RkyvFormat>>::Error;

    fn encode(&self) -> Result<Vec<u8>, Self::Error> {
        let request: RequestKind = self.into();
        request.encode()
    }
}

impl<'a> DecodeZeroCopyFallible<RkyvFormat> for protocol::RequestKind<'a> {
    type Error = <&'a ArchivedRequestKind<'a> as DecodeZeroCopyFallible<RkyvFormat>>::Error;
}

impl<'a>
    DecodeZeroCopy<
        'a,
        RkyvFormat,
        <protocol::RequestKind<'_> as DecodeZeroCopyFallible<RkyvFormat>>::Error,
    > for protocol::RequestKind<'a>
{
    fn decode_zero_copy(
        buffer: &'a [u8],
    ) -> Result<Self, <Self as DecodeZeroCopyFallible<RkyvFormat>>::Error> {
        let archived: &ArchivedRequestKind = DecodeZeroCopy::decode_zero_copy(buffer)?;
        Ok(archived.into())
    }
}
