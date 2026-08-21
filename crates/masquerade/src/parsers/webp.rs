//! WebP file verification by checking the RIFF container header.
//!
//! Specs: <https://datatracker.ietf.org/doc/rfc9649/>

// The webp file format is based on the RIFF structure:
// | Order | Chunk         | Description                        |
// |-------|---------------|------------------------------------|
// | 1     | RIFF ('WEBP') | RIFF header with WEBP file type    |
// | 2     | WebP Chunk    | Image data (VP8 , VP8L, or VP8X)   |
//
// The first chunk must be a valid WebP chunk type, such as
// 'VP8 ', 'VP8L', or 'VP8X'.

use std::io::{self, Read, Seek};

use crate::{CheckResult, ValidationLimits, containers::riff};

const IMAGE_CHUNKS: [[u8; 4]; 3] = [*b"VP8 ", *b"VP8L", *b"VP8X"];
const SPEC: riff::RiffSpec = riff::RiffSpec::new("WebP", *b"WEBP", &[])
    .with_first_chunk(riff::FirstChunkPolicy::OneOf(&IMAGE_CHUNKS));

pub(crate) fn signature<R: Read + Seek + ?Sized>(
    reader: &mut R,
    limits: &ValidationLimits,
) -> io::Result<bool> {
    SPEC.has_signature(reader, limits)
}

pub(crate) fn validate_structure<R: Read + Seek + ?Sized>(
    reader: &mut R,
    limits: &ValidationLimits,
) -> io::Result<CheckResult> {
    SPEC.validate_structure(reader, limits)
}
