"""PNG file verification by checking the PNG signature and IHDR chunk.

The PNG file structure begins with:
| Order | Field     | Bytes | Description                 |
|-------|-----------|-------|-----------------------------|
| 1     | Signature | 8     | PNG file signature          |
| 2     | IHDR      | 4     | First chunk type (required) |

This verifier checks the signature and ensures the first chunk is IHDR.

TODO: More comprehensive validation. This is effectively just the magic
number check.
"""

from pathlib import Path

from masquerade.results import CheckResult

_PNG_SIGNATURE = b"\x89PNG\r\n\x1a\n"


def check(path: str | Path) -> CheckResult:
    """Verify PNG signature and the IHDR chunk marker.

    :param path: File path.
    :returns: Structured PNG validation outcome.
    """
    with Path(path).open("rb") as f:
        header = f.read(16)
        if len(header) < 16:
            return CheckResult.rejected(
                "incomplete_header",
                "PNG signature or first chunk is incomplete",
            )

        if header[0:8] != _PNG_SIGNATURE:
            return CheckResult.rejected(
                "invalid_signature", "PNG signature is missing or invalid"
            )

        # First chunk must be IHDR.
        if header[12:16] != b"IHDR":
            return CheckResult.rejected(
                "missing_ihdr",
                "PNG first chunk is not the required IHDR chunk",
            )
        return CheckResult.accepted()
