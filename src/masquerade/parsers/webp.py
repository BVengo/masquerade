"""WebP file verification by checking the RIFF container header.

Specs: https://web.archive.org/web/20251231173058/https://datatracker.ietf.org/doc/rfc9649/

The webp file format is based on the RIFF structure:
| Order | Chunk         | Description                        |
|-------|---------------|------------------------------------|
| 1     | RIFF ('WEBP') | RIFF header with WEBP file type    |
| 2     | WebP Chunk    | Image data (VP8 , VP8L, or VP8X)   |

The first chunk must be a valid WebP chunk type, such as
'VP8 ', 'VP8L', or 'VP8X'.
"""

from pathlib import Path

from masquerade.exceptions import InvalidStructureError
from masquerade.results import CheckResult
from masquerade.utils.riff import (
    ChunkHeader,
    iter_riff_chunks,
    validate_riff_header,
)

_WEBP_CHUNK_TYPES = {b"VP8 ", b"VP8L", b"VP8X"}


def check(path: str | Path) -> CheckResult:
    """Verify WebP container via RIFF header validation.

    :param path: File path.
    :returns: Structured WebP validation outcome.
    """
    with Path(path).open("rb") as f:
        header = validate_riff_header(f)
        if not header:
            return CheckResult.rejected(
                "invalid_riff_header", "WebP RIFF header is missing or invalid"
            )

        if header.file_type != b"WEBP":
            return CheckResult.rejected(
                "wrong_riff_type",
                f"Expected WEBP RIFF type, found {header.file_type!r}",
            )

        try:
            first_chunk = next(
                iter_riff_chunks(f, file_size=header.file_size), None
            )
        except InvalidStructureError as exc:
            return CheckResult.rejected(
                "invalid_riff_chunks", f"Invalid WebP RIFF chunks: {exc}"
            )

        if not isinstance(first_chunk, ChunkHeader):
            return CheckResult.rejected(
                "missing_image_chunk", "WebP image chunk is missing or invalid"
            )

        if first_chunk.ck_id not in _WEBP_CHUNK_TYPES:
            return CheckResult.rejected(
                "invalid_image_chunk",
                f"Unsupported WebP image chunk type: {first_chunk.ck_id!r}",
            )

        return CheckResult.accepted()
