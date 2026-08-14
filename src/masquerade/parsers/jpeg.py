"""JPEG file verification by checking SOI and EOI markers.

The required JPEG marker structure is:
| Order | Marker | Bytes | Description           |
|-------|--------|-------|-----------------------|
| 1     | SOI    | 2     | Start Of Image (FFD8) |
| 2     | EOI    | 2     | End Of Image (FFD9)   |

The file must begin with the Start Of Image marker and end with the
End Of Image marker.
"""

from pathlib import Path

from masquerade.results import CheckResult


def check(path: str | Path) -> CheckResult:
    """Verify JPEG by validating the SOI and EOI markers.

    :param path: File path.
    :returns: Structured JPEG validation outcome.
    """
    with Path(path).open("rb") as f:
        start = f.read(2)
        if len(start) < 2 or start != b"\xff\xd8":
            return CheckResult.rejected(
                "invalid_start_marker",
                "JPEG start-of-image marker is missing or invalid",
            )

        f.seek(0, 2)
        size = f.tell()
        if size < 4:
            return CheckResult.rejected(
                "file_too_short",
                "JPEG file is too short to contain its required markers",
            )

        f.seek(-2, 2)
        end = f.read(2)
        if end != b"\xff\xd9":
            return CheckResult.rejected(
                "invalid_end_marker",
                "JPEG end-of-image marker is missing or invalid",
            )
        return CheckResult.accepted()
