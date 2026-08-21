"""Tests for dependency-free file signature detection."""

from __future__ import annotations

from struct import pack
from typing import TYPE_CHECKING

import pytest
from masquerade import signature_check

if TYPE_CHECKING:
    from pathlib import Path


def _bmff(brand: bytes) -> bytes:
    return pack(">I4s4sI4s", 20, b"ftyp", brand, 0, brand)


SIGNATURES = {
    "avif": _bmff(b"avif"),
    "avi": b"RIFF\x04\x00\x00\x00AVI ",
    "bmp": b"BM",
    "jpeg": b"\xff\xd8\xff",
    "jpg": b"\xff\xd8\xff",
    "m4a": _bmff(b"M4A "),
    "mov": _bmff(b"qt  "),
    "mp3": b"ID3\x04\x00\x00\x00\x00\x00\x00",
    "mp4": _bmff(b"mp42"),
    "ogg": b"OggS",
    "png": b"\x89PNG\r\n\x1a\n",
    "vtt": b"\xef\xbb\xbf  WEBVTT\n",
    "wav": b"RIFF\x04\x00\x00\x00WAVE",
    "webp": b"RIFF\x04\x00\x00\x00WEBP",
}


@pytest.mark.parametrize(("extension", "signature"), SIGNATURES.items())
def test_supported_signature_matches(
    extension: str, signature: bytes, tmp_path: Path
) -> None:
    """Recognize every supported extension's representative signature."""
    path = tmp_path / f"file.{extension}"
    path.write_bytes(signature)

    assert signature_check(path, extension) is True


def test_mismatched_signature_is_rejected(tmp_path: Path) -> None:
    """Reject valid signatures when they do not match the extension."""
    path = tmp_path / "file.jpg"
    path.write_bytes(SIGNATURES["png"])

    assert signature_check(path, ".jpg") is False


def test_unsupported_extension_maps_to_none_in_boolean_api(
    tmp_path: Path,
) -> None:
    """Map unsupported formats to None in the lossy boolean API."""
    path = tmp_path / "file.unknown"
    path.write_bytes(b"anything")

    assert signature_check(path, ".unknown") is None
