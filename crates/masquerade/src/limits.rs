use std::fmt;

const MAX_ITEM_COUNT: usize = 1_000_000;
const MAX_BUFFER_BYTES: usize = 16 * 1_024 * 1_024;

/// Resource limits applied while parsing untrusted files.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValidationLimits {
    bmp: BmpLimits,
    bmff: BmffLimits,
    riff: RiffLimits,
    mp3: Mp3Limits,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct BmpLimits {
    max_width: u32,
    max_height: u32,
    max_pixels: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct BmffLimits {
    max_boxes: usize,
    max_ftyp_bytes: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct RiffLimits {
    max_chunks: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct Mp3Limits {
    max_scan_bytes: usize,
    frames_to_check: usize,
    min_frames: usize,
}

impl ValidationLimits {
    #[must_use]
    pub fn builder() -> ValidationLimitsBuilder {
        ValidationLimitsBuilder::default()
    }

    #[must_use]
    pub const fn max_bmp_width(&self) -> u32 {
        self.bmp.max_width
    }

    #[must_use]
    pub const fn max_bmp_height(&self) -> u32 {
        self.bmp.max_height
    }

    #[must_use]
    pub const fn max_bmp_pixels(&self) -> u64 {
        self.bmp.max_pixels
    }

    #[must_use]
    pub const fn max_bmff_boxes(&self) -> usize {
        self.bmff.max_boxes
    }

    #[must_use]
    pub const fn max_ftyp_bytes(&self) -> usize {
        self.bmff.max_ftyp_bytes
    }

    #[must_use]
    pub const fn max_riff_chunks(&self) -> usize {
        self.riff.max_chunks
    }

    #[must_use]
    pub const fn max_mp3_scan_bytes(&self) -> usize {
        self.mp3.max_scan_bytes
    }

    #[must_use]
    pub const fn mp3_frames_to_check(&self) -> usize {
        self.mp3.frames_to_check
    }

    #[must_use]
    pub const fn mp3_min_frames(&self) -> usize {
        self.mp3.min_frames
    }
}

impl Default for ValidationLimits {
    fn default() -> Self {
        Self {
            bmp: BmpLimits {
                max_width: 32_768,
                max_height: 32_768,
                max_pixels: 100_000_000,
            },
            bmff: BmffLimits {
                max_boxes: 50,
                max_ftyp_bytes: 64 * 1_024,
            },
            riff: RiffLimits { max_chunks: 10_000 },
            mp3: Mp3Limits {
                max_scan_bytes: 65_536,
                frames_to_check: 5,
                min_frames: 2,
            },
        }
    }
}

/// Builder for a validated [`ValidationLimits`] value.
#[derive(Clone, Debug, Default)]
pub struct ValidationLimitsBuilder {
    limits: ValidationLimits,
}

impl ValidationLimitsBuilder {
    #[must_use]
    pub const fn max_bmp_width(mut self, value: u32) -> Self {
        self.limits.bmp.max_width = value;
        self
    }

    #[must_use]
    pub const fn max_bmp_height(mut self, value: u32) -> Self {
        self.limits.bmp.max_height = value;
        self
    }

    #[must_use]
    pub const fn max_bmp_pixels(mut self, value: u64) -> Self {
        self.limits.bmp.max_pixels = value;
        self
    }

    #[must_use]
    pub const fn max_bmff_boxes(mut self, value: usize) -> Self {
        self.limits.bmff.max_boxes = value;
        self
    }

    #[must_use]
    pub const fn max_ftyp_bytes(mut self, value: usize) -> Self {
        self.limits.bmff.max_ftyp_bytes = value;
        self
    }

    #[must_use]
    pub const fn max_riff_chunks(mut self, value: usize) -> Self {
        self.limits.riff.max_chunks = value;
        self
    }

    #[must_use]
    pub const fn max_mp3_scan_bytes(mut self, value: usize) -> Self {
        self.limits.mp3.max_scan_bytes = value;
        self
    }

    #[must_use]
    pub const fn mp3_frames_to_check(mut self, value: usize) -> Self {
        self.limits.mp3.frames_to_check = value;
        self
    }

    #[must_use]
    pub const fn mp3_min_frames(mut self, value: usize) -> Self {
        self.limits.mp3.min_frames = value;
        self
    }

    /// Validate and construct the configured limits.
    ///
    /// # Errors
    ///
    /// Returns an error if a limit could disable validation or make the MP3
    /// frame requirements internally inconsistent.
    pub fn build(self) -> Result<ValidationLimits, InvalidValidationLimits> {
        let limits = self.limits;
        limits.bmp.validate()?;
        limits.bmff.validate()?;
        limits.riff.validate()?;
        limits.mp3.validate()?;
        Ok(limits)
    }
}

impl BmpLimits {
    fn validate(&self) -> Result<(), InvalidValidationLimits> {
        if self.max_width == 0 {
            return Err(InvalidValidationLimits::new(
                "max_bmp_width must be greater than zero",
            ));
        }
        if self.max_height == 0 {
            return Err(InvalidValidationLimits::new(
                "max_bmp_height must be greater than zero",
            ));
        }
        if self.max_pixels == 0 {
            return Err(InvalidValidationLimits::new(
                "max_bmp_pixels must be greater than zero",
            ));
        }
        Ok(())
    }
}

impl BmffLimits {
    fn validate(&self) -> Result<(), InvalidValidationLimits> {
        validate_item_count("max_bmff_boxes", self.max_boxes)?;
        if self.max_ftyp_bytes < 8 {
            return Err(InvalidValidationLimits::new(
                "max_ftyp_bytes must be at least 8",
            ));
        }
        if self.max_ftyp_bytes > MAX_BUFFER_BYTES {
            return Err(InvalidValidationLimits::new(format!(
                "max_ftyp_bytes cannot exceed {MAX_BUFFER_BYTES}"
            )));
        }
        Ok(())
    }
}

impl RiffLimits {
    fn validate(&self) -> Result<(), InvalidValidationLimits> {
        validate_item_count("max_riff_chunks", self.max_chunks)
    }
}

impl Mp3Limits {
    fn validate(&self) -> Result<(), InvalidValidationLimits> {
        validate_item_count("mp3_frames_to_check", self.frames_to_check)?;
        validate_item_count("mp3_min_frames", self.min_frames)?;
        if self.max_scan_bytes < 4 {
            return Err(InvalidValidationLimits::new(
                "max_mp3_scan_bytes must be at least 4",
            ));
        }
        if self.max_scan_bytes > MAX_BUFFER_BYTES {
            return Err(InvalidValidationLimits::new(format!(
                "max_mp3_scan_bytes cannot exceed {MAX_BUFFER_BYTES}"
            )));
        }
        if self.min_frames > self.frames_to_check {
            return Err(InvalidValidationLimits::new(
                "mp3_min_frames cannot exceed mp3_frames_to_check",
            ));
        }
        Ok(())
    }
}

fn validate_item_count(name: &str, value: usize) -> Result<(), InvalidValidationLimits> {
    if value == 0 {
        return Err(InvalidValidationLimits::new(format!(
            "{name} must be greater than zero"
        )));
    }
    if value > MAX_ITEM_COUNT {
        return Err(InvalidValidationLimits::new(format!(
            "{name} cannot exceed {MAX_ITEM_COUNT}"
        )));
    }
    Ok(())
}

/// Error returned for an inconsistent resource-limit configuration.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InvalidValidationLimits {
    reason: String,
}

impl InvalidValidationLimits {
    fn new(reason: impl Into<String>) -> Self {
        Self {
            reason: reason.into(),
        }
    }
}

impl fmt::Display for InvalidValidationLimits {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.reason)
    }
}

impl std::error::Error for InvalidValidationLimits {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_inconsistent_mp3_limits() {
        let result = ValidationLimits::builder()
            .mp3_frames_to_check(2)
            .mp3_min_frames(3)
            .build();
        assert_eq!(
            result.unwrap_err().to_string(),
            "mp3_min_frames cannot exceed mp3_frames_to_check"
        );
    }
}
