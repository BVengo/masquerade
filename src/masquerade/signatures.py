"""Bounded, dependency-free signature checks for supported file formats."""

from __future__ import annotations

from pathlib import Path
from typing import TYPE_CHECKING

from masquerade.results import CheckResult

if TYPE_CHECKING:
    from collections.abc import Callable

_MAX_PROBE_BYTES = 4096
_RIFF_TYPES = {
    ".avi": b"AVI ",
    ".wav": b"WAVE",
    ".webp": b"WEBP",
}
_BMFF_BRANDS = {
    ".avif": frozenset({b"avif", b"avis"}),
    ".m4a": frozenset({b"M4A ", b"M4B ", b"M4P "}),
    ".mov": frozenset({b"qt  "}),
    ".mp4": frozenset({b"mp41", b"mp42"}),
}


def check_signature(path: str | Path, extension: str) -> bool | None:
    """Check whether a file has the expected signature for its extension.

    The probe reads at most the first 4 KiB. A return value of ``None`` means
    the extension is unsupported, rather than that the file is invalid.

    :param path: File to inspect.
    :param extension: Expected extension, with or without a leading dot.
    :returns: Whether the signature matches, or ``None`` when unsupported.
    """
    return inspect_signature(path, extension).valid


def inspect_signature(path: str | Path, extension: str) -> CheckResult:
    """Check a signature and preserve the reason for a negative outcome."""
    normalized_extension = _normalize_extension(extension)
    matcher = _matchers().get(normalized_extension)
    if matcher is None:
        return CheckResult.undetermined(
            "unsupported_extension",
            f"No signature check is available for {normalized_extension}",
        )

    with Path(path).open("rb") as stream:
        data = stream.read(_MAX_PROBE_BYTES)

    if matcher(data):
        return CheckResult.accepted()
    return CheckResult.rejected(
        "signature_mismatch",
        f"File signature does not match {normalized_extension}",
    )


def _normalize_extension(extension: str) -> str:
    normalized = extension.lower()
    return normalized if normalized.startswith(".") else f".{normalized}"


def _matchers() -> dict[str, Callable[[bytes], bool]]:
    matchers: dict[str, Callable[[bytes], bool]] = {
        ".bmp": lambda data: data.startswith(b"BM"),
        ".jpeg": _is_jpeg,
        ".jpg": _is_jpeg,
        ".mp3": _is_mp3,
        ".ogg": lambda data: data.startswith(b"OggS"),
        ".png": lambda data: data.startswith(b"\x89PNG\r\n\x1a\n"),
        ".vtt": _is_webvtt,
    }
    matchers.update(
        {
            extension: lambda data, expected=riff_type: _is_riff(
                data, expected
            )
            for extension, riff_type in _RIFF_TYPES.items()
        }
    )
    matchers.update(
        {
            extension: lambda data, expected=brands: _has_bmff_brand(
                data, expected
            )
            for extension, brands in _BMFF_BRANDS.items()
        }
    )
    return matchers


def _is_jpeg(data: bytes) -> bool:
    return data.startswith(b"\xff\xd8\xff")


def _is_mp3(data: bytes) -> bool:
    if data.startswith(b"ID3"):
        return (
            len(data) >= 10
            and data[3] in (2, 3, 4)
            and data[4] != 0xFF
            and all(byte < 0x80 for byte in data[6:10])
        )
    if len(data) < 4:
        return False

    header = int.from_bytes(data[:4], "big")
    version = (header >> 19) & 0b11
    layer = (header >> 17) & 0b11
    bitrate = (header >> 12) & 0b1111
    sample_rate = (header >> 10) & 0b11
    return (
        ((header >> 21) & 0x7FF) == 0x7FF
        and version != 0b01
        and layer == 0b01
        and bitrate not in (0, 0b1111)
        and sample_rate != 0b11
    )


def _is_riff(data: bytes, expected_type: bytes) -> bool:
    return (
        len(data) >= 12 and data[:4] == b"RIFF" and data[8:12] == expected_type
    )


def _is_webvtt(data: bytes) -> bool:
    return data.removeprefix(b"\xef\xbb\xbf").lstrip().startswith(b"WEBVTT")


def _has_bmff_brand(data: bytes, expected_brands: frozenset[bytes]) -> bool:
    if len(data) < 16 or data[4:8] != b"ftyp":
        return False

    box_size = int.from_bytes(data[:4], "big")
    header_size = 8
    if box_size == 1:
        if len(data) < 24:
            return False
        box_size = int.from_bytes(data[8:16], "big")
        header_size = 16

    if box_size < header_size + 8 or box_size > len(data):
        return False

    payload = data[header_size:box_size]
    brands = {payload[:4]}
    brands.update(
        payload[index : index + 4] for index in range(8, len(payload), 4)
    )
    return not brands.isdisjoint(expected_brands)
