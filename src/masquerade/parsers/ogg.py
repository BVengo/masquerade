"""OGG file verification by checking the OggS page header.

The OGG stream begins with an OggS page header:
| Field            | Bytes | Description                          |
|------------------|-------|--------------------------------------|
| "OggS"           | 4     | Capture pattern                      |
| version          | 1     | Stream structure version (must be 0) |

This verifier checks the capture pattern and stream structure version.
"""

from pathlib import Path

from masquerade.results import CheckResult


def check(path: str | Path) -> CheckResult:
    """Verify OGG container by validating the OggS capture pattern.

    :param path: File path.
    :returns: Structured OGG validation outcome.
    """
    with Path(path).open("rb") as f:
        header = f.read(27)
        if len(header) < 27:
            return CheckResult.rejected(
                "incomplete_page_header", "OGG page header is incomplete"
            )

        if header[0:4] != b"OggS":
            return CheckResult.rejected(
                "invalid_capture_pattern",
                "OGG capture pattern is missing or invalid",
            )

        # OGG stream structure version must be 0.
        if header[4] != 0:
            return CheckResult.rejected(
                "unsupported_stream_version",
                f"OGG stream structure version {header[4]} is unsupported",
            )
        return CheckResult.accepted()
