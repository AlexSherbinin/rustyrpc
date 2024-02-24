use crate::{
    format::{rkyv::RkyvFormat, Decode, Encode},
    protocol,
};

impl Encode<RkyvFormat> for protocol::ServiceFound {
    type Error = <u32 as Encode<RkyvFormat>>::Error;

    fn encode(&self) -> Result<Vec<u8>, Self::Error> {
        self.0.encode()
    }
}

impl Decode<RkyvFormat> for protocol::ServiceFound {
    type Error = <u32 as Decode<RkyvFormat>>::Error;

    fn decode(buffer: &[u8]) -> Result<Self, Self::Error> {
        Ok(Self(u32::decode(buffer)?))
    }
}
