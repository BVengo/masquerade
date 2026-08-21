//! JPEG file verification by required boundary markers.

// The required JPEG marker structure is:
// | Order | Marker | Bytes | Description           |
// |-------|--------|-------|-----------------------|
// | 1     | SOI    | 2     | Start Of Image (FFD8) |
// | 2     | EOI    | 2     | End Of Image (FFD9)   |
//
// The file must begin with the Start Of Image marker and end with the
// End Of Image marker.

use std::io::{self, Read, Seek, SeekFrom};

use crate::{
    CheckResult, DiagnosticCode, ValidationLimits,
    io_util::{read_exact_or_eof, read_prefix},
};

pub(crate) fn signature<R: Read + Seek + ?Sized>(
    reader: &mut R,
    _limits: &ValidationLimits,
) -> io::Result<bool> {
    Ok(read_prefix(reader, 3)?.starts_with(b"\xff\xd8\xff"))
}

pub(crate) fn validate_structure<R: Read + Seek + ?Sized>(
    reader: &mut R,
    _limits: &ValidationLimits,
) -> io::Result<CheckResult> {
    reader.seek(SeekFrom::Start(0))?;
    let mut start = [0_u8; 2];
    if !read_exact_or_eof(reader, &mut start)? || start != *b"\xff\xd8" {
        return Ok(CheckResult::invalid(
            DiagnosticCode::InvalidStartMarker,
            "JPEG start-of-image marker is missing or invalid",
        ));
    }
    if reader.seek(SeekFrom::End(0))? < 4 {
        return Ok(CheckResult::invalid(
            DiagnosticCode::FileTooShort,
            "JPEG file is too short to contain its required markers",
        ));
    }
    reader.seek(SeekFrom::End(-2))?;
    let mut end = [0_u8; 2];
    if !read_exact_or_eof(reader, &mut end)? {
        return Ok(CheckResult::invalid(
            DiagnosticCode::InvalidEndMarker,
            "JPEG end-of-image marker is missing or invalid",
        ));
    }
    // TODO: Walk the marker segments, validate their lengths and parse each
    // entropy-coded scan through EOI when Python performs structural parsing.
    Ok(if end == *b"\xff\xd9" {
        CheckResult::valid()
    } else {
        CheckResult::invalid(
            DiagnosticCode::InvalidEndMarker,
            "JPEG end-of-image marker is missing or invalid",
        )
    })
}
