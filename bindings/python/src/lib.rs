use std::path::PathBuf;

use masquerade::{
    CheckResult as CoreCheckResult, Inspector, MediaType, ValidationResult as CoreValidationResult,
    ValidationStatus as CoreValidationStatus,
};
use pyo3::prelude::*;

#[pyclass(
    frozen,
    eq,
    eq_int,
    hash,
    rename_all = "SCREAMING_SNAKE_CASE",
    skip_from_py_object,
    module = "masquerade._native"
)]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
enum ValidationStatus {
    Valid,
    Invalid,
    Unsupported,
}

#[pymethods]
#[allow(clippy::trivially_copy_pass_by_ref)]
impl ValidationStatus {
    #[getter]
    const fn value(&self) -> &'static str {
        match self {
            Self::Valid => "valid",
            Self::Invalid => "invalid",
            Self::Unsupported => "unsupported",
        }
    }

    fn as_bool(&self) -> Option<bool> {
        match self {
            Self::Valid => Some(true),
            Self::Invalid => Some(false),
            Self::Unsupported => None,
        }
    }

    fn __str__(&self) -> &'static str {
        self.value()
    }
}

impl From<CoreValidationStatus> for ValidationStatus {
    fn from(status: CoreValidationStatus) -> Self {
        match status {
            CoreValidationStatus::Valid => Self::Valid,
            CoreValidationStatus::Invalid => Self::Invalid,
            CoreValidationStatus::Unsupported => Self::Unsupported,
        }
    }
}

#[pyclass(frozen, get_all, skip_from_py_object, module = "masquerade._native")]
#[derive(Clone, Debug)]
struct CheckResult {
    status: ValidationStatus,
    code: Option<String>,
    reason: Option<String>,
}

#[pymethods]
impl CheckResult {
    #[getter]
    fn is_valid(&self) -> bool {
        self.status == ValidationStatus::Valid
    }

    fn __repr__(&self) -> String {
        format!(
            "CheckResult(status=ValidationStatus.{:?}, code={:?}, reason={:?})",
            self.status, self.code, self.reason
        )
    }
}

impl From<&CoreCheckResult> for CheckResult {
    fn from(result: &CoreCheckResult) -> Self {
        let diagnostic = result.diagnostic();
        Self {
            status: result.status().into(),
            code: diagnostic.map(|value| value.code().to_string()),
            reason: diagnostic.map(|value| value.reason().to_owned()),
        }
    }
}

#[pyclass(frozen, skip_from_py_object, module = "masquerade._native")]
#[derive(Clone, Debug)]
struct ValidationResult {
    signature: CheckResult,
    structure: Option<CheckResult>,
    status: ValidationStatus,
}

#[pymethods]
impl ValidationResult {
    #[getter]
    fn signature(&self) -> CheckResult {
        self.signature.clone()
    }

    #[getter]
    fn structure(&self) -> Option<CheckResult> {
        self.structure.clone()
    }

    #[getter]
    fn status(&self) -> ValidationStatus {
        self.status
    }

    fn __repr__(&self) -> String {
        format!(
            "ValidationResult(signature={:?}, structure={:?})",
            self.signature, self.structure
        )
    }
}

impl From<CoreValidationResult> for ValidationResult {
    fn from(result: CoreValidationResult) -> Self {
        Self {
            signature: result.signature().into(),
            structure: result.structure().map(Into::into),
            status: result.status().into(),
        }
    }
}

#[pyfunction]
#[pyo3(signature = (path, *, signature_only = false))]
fn inspect_file(py: Python<'_>, path: PathBuf, signature_only: bool) -> PyResult<ValidationResult> {
    py.detach(move || {
        Inspector::new()
            .signature_only(signature_only)
            .inspect(path)
            .map(Into::into)
    })
    .map_err(PyErr::from)
}

#[pyfunction]
#[pyo3(signature = (path, *, signature_only = false))]
fn check_file(
    py: Python<'_>,
    path: PathBuf,
    signature_only: bool,
) -> PyResult<(Option<bool>, Option<bool>)> {
    let result = py
        .detach(move || {
            Inspector::new()
                .signature_only(signature_only)
                .inspect(path)
        })
        .map_err(PyErr::from)?;
    Ok((
        result.signature().status().as_option(),
        result
            .structure()
            .and_then(|value| value.status().as_option()),
    ))
}

#[pyfunction]
fn signature_check(py: Python<'_>, path: PathBuf, extension: &str) -> PyResult<Option<bool>> {
    let Ok(media_type) = extension.parse::<MediaType>() else {
        return Ok(None);
    };
    let result = py
        .detach(move || {
            Inspector::new()
                .signature_only(true)
                .inspect_as(path, media_type)
        })
        .map_err(PyErr::from)?;
    Ok(result.signature().status().as_option())
}

#[pyfunction]
fn structure_check(py: Python<'_>, path: PathBuf, extension: &str) -> PyResult<Option<bool>> {
    let Ok(media_type) = extension.parse::<MediaType>() else {
        return Ok(None);
    };
    py.detach(move || Inspector::new().validate_structure(path, media_type))
        .map(|result| result.status().as_option())
        .map_err(PyErr::from)
}

#[pyfunction]
fn main(py: Python<'_>) -> PyResult<u8> {
    let arguments = py
        .import("sys")?
        .getattr("argv")?
        .extract::<Vec<PathBuf>>()?;
    Ok(
        match masquerade_cli::run(arguments.into_iter().skip(1).map(PathBuf::into_os_string)) {
            Ok(code) => code,
            Err(message) => {
                eprintln!("masquerade: {message}");
                2
            }
        },
    )
}

#[pymodule]
fn _native(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_class::<ValidationStatus>()?;
    module.add_class::<CheckResult>()?;
    module.add_class::<ValidationResult>()?;
    module.add_function(wrap_pyfunction!(inspect_file, module)?)?;
    module.add_function(wrap_pyfunction!(check_file, module)?)?;
    module.add_function(wrap_pyfunction!(signature_check, module)?)?;
    module.add_function(wrap_pyfunction!(structure_check, module)?)?;
    module.add_function(wrap_pyfunction!(main, module)?)?;
    Ok(())
}
