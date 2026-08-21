//! Fast, dependency-free checks of files against their declared type.
//!
//! ```no_run
//! let result = masquerade::inspect("upload.jpg")?;
//! assert!(result.status().is_valid());
//! # Ok::<(), std::io::Error>(())
//! ```
//!
//! Use [`Inspector`] for byte buffers, explicit media types, signature-only
//! checks or custom resource limits.
//!
//! A valid status means the input passed the checks currently implemented for
//! that format. It does not imply complete specification conformance or safety
//! when decoded by another library.

mod containers;
mod inspector;
mod io_util;
mod limits;
mod parsers;
mod result;

use std::io;
use std::path::Path;

pub use inspector::Inspector;
pub use limits::{InvalidValidationLimits, ValidationLimits, ValidationLimitsBuilder};
pub use parsers::{MediaType, UnsupportedMediaType};
pub use result::{CheckResult, Diagnostic, DiagnosticCode, ValidationResult, ValidationStatus};

/// Inspect a file using its extension as the expected media type.
///
/// # Errors
///
/// Returns an I/O error when the file cannot be opened, read or seeked.
pub fn inspect(path: impl AsRef<Path>) -> io::Result<ValidationResult> {
    Inspector::new().inspect(path)
}
