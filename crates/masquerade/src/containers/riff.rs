//! Resource Interchange File Format (RIFF) container validation.
//!
//! Specs: <https://www.tactilemedia.com/info/MCI_Control_Info.html>

// RIFF is a multimedia container format used by several file types, such
// as AVI and WAV. This module verifies RIFF container headers for specific
// file types.
//
// File structure:
// - header (12 bytes)
// - data (chunks and lists, in any order)
//
// Header:
// | Field         | Bytes | Description                                         |
// |---------------|-------|-----------------------------------------------------|
// | FOURCC code   | 4     | The literal FOURCC identifier 'RIFF'                |
// | file size     | 4     | Size of the file minus 8 bytes. Includes all data   |
// |               |       | past this point, including the file type identifier |
// | file type     | 4     | FOURCC identifier ('AVI ', 'WAVE', etc.)            |
//
// Chunk:
// | Field   | Bytes | Description                                               |
// |---------|-------|-----------------------------------------------------------|
// | ckID    | 4     | FOURCC code identifying the chunk type                    |
// | ckSize  | 4     | Size of the chunk data in bytes                           |
// | ckData  | var   | The actual chunk data                                     |
//
// List Chunks:
// | Field    | Bytes | Description                                              |
// |----------|-------|----------------------------------------------------------|
// | code     | 4     | The literal FOURCC identifier 'LIST'                     |
// | listSize | 4     | Size of the list data in bytes, including listType       |
// | listType | 4     | FOURCC code identifying the type of list                 |
// | listData | var   | The actual list data                                     |

use std::io::{self, Read, Seek, SeekFrom};

use crate::{CheckResult, DiagnosticCode, ValidationLimits, io_util::read_exact_or_eof};

#[derive(Clone, Copy, Debug)]
pub(crate) enum FirstChunkPolicy {
    Any,
    OneOf(&'static [[u8; 4]]),
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct RiffSpec {
    name: &'static str,
    form_type: [u8; 4],
    required_chunks: &'static [[u8; 4]],
    first_chunk: FirstChunkPolicy,
}

impl RiffSpec {
    pub(crate) const fn new(
        name: &'static str,
        form_type: [u8; 4],
        required_chunks: &'static [[u8; 4]],
    ) -> Self {
        Self {
            name,
            form_type,
            required_chunks,
            first_chunk: FirstChunkPolicy::Any,
        }
    }

    pub(crate) const fn with_first_chunk(mut self, policy: FirstChunkPolicy) -> Self {
        self.first_chunk = policy;
        self
    }

    pub(crate) fn has_signature<R: Read + Seek + ?Sized>(
        self,
        reader: &mut R,
        _limits: &ValidationLimits,
    ) -> io::Result<bool> {
        reader.seek(SeekFrom::Start(0))?;
        let mut header = [0_u8; 12];
        Ok(read_exact_or_eof(reader, &mut header)?
            && &header[..4] == b"RIFF"
            && header[8..12] == self.form_type)
    }

    pub(crate) fn validate_structure<R: Read + Seek + ?Sized>(
        self,
        reader: &mut R,
        limits: &ValidationLimits,
    ) -> io::Result<CheckResult> {
        let declared_size = match self.validate_header(reader)? {
            HeaderValidation::Valid(size) => size,
            HeaderValidation::Invalid(result) => return Ok(result),
        };

        reader.seek(SeekFrom::Start(12))?;
        let mut position = 12_u64;
        let mut found = vec![false; self.required_chunks.len()];
        let mut chunks = 0_usize;
        while position < declared_size {
            if chunks == limits.max_riff_chunks() {
                return Ok(CheckResult::invalid(
                    DiagnosticCode::ChunkLimitExceeded,
                    format!("{} contains too many RIFF chunks", self.name),
                ));
            }
            if declared_size - position < 8 {
                return Ok(self.invalid_chunks("incomplete chunk header"));
            }
            let mut chunk = [0_u8; 8];
            reader.read_exact(&mut chunk)?;
            let kind: [u8; 4] = chunk[..4].try_into().expect("four bytes");
            let size = u64::from(u32::from_le_bytes(
                chunk[4..].try_into().expect("four bytes"),
            ));
            position += 8;

            if chunks == 0 && !self.first_chunk.accepts(kind) {
                return Ok(CheckResult::invalid(
                    DiagnosticCode::InvalidImageChunk,
                    format!("Unsupported {} first chunk type: {kind:?}", self.name),
                ));
            }

            let effective_kind = if kind == *b"LIST" {
                if size < 4 || declared_size - position < 4 {
                    return Ok(self.invalid_chunks("invalid LIST chunk size"));
                }
                let mut list_type = [0_u8; 4];
                reader.read_exact(&mut list_type)?;
                list_type
            } else {
                kind
            };
            for (index, required) in self.required_chunks.iter().enumerate() {
                if effective_kind == *required {
                    found[index] = true;
                }
            }

            let padded_size = size.saturating_add(size % 2);
            let Some(next) = position.checked_add(padded_size) else {
                return Ok(self.invalid_chunks("chunk size overflow"));
            };
            if next > declared_size {
                return Ok(self.invalid_chunks("chunk exceeds declared RIFF size"));
            }
            reader.seek(SeekFrom::Start(next))?;
            position = next;
            chunks += 1;
        }

        if chunks == 0 && !matches!(self.first_chunk, FirstChunkPolicy::Any) {
            return Ok(CheckResult::invalid(
                DiagnosticCode::MissingImageChunk,
                format!("{} first chunk is missing", self.name),
            ));
        }
        let missing: Vec<_> = self
            .required_chunks
            .iter()
            .zip(found)
            .filter(|(_, present)| !present)
            .map(|(kind, _)| String::from_utf8_lossy(kind).into_owned())
            .collect();
        if !missing.is_empty() {
            return Ok(CheckResult::invalid(
                DiagnosticCode::MissingRequiredChunk,
                format!(
                    "Required {} chunks were not found: {}",
                    self.name,
                    missing.join(", ")
                ),
            ));
        }
        Ok(CheckResult::valid())
    }

    fn validate_header<R: Read + Seek + ?Sized>(
        self,
        reader: &mut R,
    ) -> io::Result<HeaderValidation> {
        reader.seek(SeekFrom::Start(0))?;
        let mut header = [0_u8; 12];
        if !read_exact_or_eof(reader, &mut header)? || &header[..4] != b"RIFF" {
            return Ok(HeaderValidation::Invalid(CheckResult::invalid(
                DiagnosticCode::InvalidRiffHeader,
                format!("{} RIFF header is missing or invalid", self.name),
            )));
        }
        if header[8..12] != self.form_type {
            return Ok(HeaderValidation::Invalid(CheckResult::invalid(
                DiagnosticCode::WrongRiffType,
                format!(
                    "Expected {} RIFF type, found {:?}",
                    self.name,
                    &header[8..12]
                ),
            )));
        }
        let declared_size = u64::from(u32::from_le_bytes(
            header[4..8].try_into().expect("four bytes"),
        )) + 8;
        let actual_size = reader.seek(SeekFrom::End(0))?;
        if declared_size != actual_size {
            return Ok(HeaderValidation::Invalid(CheckResult::invalid(
                DiagnosticCode::RiffSizeMismatch,
                format!(
                    "{} declared RIFF size does not match its actual size",
                    self.name
                ),
            )));
        }
        Ok(HeaderValidation::Valid(declared_size))
    }

    fn invalid_chunks(self, reason: &str) -> CheckResult {
        CheckResult::invalid(
            DiagnosticCode::InvalidRiffChunks,
            format!("Invalid {} RIFF chunks: {reason}", self.name),
        )
    }
}

impl FirstChunkPolicy {
    fn accepts(self, kind: [u8; 4]) -> bool {
        match self {
            Self::Any => true,
            Self::OneOf(accepted) => accepted.contains(&kind),
        }
    }
}

enum HeaderValidation {
    Valid(u64),
    Invalid(CheckResult),
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use super::*;

    #[test]
    fn first_chunk_policy_rejects_an_unlisted_chunk() {
        const FIRST_CHUNKS: [[u8; 4]; 1] = [*b"GOOD"];
        let spec = RiffSpec::new("test", *b"TEST", &[])
            .with_first_chunk(FirstChunkPolicy::OneOf(&FIRST_CHUNKS));
        let mut data = b"RIFF\x0c\0\0\0TESTBAD!\0\0\0\0".to_vec();
        let result = spec
            .validate_structure(&mut Cursor::new(&mut data), &ValidationLimits::default())
            .unwrap();
        assert_eq!(result.status(), crate::ValidationStatus::Invalid);
    }
}
