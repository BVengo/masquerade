//! Structural validation for a deliberately narrow BMP profile.
//!
//! The validator accepts Windows `BITMAPINFOHEADER` files containing only
//! uncompressed 24-bit or 32-bit pixels. It validates the fields that select a
//! decoder path or control offsets and allocation sizes, but it does not decode
//! individual pixels.

use std::io::{self, Read, Seek, SeekFrom};

use crate::{
    CheckResult, DiagnosticCode, ValidationLimits,
    io_util::{read_exact_or_eof, read_prefix},
};

const FILE_HEADER_SIZE: u64 = 14;
const INFO_HEADER_SIZE: u32 = 40;
const PIXEL_OFFSET: u64 = FILE_HEADER_SIZE + INFO_HEADER_SIZE as u64;
const BI_RGB: u32 = 0;

pub(crate) fn signature<R: Read + Seek + ?Sized>(
    reader: &mut R,
    _limits: &ValidationLimits,
) -> io::Result<bool> {
    Ok(read_prefix(reader, 2)?.starts_with(b"BM"))
}

pub(crate) fn validate_structure<R: Read + Seek + ?Sized>(
    reader: &mut R,
    limits: &ValidationLimits,
) -> io::Result<CheckResult> {
    reader.seek(SeekFrom::Start(0))?;

    let mut file_header = [0_u8; FILE_HEADER_SIZE as usize];
    if !read_exact_or_eof(reader, &mut file_header)? {
        return Ok(CheckResult::invalid(
            DiagnosticCode::IncompleteHeader,
            "BMP file header is incomplete",
        ));
    }
    if &file_header[..2] != b"BM" {
        return Ok(CheckResult::invalid(
            DiagnosticCode::InvalidSignature,
            "BMP signature is missing or invalid",
        ));
    }

    let declared_size = u64::from(u32::from_le_bytes(
        file_header[2..6].try_into().expect("four bytes"),
    ));
    let actual_size = reader.seek(SeekFrom::End(0))?;
    if declared_size != actual_size {
        return Ok(CheckResult::invalid(
            DiagnosticCode::FileSizeMismatch,
            "BMP declared file size does not match its actual size",
        ));
    }

    let pixel_offset = u64::from(u32::from_le_bytes(
        file_header[10..14].try_into().expect("four bytes"),
    ));
    if pixel_offset != PIXEL_OFFSET {
        return Ok(CheckResult::invalid(
            DiagnosticCode::InvalidHeaderBounds,
            "BMP pixel offset is not valid for the supported header profile",
        ));
    }

    reader.seek(SeekFrom::Start(FILE_HEADER_SIZE))?;
    let mut info_header = [0_u8; INFO_HEADER_SIZE as usize];
    if !read_exact_or_eof(reader, &mut info_header)? {
        return Ok(CheckResult::invalid(
            DiagnosticCode::IncompleteHeader,
            "BMP info header is incomplete",
        ));
    }

    let info_header_size =
        u32::from_le_bytes(info_header[0..4].try_into().expect("four bytes"));
    if info_header_size != INFO_HEADER_SIZE {
        return Ok(CheckResult::invalid(
            DiagnosticCode::InvalidInfoHeader,
            "BMP does not use the supported 40-byte info header",
        ));
    }

    let width = i32::from_le_bytes(info_header[4..8].try_into().expect("four bytes"));
    let height = i32::from_le_bytes(info_header[8..12].try_into().expect("four bytes"));
    if width <= 0 || height == 0 {
        return Ok(CheckResult::invalid(
            DiagnosticCode::InvalidDimensions,
            "BMP width must be positive and height must be non-zero",
        ));
    }

    let width = u32::try_from(width).expect("positive width");
    let height = height.unsigned_abs();
    let pixels = u64::from(width)
        .checked_mul(u64::from(height))
        .expect("BMP dimensions fit in u64");
    if width > limits.max_bmp_width()
        || height > limits.max_bmp_height()
        || pixels > limits.max_bmp_pixels()
    {
        return Ok(CheckResult::invalid(
            DiagnosticCode::DimensionLimitExceeded,
            "BMP dimensions exceed the configured limits",
        ));
    }

    let planes = u16::from_le_bytes(info_header[12..14].try_into().expect("two bytes"));
    if planes != 1 {
        return Ok(CheckResult::invalid(
            DiagnosticCode::InvalidPlanes,
            "BMP number of color planes is not 1",
        ));
    }

    let bits_per_pixel =
        u16::from_le_bytes(info_header[14..16].try_into().expect("two bytes"));
    if !matches!(bits_per_pixel, 24 | 32) {
        return Ok(CheckResult::invalid(
            DiagnosticCode::InvalidBitDepth,
            "BMP bit depth is not supported; expected 24 or 32 bits per pixel",
        ));
    }

    let compression =
        u32::from_le_bytes(info_header[16..20].try_into().expect("four bytes"));
    if compression != BI_RGB {
        return Ok(CheckResult::invalid(
            DiagnosticCode::InvalidCompression,
            "BMP compression is not supported; expected uncompressed BI_RGB pixels",
        ));
    }

    let image_size = u64::from(u32::from_le_bytes(
        info_header[20..24].try_into().expect("four bytes"),
    ));
    let colors_used =
        u32::from_le_bytes(info_header[32..36].try_into().expect("four bytes"));
    if colors_used != 0 {
        return Ok(CheckResult::invalid(
            DiagnosticCode::InvalidInfoHeader,
            "BMP color tables are not supported for 24-bit or 32-bit images",
        ));
    }

    // BMP rows are padded to a four-byte boundary. Checked arithmetic keeps
    // this safe if the configured dimension limits are raised.
    let row_bits = u64::from(width)
        .checked_mul(u64::from(bits_per_pixel))
        .ok_or_else(|| io::Error::other("BMP row size overflow"))?;
    let row_bytes = row_bits
        .checked_add(31)
        .and_then(|value| value.checked_div(32))
        .and_then(|value| value.checked_mul(4))
        .ok_or_else(|| io::Error::other("BMP row stride overflow"))?;
    let expected_image_size = row_bytes
        .checked_mul(u64::from(height))
        .ok_or_else(|| io::Error::other("BMP pixel array size overflow"))?;

    if image_size != 0 && image_size != expected_image_size {
        return Ok(CheckResult::invalid(
            DiagnosticCode::InvalidPixelData,
            "BMP declared image size does not match its dimensions",
        ));
    }

    let expected_file_size = pixel_offset
        .checked_add(expected_image_size)
        .ok_or_else(|| io::Error::other("BMP file size overflow"))?;
    if expected_file_size != actual_size {
        return Ok(CheckResult::invalid(
            DiagnosticCode::InvalidPixelData,
            "BMP pixel array does not exactly fill the declared file",
        ));
    }

    Ok(CheckResult::valid())
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use super::*;
    use crate::ValidationStatus;

    fn bmp(width: i32, height: i32, bit_depth: u16, compression: u32) -> Vec<u8> {
        let absolute_height = u64::from(height.unsigned_abs());
        let row_bits = u64::from(width.unsigned_abs()) * u64::from(bit_depth);
        let row_bytes = row_bits.div_ceil(32) * 4;
        let image_size = row_bytes * absolute_height;
        let file_size = PIXEL_OFFSET + image_size;

        let mut data = Vec::new();
        data.extend_from_slice(b"BM");
        data.extend_from_slice(&u32::try_from(file_size).unwrap().to_le_bytes());
        data.extend_from_slice(&[0; 4]);
        data.extend_from_slice(&u32::try_from(PIXEL_OFFSET).unwrap().to_le_bytes());
        data.extend_from_slice(&INFO_HEADER_SIZE.to_le_bytes());
        data.extend_from_slice(&width.to_le_bytes());
        data.extend_from_slice(&height.to_le_bytes());
        data.extend_from_slice(&1_u16.to_le_bytes());
        data.extend_from_slice(&bit_depth.to_le_bytes());
        data.extend_from_slice(&compression.to_le_bytes());
        data.extend_from_slice(&u32::try_from(image_size).unwrap().to_le_bytes());
        data.extend_from_slice(&[0; 16]);
        data.resize(usize::try_from(file_size).unwrap(), 0);
        data
    }

    fn validate(data: &[u8]) -> CheckResult {
        validate_structure(&mut Cursor::new(data), &ValidationLimits::default()).unwrap()
    }

    #[test]
    fn accepts_supported_bottom_up_and_top_down_images() {
        assert_eq!(
            validate(&bmp(1, 1, 24, BI_RGB)).status(),
            ValidationStatus::Valid
        );
        assert_eq!(
            validate(&bmp(2, -2, 32, BI_RGB)).status(),
            ValidationStatus::Valid
        );
    }

    #[test]
    fn rejects_invalid_or_excessive_dimensions() {
        let invalid = validate(&bmp(1, 0, 24, BI_RGB));
        assert_eq!(
            invalid.diagnostic().unwrap().code(),
            DiagnosticCode::InvalidDimensions
        );

        let mut data = bmp(2, 2, 24, BI_RGB);
        let limits = ValidationLimits::builder()
            .max_bmp_pixels(3)
            .build()
            .unwrap();
        let excessive = validate_structure(&mut Cursor::new(&mut data), &limits).unwrap();
        assert_eq!(
            excessive.diagnostic().unwrap().code(),
            DiagnosticCode::DimensionLimitExceeded
        );
    }

    #[test]
    fn rejects_decoder_paths_outside_the_supported_profile() {
        assert_eq!(
            validate(&bmp(1, 1, 8, BI_RGB))
                .diagnostic()
                .unwrap()
                .code(),
            DiagnosticCode::InvalidBitDepth
        );
        assert_eq!(
            validate(&bmp(1, 1, 24, 1))
                .diagnostic()
                .unwrap()
                .code(),
            DiagnosticCode::InvalidCompression
        );
    }

    #[test]
    fn rejects_incorrect_pixel_offset_and_extent() {
        let mut offset = bmp(1, 1, 24, BI_RGB);
        offset[10..14].copy_from_slice(&55_u32.to_le_bytes());
        assert_eq!(
            validate(&offset).diagnostic().unwrap().code(),
            DiagnosticCode::InvalidHeaderBounds
        );

        let mut truncated = bmp(1, 1, 24, BI_RGB);
        truncated.pop();
        let size = u32::try_from(truncated.len()).unwrap();
        truncated[2..6].copy_from_slice(&size.to_le_bytes());
        assert_eq!(
            validate(&truncated).diagnostic().unwrap().code(),
            DiagnosticCode::InvalidPixelData
        );
    }

    #[test]
    fn rejects_incorrect_declared_image_size() {
        let mut data = bmp(1, 1, 24, BI_RGB);
        data[34..38].copy_from_slice(&3_u32.to_le_bytes());
        assert_eq!(
            validate(&data).diagnostic().unwrap().code(),
            DiagnosticCode::InvalidPixelData
        );
    }
}
