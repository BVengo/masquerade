//! BMP file verification by checking the bitmap file header.

// The BMP file header has the following structure (14 bytes total):

// | Field         | Bytes | Description                                 |
// |---------------|-------|---------------------------------------------|
// | signature     | 2     | "BM"                                        |
// | file size     | 4     | Size of the BMP file in bytes               |
// | reserved      | 4     | For future use.                             |
// | pixel offset  | 4     | Offset to the pixel data in the file        |

// The declared file size must match the actual file size, and the pixel
// offset must be at least 14 bytes.

use std::io::{self, Read, Seek, SeekFrom};

use crate::{
    CheckResult, DiagnosticCode, ValidationLimits,
    io_util::{read_exact_or_eof, read_prefix},
};

pub(crate) fn signature<R: Read + Seek + ?Sized>(
    reader: &mut R,
    _limits: &ValidationLimits,
) -> io::Result<bool> {
    Ok(read_prefix(reader, 2)?.starts_with(b"BM"))
}

pub(crate) fn validate_structure<R: Read + Seek + ?Sized>(
    reader: &mut R,
    _limits: &ValidationLimits,
) -> io::Result<CheckResult> {
    reader.seek(SeekFrom::Start(0))?;
    let mut header = [0_u8; 14];
    if !read_exact_or_eof(reader, &mut header)? {
        return Ok(CheckResult::invalid(
            DiagnosticCode::IncompleteHeader,
            "BMP file header is incomplete",
        ));
    }
    if &header[..2] != b"BM" {
        return Ok(CheckResult::invalid(
            DiagnosticCode::InvalidSignature,
            "BMP signature is missing or invalid",
        ));
    }
    let declared = u64::from(u32::from_le_bytes(
        header[2..6].try_into().expect("four bytes"),
    ));
    let pixel_offset = u32::from_le_bytes(header[10..14].try_into().expect("four bytes"));
    if declared < 14 || pixel_offset < 14 {
        return Ok(CheckResult::invalid(
            DiagnosticCode::InvalidHeaderBounds,
            "BMP file size or pixel offset is outside valid bounds",
        ));
    }
    // TODO: Validate the DIB header, dimensions, colour planes, bit depth and
    // pixel-array bounds when the Python implementation gains those checks.
    Ok(if declared == reader.seek(SeekFrom::End(0))? {
        CheckResult::valid()
    } else {
        CheckResult::invalid(
            DiagnosticCode::FileSizeMismatch,
            "BMP declared file size does not match its actual size",
        )
    })
}
