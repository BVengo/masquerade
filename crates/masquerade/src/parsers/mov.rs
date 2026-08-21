//! `QuickTime` MOV verification by checking ISO BMFF box alignment.

// The MOV file format is based on ISO BMFF with the following structure:
// | Order | Box  | Description                                  |
// |-------|------|----------------------------------------------|
// | 1     | ftyp | Major/compatible brands include "qt  "       |
// | 2     | moov | Movie metadata (required)                    |
// | 3     | mdat | Media data (required)                        |
// | 4     | other| Optional additional boxes                    |
//
// The first top-level box must be "ftyp", and both "moov" and "mdat"
// must appear.

use std::io::{self, Read, Seek};

use crate::{CheckResult, ValidationLimits, containers::bmff};

const BRANDS: [[u8; 4]; 1] = [*b"qt  "];
const REQUIRED_BOXES: [[u8; 4]; 2] = [*b"moov", *b"mdat"];
const SPEC: bmff::BmffSpec = bmff::BmffSpec::new("MOV", &BRANDS, &REQUIRED_BOXES);

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
