mod implementation;
mod protocol;

use super::{EncodingFormat, ZeroCopyEncodingFormat};
use thiserror::Error;

/// Represents an error that can occur while deserialization of data structure from `rkyv` format.
#[derive(Error, Debug)]
#[error("{0}")]
pub struct RkyvDeserializationError(String);

/// Represents an `rkyv` format
pub struct RkyvFormat;

impl EncodingFormat for RkyvFormat {}
impl ZeroCopyEncodingFormat for RkyvFormat {}
