"""Integration tests for supported file types."""

from __future__ import annotations

from pathlib import Path
from struct import pack

import pytest
from masquerade import check_file

ROOT = Path(__file__).resolve().parents[3]
DATA_DIR = ROOT / "data"


SUPPORTED_FILES = {
    "avif": DATA_DIR / "valid_file.avif",
    "avi": DATA_DIR / "valid_file.avi",
    "bmp": DATA_DIR / "valid_file.bmp",
    "jpeg": DATA_DIR / "valid_file.jpeg",
    "jpg": DATA_DIR / "valid_file.jpg",
    "mov": DATA_DIR / "valid_file.mov",
    "mp3": DATA_DIR / "valid_file.mp3",
    "mp4": DATA_DIR / "valid_file.mp4",
    "ogg": DATA_DIR / "valid_file.ogg",
    "png": DATA_DIR / "valid_file.png",
    "wav": DATA_DIR / "valid_file.wav",
    "webp": DATA_DIR / "valid_file.webp",
}


def _write_invalid_fixture(path: Path) -> None:
    """Write an invalid media file fixture."""
    path.write_text("this is not a valid media file\n", encoding="utf-8")


def _write_valid_bmp_fixture(path: Path) -> None:
    """Write a minimal 1x1, 24-bit BMP including row padding."""
    file_header = pack("<2sIHHI", b"BM", 58, 0, 0, 54)
    dib_header = pack("<IiiHHIIiiII", 40, 1, 1, 1, 24, 0, 4, 0, 0, 0, 0)
    path.write_bytes(file_header + dib_header + b"\x00\x00\x00\x00")


@pytest.mark.parametrize(
    ("extension", "fixture_path"), SUPPORTED_FILES.items()
)
def test_supported_types_validate(
    extension: str, fixture_path: Path, tmp_path: Path
) -> None:
    """Test that all supported types validate correctly."""
    if extension == "bmp":
        fixture_path = tmp_path / "valid_file.bmp"
        _write_valid_bmp_fixture(fixture_path)

    signature, structure = check_file(fixture_path)
    assert signature is True, f"Signature check failed for: {extension}"
    assert structure is True, f"Structure check failed for: {extension}"


@pytest.mark.parametrize("extension", SUPPORTED_FILES)
def test_supported_types_reject_invalid(
    extension: str, tmp_path: Path
) -> None:
    """Test that all supported types reject invalid files."""
    invalid_path = tmp_path / f"invalid_file.{extension}"
    _write_invalid_fixture(invalid_path)

    signature, structure = check_file(invalid_path)

    # Either the signature is False, or structure is False when the signature
    # check accepted the invalid file.
    assert signature is False or structure is False, (
        f"Both checks passed for invalid: {extension}"
    )
