use std::fs::File;
use std::io::{self, Cursor, Read, Seek};
use std::path::Path;

use crate::{
    CheckResult, DiagnosticCode, MediaType, ValidationLimits, ValidationResult, ValidationStatus,
};

/// Configurable media inspector.
#[derive(Clone, Debug, Default)]
pub struct Inspector {
    signature_only: bool,
    limits: ValidationLimits,
}

impl Inspector {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub const fn signature_only(mut self, enabled: bool) -> Self {
        self.signature_only = enabled;
        self
    }

    #[must_use]
    pub fn with_limits(mut self, limits: ValidationLimits) -> Self {
        self.limits = limits;
        self
    }

    /// Inspect a file using its extension as the expected type.
    ///
    /// # Errors
    /// Returns an I/O error when the file cannot be opened, read or seeked.
    pub fn inspect(&self, path: impl AsRef<Path>) -> io::Result<ValidationResult> {
        let path = path.as_ref();
        let Some(media_type) = MediaType::from_path(path) else {
            ensure_readable(path)?;
            return Ok(self.unsupported(path));
        };
        self.inspect_as(path, media_type)
    }

    /// Inspect a file as an explicitly supplied type.
    ///
    /// # Errors
    /// Returns an I/O error when the file cannot be opened, read or seeked.
    pub fn inspect_as(
        &self,
        path: impl AsRef<Path>,
        media_type: MediaType,
    ) -> io::Result<ValidationResult> {
        self.inspect_reader(&mut File::open(path)?, media_type)
    }

    /// Inspect in-memory media as an explicitly supplied type.
    ///
    /// # Errors
    /// Returns an I/O error if the cursor cannot be read or seeked.
    pub fn inspect_bytes(
        &self,
        data: &[u8],
        media_type: MediaType,
    ) -> io::Result<ValidationResult> {
        self.inspect_reader(&mut Cursor::new(data), media_type)
    }

    /// Inspect any seekable stream as an explicitly supplied type.
    ///
    /// # Errors
    /// Returns an I/O error when the stream cannot be read or seeked.
    pub fn inspect_reader<R: Read + Seek + ?Sized>(
        &self,
        reader: &mut R,
        media_type: MediaType,
    ) -> io::Result<ValidationResult> {
        let signature = media_type.check_signature(reader, &self.limits)?;
        if self.signature_only || signature.status() == ValidationStatus::Invalid {
            return Ok(ValidationResult::new(signature, None));
        }
        let structure = media_type.validate_structure(reader, &self.limits)?;
        Ok(ValidationResult::new(signature, Some(structure)))
    }

    /// Run only the structural parser for an explicitly supplied type.
    ///
    /// # Errors
    /// Returns an I/O error when the file cannot be opened, read or seeked.
    pub fn validate_structure(
        &self,
        path: impl AsRef<Path>,
        media_type: MediaType,
    ) -> io::Result<CheckResult> {
        media_type.validate_structure(&mut File::open(path)?, &self.limits)
    }

    fn unsupported(&self, path: &Path) -> ValidationResult {
        let extension = UnsupportedPathExtension::from_path(path);
        ValidationResult::new(
            CheckResult::unsupported(
                DiagnosticCode::UnsupportedExtension,
                unsupported_signature_reason(extension),
            ),
            (!self.signature_only).then(|| {
                CheckResult::unsupported(
                    DiagnosticCode::ParserUnavailable,
                    unsupported_structure_reason(extension),
                )
            }),
        )
    }
}

fn ensure_readable(path: &Path) -> io::Result<()> {
    File::open(path).map(drop)
}

#[derive(Clone, Copy)]
enum UnsupportedPathExtension<'a> {
    Missing,
    NonUnicode,
    Named(&'a str),
}

impl<'a> UnsupportedPathExtension<'a> {
    fn from_path(path: &'a Path) -> Self {
        match path.extension() {
            None => Self::Missing,
            Some(extension) => extension.to_str().map_or(Self::NonUnicode, Self::Named),
        }
    }
}

fn unsupported_signature_reason(extension: UnsupportedPathExtension<'_>) -> String {
    match extension {
        UnsupportedPathExtension::Missing => {
            "No signature check is available for a path without an extension".to_owned()
        }
        UnsupportedPathExtension::NonUnicode => {
            "No signature check is available for a non-Unicode extension".to_owned()
        }
        UnsupportedPathExtension::Named(extension) => {
            format!("No signature check is available for .{extension}")
        }
    }
}

fn unsupported_structure_reason(extension: UnsupportedPathExtension<'_>) -> String {
    match extension {
        UnsupportedPathExtension::Missing => {
            "No structural parser is available for a path without an extension".to_owned()
        }
        UnsupportedPathExtension::NonUnicode => {
            "No structural parser is available for a non-Unicode extension".to_owned()
        }
        UnsupportedPathExtension::Named(extension) => {
            format!("No structural parser is available for extension: {extension}")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unsupported_extensions_have_an_explicit_status() {
        let result = Inspector::new().unsupported(Path::new("file.unknown"));
        assert_eq!(result.status(), ValidationStatus::Unsupported);
        assert_eq!(
            result.signature().diagnostic().unwrap().code(),
            DiagnosticCode::UnsupportedExtension
        );
    }

    #[test]
    fn extensionless_paths_have_a_readable_diagnostic() {
        let result = Inspector::new().unsupported(Path::new("file"));
        assert_eq!(
            result.signature().diagnostic().unwrap().reason(),
            "No signature check is available for a path without an extension"
        );
    }
}
