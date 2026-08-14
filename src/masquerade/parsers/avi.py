"""AVI file verification by checking the RIFF container header.

Specs: https://web.archive.org/web/20250703012100/https://learn.microsoft.com/en-us/windows/win32/directshow/avi-riff-file-reference

The AVI file format is based on the RIFF structure:
| Order | Chunk            | Description                          |
|-------|------------------|--------------------------------------|
| 1     | RIFF ('AVI ')     | RIFF header with AVI file type      |
| 2     | LIST ('hdrl')     | Header list (required)              |
| 3     | LIST ('movi')     | Movie data list (required)          |
| 4     | 'idx1'            | Index chunk (optional)              |

It has two manadatory LIST chunks: 'hdrl' (header list) and 'movi'
(movie data). There is an optional index chunk, which we do not require
for validation.

"""

from pathlib import Path

from masquerade.exceptions import InvalidStructureError
from masquerade.results import CheckResult
from masquerade.utils.riff import (
    ListHeader,
    iter_riff_chunks,
    validate_riff_header,
)


def check(path: str | Path) -> CheckResult:
    """Verify AVI container via RIFF header validation.

    :param path: File path.
    :returns: Structured AVI validation outcome.
    """
    with Path(path).open("rb") as f:
        header = validate_riff_header(f)
        if not header:
            return CheckResult.rejected(
                "invalid_riff_header", "AVI RIFF header is missing or invalid"
            )

        # Validate correct file type
        if header.file_type != b"AVI ":
            return CheckResult.rejected(
                "wrong_riff_type",
                f"Expected AVI RIFF type, found {header.file_type!r}",
            )

        found_hdrl = False
        found_movi = False

        try:
            for chunk in iter_riff_chunks(f, file_size=header.file_size):
                if isinstance(chunk, ListHeader):
                    if chunk.list_type == b"hdrl":
                        found_hdrl = True
                    elif chunk.list_type == b"movi":
                        found_movi = True

                if found_hdrl and found_movi:
                    return CheckResult.accepted()
        except InvalidStructureError as exc:
            return CheckResult.rejected(
                "invalid_riff_chunks", f"Invalid AVI RIFF chunks: {exc}"
            )

        return CheckResult.rejected(
            "missing_required_chunk",
            "Required AVI LIST chunks 'hdrl' and 'movi' were not both found",
        )
