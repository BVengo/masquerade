//! ISO Base Media File Format (BMFF) top-level box validation.
//!
//! Specs: <https://www.loc.gov/preservation/digital/formats/fdd/fdd000079.shtml>

// The ISO BMFF is a container format used by various media file types,
// including MP4 and AVIF. This module provides utilities to read and
// validate the box structure of BMFF files.
//
// Each BMFF box (atom) has the following header structure:
// | Field      | Bytes | Description                              |
// |------------|-------|------------------------------------------|
// | size       | 4     | Box size (or 1 to signal 64-bit size)    |
// | type       | 4     | Box type (FourCC)                        |
// | largesize  | 8     | Extended size (only if size == 1)        |
// | data       | var   | Box payload                              |
//
// Top-level files are a sequence of boxes, commonly starting with "ftyp".

use std::io::{self, Read, Seek, SeekFrom};

use crate::{
    CheckResult, DiagnosticCode, ValidationLimits,
    io_util::{read_exact_or_eof, read_prefix},
};

const SIGNATURE_PROBE_BYTES: usize = 4_096;

#[derive(Clone, Copy, Debug)]
pub(crate) struct BmffSpec {
    name: &'static str,
    compatible_brands: &'static [[u8; 4]],
    required_boxes: &'static [[u8; 4]],
}

impl BmffSpec {
    pub(crate) const fn new(
        name: &'static str,
        compatible_brands: &'static [[u8; 4]],
        required_boxes: &'static [[u8; 4]],
    ) -> Self {
        Self {
            name,
            compatible_brands,
            required_boxes,
        }
    }

    pub(crate) fn has_signature<R: Read + Seek + ?Sized>(
        self,
        reader: &mut R,
        _limits: &ValidationLimits,
    ) -> io::Result<bool> {
        let data = read_prefix(reader, SIGNATURE_PROBE_BYTES)?;
        if data.len() < 16 || &data[4..8] != b"ftyp" {
            return Ok(false);
        }
        let mut size = u64::from(u32::from_be_bytes(
            data[..4].try_into().expect("four bytes"),
        ));
        let mut header_size = 8_usize;
        if size == 1 {
            if data.len() < 24 {
                return Ok(false);
            }
            size = u64::from_be_bytes(data[8..16].try_into().expect("eight bytes"));
            header_size = 16;
        }
        let Ok(size) = usize::try_from(size) else {
            return Ok(false);
        };
        if size < header_size + 8 || size > data.len() {
            return Ok(false);
        }
        Ok(has_compatible_brand(
            &data[header_size..size],
            self.compatible_brands,
        ))
    }

    pub(crate) fn validate_structure<R: Read + Seek + ?Sized>(
        self,
        reader: &mut R,
        limits: &ValidationLimits,
    ) -> io::Result<CheckResult> {
        let file_size = reader.seek(SeekFrom::End(0))?;
        reader.seek(SeekFrom::Start(0))?;
        let mut boxes = Vec::new();

        while reader.stream_position()? < file_size {
            if boxes.len() == limits.max_bmff_boxes() {
                return Ok(CheckResult::invalid(
                    DiagnosticCode::BoxLimitExceeded,
                    format!(
                        "{} contains more than {} top-level boxes",
                        self.name,
                        limits.max_bmff_boxes()
                    ),
                ));
            }
            let start = reader.stream_position()?;
            let mut header = match read_header(reader, start)? {
                HeaderRead::Complete(header) => header,
                HeaderRead::Incomplete(reason) => {
                    return Ok(CheckResult::invalid(
                        DiagnosticCode::InvalidBoxStructure,
                        format!("Invalid {} box structure: {reason}", self.name),
                    ));
                }
            };
            if header.size == 0 {
                header.size = file_size - start;
            }
            let Some(end) = start.checked_add(header.size) else {
                return Ok(self.invalid_bounds());
            };
            if header.size < header.header_size || end > file_size {
                return Ok(self.invalid_bounds());
            }
            boxes.push(header);
            reader.seek(SeekFrom::Start(end))?;
        }

        let Some(ftyp) = boxes.first() else {
            return Ok(self.missing_ftyp());
        };
        if ftyp.kind != *b"ftyp" {
            return Ok(self.missing_ftyp());
        }
        if !valid_ftyp(
            reader,
            *ftyp,
            self.compatible_brands,
            limits.max_ftyp_bytes(),
        )? {
            return Ok(CheckResult::invalid(
                DiagnosticCode::IncompatibleBrand,
                format!(
                    "{} ftyp box has no supported {} brand",
                    self.name, self.name
                ),
            ));
        }

        let missing: Vec<_> = self
            .required_boxes
            .iter()
            .filter(|required| !boxes.iter().any(|header| &header.kind == *required))
            .map(|kind| String::from_utf8_lossy(kind).into_owned())
            .collect();
        if !missing.is_empty() {
            return Ok(CheckResult::invalid(
                DiagnosticCode::MissingRequiredBox,
                format!(
                    "{} required boxes are missing: {}",
                    self.name,
                    missing.join(", ")
                ),
            ));
        }
        Ok(CheckResult::valid())
    }

    fn invalid_bounds(self) -> CheckResult {
        CheckResult::invalid(
            DiagnosticCode::InvalidBoxStructure,
            format!(
                "Invalid {} box structure: box exceeds file bounds",
                self.name
            ),
        )
    }

    fn missing_ftyp(self) -> CheckResult {
        CheckResult::invalid(
            DiagnosticCode::MissingFtyp,
            format!("{} first box is not the required ftyp box", self.name),
        )
    }
}

#[derive(Clone, Copy, Debug)]
struct BoxHeader {
    size: u64,
    kind: [u8; 4],
    start: u64,
    header_size: u64,
}

enum HeaderRead {
    Complete(BoxHeader),
    Incomplete(&'static str),
}

fn read_header<R: Read + ?Sized>(reader: &mut R, start: u64) -> io::Result<HeaderRead> {
    let mut header = [0_u8; 8];
    if !read_exact_or_eof(reader, &mut header)? {
        return Ok(HeaderRead::Incomplete("incomplete box header"));
    }
    let short_size = u32::from_be_bytes(header[..4].try_into().expect("four bytes"));
    let kind = header[4..].try_into().expect("four bytes");
    let (size, header_size) = if short_size == 1 {
        let mut large = [0_u8; 8];
        if !read_exact_or_eof(reader, &mut large)? {
            return Ok(HeaderRead::Incomplete("incomplete extended box size"));
        }
        (u64::from_be_bytes(large), 16)
    } else {
        (u64::from(short_size), 8)
    };
    Ok(HeaderRead::Complete(BoxHeader {
        size,
        kind,
        start,
        header_size,
    }))
}

fn valid_ftyp<R: Read + Seek + ?Sized>(
    reader: &mut R,
    ftyp: BoxHeader,
    compatible_brands: &[[u8; 4]],
    max_bytes: usize,
) -> io::Result<bool> {
    let payload_size = ftyp.size - ftyp.header_size;
    let Ok(payload_size) = usize::try_from(payload_size) else {
        return Ok(false);
    };
    if payload_size < 8 || payload_size > max_bytes || payload_size % 4 != 0 {
        return Ok(false);
    }
    let mut payload = vec![0; payload_size];
    reader.seek(SeekFrom::Start(ftyp.start + ftyp.header_size))?;
    if !read_exact_or_eof(reader, &mut payload)? {
        return Ok(false);
    }
    Ok(has_compatible_brand(&payload, compatible_brands))
}

fn has_compatible_brand(payload: &[u8], compatible_brands: &[[u8; 4]]) -> bool {
    compatible_brands.iter().any(|brand| {
        payload[..4] == *brand
            || payload[8..]
                .chunks_exact(4)
                .any(|candidate| candidate == brand)
    })
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use super::*;

    #[test]
    fn reports_an_incomplete_header_as_invalid_media() {
        const BRANDS: [[u8; 4]; 1] = [*b"test"];
        let spec = BmffSpec::new("test", &BRANDS, &[]);
        let result = spec
            .validate_structure(&mut Cursor::new(vec![0; 4]), &ValidationLimits::default())
            .unwrap();
        assert_eq!(result.status(), crate::ValidationStatus::Invalid);
    }
}
