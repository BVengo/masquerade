//! OGG file verification by checking the `OggS` page header.

// The OGG stream begins with an OggS page header:
// | Field            | Bytes | Description                          |
// |------------------|-------|--------------------------------------|
// | "OggS"           | 4     | Capture pattern                      |
// | version          | 1     | Stream structure version (must be 0) |
//
// This verifier checks the capture pattern and stream structure version.

use std::io::{self, Read, Seek, SeekFrom};

use crate::{
    CheckResult, DiagnosticCode, ValidationLimits,
    io_util::{read_exact_or_eof, read_prefix},
};

pub(crate) fn signature<R: Read + Seek + ?Sized>(
    reader: &mut R,
    _limits: &ValidationLimits,
) -> io::Result<bool> {
    Ok(read_prefix(reader, 4)?.starts_with(b"OggS"))
}

pub(crate) fn validate_structure<R: Read + Seek + ?Sized>(
    reader: &mut R,
    _limits: &ValidationLimits,
) -> io::Result<CheckResult> {
    reader.seek(SeekFrom::Start(0))?;
    let mut header = [0_u8; 27];
    if !read_exact_or_eof(reader, &mut header)? {
        return Ok(CheckResult::invalid(
            DiagnosticCode::IncompletePageHeader,
            "OGG page header is incomplete",
        ));
    }
    if &header[..4] != b"OggS" {
        return Ok(CheckResult::invalid(
            DiagnosticCode::InvalidCapturePattern,
            "OGG capture pattern is missing or invalid",
        ));
    }
    // TODO: Validate page flags, lacing tables, body sizes, sequence numbers,
    // logical-stream boundaries and page CRCs when Python gains those checks.
    Ok(if header[4] == 0 {
        CheckResult::valid()
    } else {
        CheckResult::invalid(
            DiagnosticCode::UnsupportedStreamVersion,
            format!("OGG stream structure version {} is unsupported", header[4]),
        )
    })
}
