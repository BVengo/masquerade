"""Tests for structured validation outcomes."""

from __future__ import annotations

from typing import TYPE_CHECKING

import pytest

from masquerade import check_file, inspect_file
from masquerade.parsers.jpeg import check as check_jpeg

if TYPE_CHECKING:
    from pathlib import Path


def test_inspect_file_reports_detailed_failure_reason(tmp_path: Path) -> None:
    """Preserve a parser's reason after the signature check succeeds."""
    path = tmp_path / "truncated.jpg"
    path.write_bytes(b"\xff\xd8\xffinvalid")

    result = inspect_file(path)

    assert result.magic.valid is True
    assert result.detailed is not None
    assert result.detailed.valid is False
    assert result.detailed.code == "invalid_end_marker"
    assert result.detailed.reason == (
        "JPEG end-of-image marker is missing or invalid"
    )
    assert result.valid is False


def test_check_file_preserves_boolean_compatibility(tmp_path: Path) -> None:
    """Keep the original tuple API while parsers use structured results."""
    path = tmp_path / "truncated.jpg"
    path.write_bytes(b"\xff\xd8\xffinvalid")

    assert check_file(path) == (True, False)


def test_parser_returns_structured_success(tmp_path: Path) -> None:
    """Return a structured outcome directly from a detailed parser."""
    path = tmp_path / "valid.jpg"
    path.write_bytes(b"\xff\xd8content\xff\xd9")

    result = check_jpeg(path)

    assert result.valid is True
    assert result.code is None
    assert result.reason is None


def test_unsupported_extension_is_undetermined(tmp_path: Path) -> None:
    """Explain both unavailable checks for an unsupported extension."""
    path = tmp_path / "file.unknown"
    path.write_bytes(b"anything")

    result = inspect_file(path)

    assert result.magic.valid is None
    assert result.magic.code == "unsupported_extension"
    assert result.detailed is not None
    assert result.detailed.valid is None
    assert result.detailed.code == "parser_unavailable"
    assert result.valid is None


def test_filesystem_errors_propagate(tmp_path: Path) -> None:
    """Do not classify an inaccessible file as invalid media."""
    path = tmp_path / "missing.jpg"

    with pytest.raises(FileNotFoundError):
        inspect_file(path)
