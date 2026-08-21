//! WWAV file verification by checking the RIFF container header and chunks.
//!
//! Specs: <https://docs.fileformat.com/audio/wav/>

// Follows the RIFF container structure. The first chunk must be a
// a valid WebP chunk type, such as 'VP8 ', 'VP8L', or 'VP8X'.

use std::io::{self, Read, Seek};

use crate::{CheckResult, ValidationLimits, containers::riff};

const REQUIRED_CHUNKS: [[u8; 4]; 2] = [*b"fmt ", *b"data"];
const SPEC: riff::RiffSpec = riff::RiffSpec::new("WAV", *b"WAVE", &REQUIRED_CHUNKS);

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
