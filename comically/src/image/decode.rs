//! Image decoding

use anyhow::Result;
use imageproc::image::{load_from_memory, DynamicImage};

/// Decode image from memory
///
/// Currently wraps the image crate's load_from_memory.
/// Future: Add fast format-specific decoders (zune-jpeg, png crate direct)
pub fn decode(data: &[u8]) -> Result<DynamicImage> {
    load_from_memory(data).map_err(Into::into)
}
