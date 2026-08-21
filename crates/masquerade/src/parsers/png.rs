//! PNG file verification by checking the PNG signature and IHDR chunk.

// The PNG file structure begins with:
// | Order | Field     | Bytes | Description                 |
// |-------|-----------|-------|-----------------------------|
// | 1     | Signature | 8     | PNG file signature          |
// | 2     | IHDR      | 4     | First chunk type (required) |
//
// This verifier checks the signature and ensures the first chunk is IHDR.

use std::io::{self, Read, Seek, SeekFrom};

use crate::{
    CheckResult, DiagnosticCode, ValidationLimits,
    io_util::{read_exact_or_eof, read_prefix},
};

const SIGNATURE: &[u8; 8] = b"\x89PNG\r\n\x1a\n";

pub(crate) fn signature<R: Read + Seek + ?Sized>(
    reader: &mut R,
    _limits: &ValidationLimits,
) -> io::Result<bool> {
    Ok(read_prefix(reader, SIGNATURE.len())? == SIGNATURE)
}

pub(crate) fn validate_structure<R: Read + Seek + ?Sized>(
    reader: &mut R,
    _limits: &ValidationLimits,
) -> io::Result<CheckResult> {
    reader.seek(SeekFrom::Start(0))?;
    let mut header = [0_u8; 16];
    if !read_exact_or_eof(reader, &mut header)? {
        return Ok(CheckResult::invalid(
            DiagnosticCode::IncompleteHeader,
            "PNG signature or first chunk is incomplete",
        ));
    }
    if &header[..8] != SIGNATURE {
        return Ok(CheckResult::invalid(
            DiagnosticCode::InvalidSignature,
            "PNG signature is missing or invalid",
        ));
    }
    // TODO: Validate IHDR fields, chunk ordering and lengths, every chunk CRC,
    // required IDAT/IEND chunks, and reject bytes after IEND when Python does.
    Ok(if &header[12..16] == b"IHDR" {
        CheckResult::valid()
    } else {
        CheckResult::invalid(
            DiagnosticCode::MissingIhdr,
            "PNG first chunk is not the required IHDR chunk",
        )
    })
}
