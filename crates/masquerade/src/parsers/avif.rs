//! AVIF verification by checking ISO BMFF box alignment and brands.
//!
//! Specs: <https://www.loc.gov/preservation/digital/formats/fdd/fdd000540.shtml>

// The AVIF file format is based on the ISO Base Media File Format (BMFF),
// with specific requirements for box structure and brands.
//
// The essential boxes for AVIF include:
// - ftyp: File type box, must indicate AVIF compatibility.
// - meta: Metadata box, required for AVIF files.

use std::io::{self, Read, Seek};

use crate::{CheckResult, ValidationLimits, containers::bmff};

const BRANDS: [[u8; 4]; 2] = [*b"avif", *b"avis"];
const REQUIRED_BOXES: [[u8; 4]; 1] = [*b"meta"];
const SPEC: bmff::BmffSpec = bmff::BmffSpec::new("AVIF", &BRANDS, &REQUIRED_BOXES);

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
