//! `WebVTT` file verification by checking the WEBVTT header.

// The WebVTT file begins with:
// | Order | Field          | Bytes | Description                         |
// |-------|----------------|-------|-------------------------------------|
// | 1     | UTF-8 BOM       | 3     | Optional byte order mark           |
// | 2     | "WEBVTT" header | var   | Header line after optional BOM     |
//
// This verifier allows leading whitespace before the WEBVTT marker.

use std::io::{self, Read, Seek};

use crate::{CheckResult, DiagnosticCode, ValidationLimits, io_util::read_prefix};

const HEADER_BYTES: usize = 64;

pub(crate) fn signature<R: Read + Seek + ?Sized>(
    reader: &mut R,
    _limits: &ValidationLimits,
) -> io::Result<bool> {
    Ok(has_webvtt_header(&read_prefix(reader, HEADER_BYTES)?))
}

pub(crate) fn validate_structure<R: Read + Seek + ?Sized>(
    reader: &mut R,
    _limits: &ValidationLimits,
) -> io::Result<CheckResult> {
    let data = read_prefix(reader, HEADER_BYTES)?;
    if data.is_empty() {
        return Ok(CheckResult::invalid(
            DiagnosticCode::EmptyFile,
            "WebVTT file is empty",
        ));
    }
    // TODO: Require the specification's exact signature boundary, UTF-8 and
    // valid blocks/cue timings when Python gains full WebVTT validation.
    Ok(if has_webvtt_header(&data) {
        CheckResult::valid()
    } else {
        CheckResult::invalid(
            DiagnosticCode::MissingWebvttHeader,
            "WebVTT header is missing",
        )
    })
}

fn has_webvtt_header(data: &[u8]) -> bool {
    data.strip_prefix(b"\xef\xbb\xbf")
        .unwrap_or(data)
        .trim_ascii_start()
        .starts_with(b"WEBVTT")
}
