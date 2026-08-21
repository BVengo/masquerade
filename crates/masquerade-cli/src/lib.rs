use std::ffi::OsString;
use std::path::PathBuf;

use masquerade::{Inspector, ValidationStatus};

/// Run the command-line interface with the supplied arguments.
///
/// # Errors
///
/// Returns a message when the arguments are invalid or the file cannot be read.
pub fn run(arguments: impl Iterator<Item = OsString>) -> Result<u8, String> {
    let mut path = None;
    let mut signature_only = false;

    for argument in arguments {
        if argument == "--signature-only" {
            signature_only = true;
        } else if argument == "--help" || argument == "-h" {
            println!("Usage: masquerade [--signature-only] <file>");
            return Ok(0);
        } else if path.is_some() {
            return Err("expected exactly one file path".to_owned());
        } else {
            path = Some(PathBuf::from(argument));
        }
    }

    let path = path.ok_or_else(|| "usage: masquerade [--signature-only] <file>".to_owned())?;
    let result = Inspector::new()
        .signature_only(signature_only)
        .inspect(&path)
        .map_err(|error| error.to_string())?;

    match result.status() {
        ValidationStatus::Valid => {
            println!("{}: valid", path.display());
            Ok(0)
        }
        ValidationStatus::Invalid => {
            let Some(failure) = result.failure() else {
                return Err("invalid result did not include a diagnostic".to_owned());
            };
            println!(
                "{}: invalid [{}]: {}",
                path.display(),
                failure.code(),
                failure.reason()
            );
            Ok(1)
        }
        ValidationStatus::Unsupported => {
            let Some(failure) = result.failure() else {
                return Err("unsupported result did not include a diagnostic".to_owned());
            };
            println!(
                "{}: unsupported [{}]: {}",
                path.display(),
                failure.code(),
                failure.reason()
            );
            Ok(1)
        }
    }
}
