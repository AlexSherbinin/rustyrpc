use core::ops::{Deref, Range};
use std::io;

use crate::transport;

/// Multipart received from stream.
pub struct MultipartReceived {
    buffer: Vec<u8>,
    part_ranges: Vec<Range<usize>>,
}

impl MultipartReceived {
    /// Retrieves part from multipart request.
    #[allow(clippy::undocumented_unsafe_blocks)]
    #[must_use]
    pub fn get_part(&self, index: usize) -> Option<&[u8]> {
        self.iter().nth(index)
    }

    /// Retrieves part from multipart request without check of bounds.
    /// # Safety
    /// Calling this method with an out-of-bounds part index is UB.
    #[must_use]
    pub unsafe fn get_part_unchecked(&self, index: usize) -> &[u8] {
        let buffer_range = self.part_ranges.get_unchecked(index).clone();
        self.get_unchecked(buffer_range)
    }

    pub(crate) async fn receive_from_stream<S: transport::Stream>(
        stream: &mut S,
        part_sizes: &[u32],
    ) -> io::Result<Self> {
        let multipart_buffer_length: u32 = part_sizes.iter().sum();
        let mut multipart_buffer = vec![
            0u8;
            multipart_buffer_length.try_into().map_err(|err| {
                io::Error::new(io::ErrorKind::Other, err)
            })?
        ];
        stream.receive_not_prefixed(&mut multipart_buffer).await?;

        // Not using collect because Iterator::scan not provides capacity via size_hint method.
        let mut current_offset = 0;
        let part_ranges = part_sizes
            .iter()
            .copied()
            .map(|part_size| -> Result<_, io::Error> {
                let part_size: usize = part_size
                    .try_into()
                    .map_err(|err| io::Error::new(io::ErrorKind::Other, err))?;
                let part_range = current_offset..part_size;
                current_offset = current_offset.checked_add(part_size).ok_or_else(|| {
                    io::Error::new(
                        io::ErrorKind::Other,
                        "Overflow occurred while calculation of multipart ranges",
                    )
                })?;
                Ok(part_range)
            })
            .try_collect()?;
        debug_assert!(multipart_buffer.len() == current_offset, "Multipart buffer size not equals to current offset. May cause UB when accessing any part");

        Ok(Self {
            buffer: multipart_buffer,
            part_ranges,
        })
    }

    /// Returns iterator over parts
    #[allow(clippy::undocumented_unsafe_blocks)]
    pub fn iter(&self) -> impl Iterator<Item = &[u8]> {
        self.part_ranges
            .iter()
            .cloned()
            .map(|part_range| unsafe { self.buffer.get_unchecked(part_range) })
    }
}

impl Deref for MultipartReceived {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        &self.buffer
    }
}
