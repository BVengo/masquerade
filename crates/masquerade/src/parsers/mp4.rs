//! MP4 structure verification using ISO BMFF box alignment.
//!
//! Spec: <https://raw.githubusercontent.com/OpenAnsible/rust-mp4/master/docs/ISO_IEC_14496-14_2003-11-15.pdf>

use std::io::{self, Read, Seek};

use crate::{CheckResult, ValidationLimits, containers::bmff};

const BRANDS: [[u8; 4]; 2] = [*b"mp41", *b"mp42"];
const REQUIRED_BOXES: [[u8; 4]; 2] = [*b"moov", *b"mdat"];
const SPEC: bmff::BmffSpec = bmff::BmffSpec::new("MP4", &BRANDS, &REQUIRED_BOXES);

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
