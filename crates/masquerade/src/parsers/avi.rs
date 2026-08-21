//! AVI file verification by checking the RIFF container structure.
//!
//! Specs: <https://learn.microsoft.com/en-us/windows/win32/directshow/avi-riff-file-reference>

// The AVI file format is based on the RIFF structure:
// | Chunk          | Description                     | Required |
// |----------------|---------------------------------|----------|
// | RIFF ('AVI ')  | RIFF container with AVI type    | Yes      |
// | LIST ('hdrl')  | Header list                     | Yes      |
// | LIST ('movi')  | Movie data list                 | Yes      |
// | 'idx1'         | Legacy index chunk              | No       |
//
// It has two mandatory LIST chunks: 'hdrl' (header list) and 'movi'
// (movie data). There is an optional index chunk, which we do not require
// for validation.

use std::io::{self, Read, Seek};

use crate::{CheckResult, ValidationLimits, containers::riff};

const REQUIRED_CHUNKS: [[u8; 4]; 2] = [*b"hdrl", *b"movi"];
const SPEC: riff::RiffSpec = riff::RiffSpec::new("AVI", *b"AVI ", &REQUIRED_CHUNKS);

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
