"""WAV file verification by checking the RIFF container header and chunks.

Specs found here: https://web.archive.org/web/20241020085723/https://docs.fileformat.com/audio/wav/

The WAV file format is based on the RIFF structure:
| Order | Chunk        | Description                 |
|-------|--------------|-----------------------------|
| 1     | RIFF ('WAVE') | RIFF header with WAVE type  |
| 2     | 'fmt '        | Format chunk (required)     |
| 3     | 'data'        | Audio data chunk (required) |
| 4     | other         | Optional extra chunks       |
"""

from pathlib import Path

from masquerade.exceptions import InvalidStructureError
from masquerade.results import CheckResult
from masquerade.utils.riff import (
    ChunkHeader,
    iter_riff_chunks,
    validate_riff_header,
)


def check(path: str | Path) -> CheckResult:
    """Verify WAV container via RIFF header validation.

    :param path: File path.
    :returns: Structured WAV validation outcome.
    """
    with Path(path).open("rb") as f:
        header = validate_riff_header(f)
        if not header:
            return CheckResult.rejected(
                "invalid_riff_header", "WAV RIFF header is missing or invalid"
            )

        if header.file_type != b"WAVE":
            return CheckResult.rejected(
                "wrong_riff_type",
                f"Expected WAVE RIFF type, found {header.file_type!r}",
            )

        found_fmt = False
        found_data = False

        try:
            for chunk in iter_riff_chunks(f, file_size=header.file_size):
                if isinstance(chunk, ChunkHeader):
                    if chunk.ck_id == b"fmt ":
                        found_fmt = True
                    elif chunk.ck_id == b"data":
                        found_data = True

                if found_fmt and found_data:
                    return CheckResult.accepted()
        except InvalidStructureError as exc:
            return CheckResult.rejected(
                "invalid_riff_chunks", f"Invalid WAV RIFF chunks: {exc}"
            )

        return CheckResult.rejected(
            "missing_required_chunk",
            "Required WAV chunks 'fmt ' and 'data' were not both found",
        )
