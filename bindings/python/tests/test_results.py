"""Tests for structured validation outcomes."""

from __future__ import annotations

from typing import TYPE_CHECKING

import pytest
from masquerade import (
    CheckResult,
    ValidationResult,
    ValidationStatus,
    check_file,
    inspect_file,
)

if TYPE_CHECKING:
    from pathlib import Path


def test_inspect_file_reports_structure_failure_reason(tmp_path: Path) -> None:
    """Preserve a parser's reason after the signature check succeeds."""
    path = tmp_path / "truncated.jpg"
    path.write_bytes(b"\xff\xd8\xffinvalid")

    result = inspect_file(path)

    assert result.signature.status is ValidationStatus.VALID
    assert result.structure is not None
    assert result.structure.status is ValidationStatus.INVALID
    assert result.structure.code == "invalid_end_marker"
    assert result.structure.reason == (
        "JPEG end-of-image marker is missing or invalid"
    )
    assert result.status is ValidationStatus.INVALID


def test_check_file_preserves_boolean_compatibility(tmp_path: Path) -> None:
    """Keep the original tuple API while parsers use structured results."""
    path = tmp_path / "truncated.jpg"
    path.write_bytes(b"\xff\xd8\xffinvalid")

    assert check_file(path) == (True, False)


def test_native_inspector_returns_structured_success(tmp_path: Path) -> None:
    """Return native structured outcomes from the public inspector."""
    path = tmp_path / "valid.jpg"
    path.write_bytes(b"\xff\xd8\xffcontent\xff\xd9")

    result = inspect_file(path)

    assert isinstance(result, ValidationResult)
    assert isinstance(result.signature, CheckResult)
    assert result.structure is not None
    assert isinstance(result.structure, CheckResult)
    assert result.structure.status is ValidationStatus.VALID
    assert result.structure.code is None
    assert result.structure.reason is None


def test_unsupported_extension_is_explicit(tmp_path: Path) -> None:
    """Explain both unavailable checks for an unsupported extension."""
    path = tmp_path / "file.unknown"
    path.write_bytes(b"anything")

    result = inspect_file(path)

    assert result.signature.status is ValidationStatus.UNSUPPORTED
    assert result.signature.code == "unsupported_extension"
    assert result.structure is not None
    assert result.structure.status is ValidationStatus.UNSUPPORTED
    assert result.structure.code == "parser_unavailable"
    assert result.status is ValidationStatus.UNSUPPORTED


def test_filesystem_errors_propagate(tmp_path: Path) -> None:
    """Do not classify an inaccessible file as invalid media."""
    path = tmp_path / "missing.jpg"

    with pytest.raises(FileNotFoundError):
        inspect_file(path)
