use core::ops::Deref;

use std::io::IoSlice;

use crate::format::{Encode, EncodingFormat};

/// Represents multipart data sendable via stream.
#[derive(Default)]
pub struct MultipartSendable {
    slices: Vec<IoSlice<'static>>,
    capacities: Vec<usize>,
}

impl MultipartSendable {
    /// Creates [`MultipartSendable`] by initializing `Vec`'s under the hood with specified capacity.
    #[must_use]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            slices: Vec::with_capacity(capacity),
            capacities: Vec::with_capacity(capacity),
        }
    }

    /// Push part to multipart.
    pub fn push(&mut self, part: Vec<u8>) {
        let capacity = part.capacity();
        let io_slice = IoSlice::new(part.leak());

        self.slices.push(io_slice);
        self.capacities.push(capacity);
    }

    /// Push part to multipart. Same as [`MultipartSendable::push`] but better for "building" multipart.
    #[must_use]
    pub fn with_part(mut self, part: Vec<u8>) -> Self {
        self.push(part);
        self
    }

    /// Encode as bytes and push to multipart.
    /// # Errors
    /// Returns an error if encoding fails.
    pub fn push_encodable<E: Encode<Format>, Format: EncodingFormat>(
        &mut self,
        encodable: &E,
    ) -> Result<(), E::Error> {
        self.push(encodable.encode()?);
        Ok(())
    }

    /// Encode as bytes and push to multipart. Same as [`MultipartSendable::push_encodable`] but better for "building" multipart.
    /// # Errors
    /// Returns an error if encoding fails
    pub fn with_encodable<E: Encode<Format>, Format: EncodingFormat>(
        mut self,
        encodable: &E,
    ) -> Result<Self, E::Error> {
        self.push_encodable(encodable)?;
        Ok(self)
    }
}

impl Deref for MultipartSendable {
    type Target = [IoSlice<'static>];

    fn deref(&self) -> &Self::Target {
        &self.slices
    }
}

impl Drop for MultipartSendable {
    #[allow(clippy::undocumented_unsafe_blocks)]
    fn drop(&mut self) {
        self.slices
            .iter_mut()
            .zip(self.capacities.iter().copied())
            .for_each(|(slice, capacity)| {
                let slice_ptr = slice.as_ptr().cast_mut();
                unsafe {
                    Vec::from_raw_parts(slice_ptr, slice.len(), capacity);
                }
            });
    }
}

impl<const LENGTH: usize> From<[Vec<u8>; LENGTH]> for MultipartSendable {
    fn from(parts: [Vec<u8>; LENGTH]) -> Self {
        let capacities = parts.iter().map(Vec::capacity).collect();
        let slices = parts
            .into_iter()
            .map(|part| IoSlice::new(part.leak()))
            .collect();

        Self { slices, capacities }
    }
}
