/// Provides functionality for working with the Rkyv library,
/// which is a fast, zero-copy deserialization framework for Rust.
pub mod rkyv;

/// Encoding format like rkyv, bincode, capnproto and others.
pub trait EncodingFormat: Send + Sync + 'static {}

/// Encoding format that supports zero-copy
pub trait ZeroCopyEncodingFormat: EncodingFormat {}

/// A data structure that can be encoded into a specified format.
pub trait Encode<Format: EncodingFormat> {
    /// Encoding error
    type Error: std::error::Error + Send + Sync + 'static;

    /// Encodes data structure into specified format
    ///
    /// # Errors
    ///
    /// Returns an error if encoding fails.
    fn encode(&self) -> Result<Vec<u8>, Self::Error>;
}

/// A data structure that can be decoded from specified format
pub trait Decode<Format: EncodingFormat>
where
    Self: Sized,
{
    /// Decoding error
    type Error: std::error::Error + Send + Sync + 'static;

    /// Decodes data structure from specified format
    ///
    /// # Errors
    /// Returns an error if decoding fails
    fn decode(buffer: &[u8]) -> Result<Self, Self::Error>;
}

/// A data structure that can be decode from specified format but without copying data.
pub trait DecodeZeroCopy<'a, Format: ZeroCopyEncodingFormat, Error>:
    DecodeZeroCopyFallible<Format>
where
    Self: Sized,
{
    /// Decodes data structure from specified format without copying it
    ///
    /// # Errors
    /// Returns an error if decoding fails
    fn decode_zero_copy(buffer: &'a [u8]) -> Result<Self, Error>;
}

/// A data structure that can fail while zero-copy deserialization
pub trait DecodeZeroCopyFallible<Format: ZeroCopyEncodingFormat> {
    /// An error that can occur while zero-copy decoding from specified format
    type Error: std::error::Error + Send + Sync + 'static;
}
