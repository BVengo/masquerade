use std::fmt;

/// The outcome of a validation stage.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ValidationStatus {
    /// The input passed every check performed by the validation stage.
    Valid,
    /// The input failed at least one check performed by the validation stage.
    Invalid,
    /// The input's media type is not supported by the validation stage.
    Unsupported,
}

impl ValidationStatus {
    #[must_use]
    pub const fn is_valid(self) -> bool {
        matches!(self, Self::Valid)
    }

    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Valid => "valid",
            Self::Invalid => "invalid",
            Self::Unsupported => "unsupported",
        }
    }

    /// Convert to a lossy tri-state representation.
    #[must_use]
    pub const fn as_option(self) -> Option<bool> {
        match self {
            Self::Valid => Some(true),
            Self::Invalid => Some(false),
            Self::Unsupported => None,
        }
    }
}

/// A stable, machine-readable validation diagnostic.
///
/// New codes may be added as more formats and checks are implemented.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum DiagnosticCode {
    BoxLimitExceeded,
    ChunkLimitExceeded,
    EmptyFile,
    FileSizeMismatch,
    FileTooShort,
    IncompatibleBrand,
    IncompleteHeader,
    IncompletePageHeader,
    InsufficientValidFrames,
    InvalidBoxStructure,
    InvalidCapturePattern,
    InvalidEndMarker,
    InvalidHeaderBounds,
    InvalidImageChunk,
    InvalidRiffChunks,
    InvalidRiffHeader,
    InvalidSignature,
    InvalidStartMarker,
    MissingFrameHeader,
    MissingFtyp,
    MissingIhdr,
    MissingImageChunk,
    MissingRequiredBox,
    MissingRequiredChunk,
    MissingWebvttHeader,
    ParserUnavailable,
    RiffSizeMismatch,
    SignatureMismatch,
    UnsupportedExtension,
    UnsupportedStreamVersion,
    WrongRiffType,
}

impl DiagnosticCode {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::BoxLimitExceeded => "box_limit_exceeded",
            Self::ChunkLimitExceeded => "chunk_limit_exceeded",
            Self::EmptyFile => "empty_file",
            Self::FileSizeMismatch => "file_size_mismatch",
            Self::FileTooShort => "file_too_short",
            Self::IncompatibleBrand => "incompatible_brand",
            Self::IncompleteHeader => "incomplete_header",
            Self::IncompletePageHeader => "incomplete_page_header",
            Self::InsufficientValidFrames => "insufficient_valid_frames",
            Self::InvalidBoxStructure => "invalid_box_structure",
            Self::InvalidCapturePattern => "invalid_capture_pattern",
            Self::InvalidEndMarker => "invalid_end_marker",
            Self::InvalidHeaderBounds => "invalid_header_bounds",
            Self::InvalidImageChunk => "invalid_image_chunk",
            Self::InvalidRiffChunks => "invalid_riff_chunks",
            Self::InvalidRiffHeader => "invalid_riff_header",
            Self::InvalidSignature => "invalid_signature",
            Self::InvalidStartMarker => "invalid_start_marker",
            Self::MissingFrameHeader => "missing_frame_header",
            Self::MissingFtyp => "missing_ftyp",
            Self::MissingIhdr => "missing_ihdr",
            Self::MissingImageChunk => "missing_image_chunk",
            Self::MissingRequiredBox => "missing_required_box",
            Self::MissingRequiredChunk => "missing_required_chunk",
            Self::MissingWebvttHeader => "missing_webvtt_header",
            Self::ParserUnavailable => "parser_unavailable",
            Self::RiffSizeMismatch => "riff_size_mismatch",
            Self::SignatureMismatch => "signature_mismatch",
            Self::UnsupportedExtension => "unsupported_extension",
            Self::UnsupportedStreamVersion => "unsupported_stream_version",
            Self::WrongRiffType => "wrong_riff_type",
        }
    }
}

impl fmt::Display for DiagnosticCode {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

/// Details about an invalid or unavailable validation stage.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Diagnostic {
    code: DiagnosticCode,
    reason: String,
}

impl Diagnostic {
    #[must_use]
    pub fn new(code: DiagnosticCode, reason: impl Into<String>) -> Self {
        Self {
            code,
            reason: reason.into(),
        }
    }

    #[must_use]
    pub const fn code(&self) -> DiagnosticCode {
        self.code
    }

    #[must_use]
    pub fn reason(&self) -> &str {
        &self.reason
    }
}

/// Outcome of one validation stage.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CheckResult {
    Valid,
    Invalid(Diagnostic),
    Unsupported(Diagnostic),
}

impl CheckResult {
    #[must_use]
    pub const fn valid() -> Self {
        Self::Valid
    }

    #[must_use]
    pub fn invalid(code: DiagnosticCode, reason: impl Into<String>) -> Self {
        Self::Invalid(Diagnostic::new(code, reason))
    }

    #[must_use]
    pub fn unsupported(code: DiagnosticCode, reason: impl Into<String>) -> Self {
        Self::Unsupported(Diagnostic::new(code, reason))
    }

    #[must_use]
    pub const fn status(&self) -> ValidationStatus {
        match self {
            Self::Valid => ValidationStatus::Valid,
            Self::Invalid(_) => ValidationStatus::Invalid,
            Self::Unsupported(_) => ValidationStatus::Unsupported,
        }
    }

    #[must_use]
    pub const fn diagnostic(&self) -> Option<&Diagnostic> {
        match self {
            Self::Valid => None,
            Self::Invalid(diagnostic) | Self::Unsupported(diagnostic) => Some(diagnostic),
        }
    }
}

/// Combined signature and structural validation outcomes.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValidationResult {
    signature: CheckResult,
    structure: Option<CheckResult>,
}

impl ValidationResult {
    pub(crate) const fn new(signature: CheckResult, structure: Option<CheckResult>) -> Self {
        Self {
            signature,
            structure,
        }
    }

    #[must_use]
    pub const fn signature(&self) -> &CheckResult {
        &self.signature
    }

    #[must_use]
    pub const fn structure(&self) -> Option<&CheckResult> {
        self.structure.as_ref()
    }

    #[must_use]
    pub fn status(&self) -> ValidationStatus {
        self.structure
            .as_ref()
            .map_or_else(|| self.signature.status(), CheckResult::status)
    }

    /// Return the most authoritative diagnostic, if validation did not succeed.
    #[must_use]
    pub fn failure(&self) -> Option<&Diagnostic> {
        self.structure
            .as_ref()
            .unwrap_or(&self.signature)
            .diagnostic()
    }

    /// Consume the report and return its signature and structural outcomes.
    #[must_use]
    pub fn into_parts(self) -> (CheckResult, Option<CheckResult>) {
        (self.signature, self.structure)
    }
}
