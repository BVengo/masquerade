"""Python bindings for Masquerade's Rust implementation."""

from masquerade._native import (
    CheckResult,
    ValidationResult,
    ValidationStatus,
    check_file,
    inspect_file,
    main,
    signature_check,
    structure_check,
)

__all__ = [
    "CheckResult",
    "ValidationResult",
    "ValidationStatus",
    "check_file",
    "inspect_file",
    "main",
    "signature_check",
    "structure_check",
]
