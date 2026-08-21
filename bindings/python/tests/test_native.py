"""Tests that the Python package cannot fall back to Python validation."""

from __future__ import annotations

import masquerade
from masquerade import ValidationStatus


def test_public_api_is_native() -> None:
    """Export validation functions and result classes from PyO3."""
    assert masquerade.inspect_file.__module__ == "masquerade._native"
    assert masquerade.CheckResult.__module__ == "masquerade._native"
    assert masquerade.ValidationResult.__module__ == "masquerade._native"


def test_validation_status_has_enum_semantics() -> None:
    """Expose stable status members and their compatibility helpers."""
    assert ValidationStatus.VALID.value == "valid"
    assert ValidationStatus.VALID.as_bool() is True
    assert ValidationStatus.INVALID.as_bool() is False
    assert ValidationStatus.UNSUPPORTED.as_bool() is None
