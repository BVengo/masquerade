"""WebVTT file verification by checking the WEBVTT header.

The WebVTT file begins with:
| Order | Field          | Bytes | Description                         |
|-------|----------------|-------|-------------------------------------|
| 1     | UTF-8 BOM       | 3     | Optional byte order mark           |
| 2     | "WEBVTT" header | var   | Header line after optional BOM     |

This verifier allows leading whitespace before the WEBVTT marker.
TODO: More comprehensive validation. This is effectively just the magic
number check.
"""

from pathlib import Path

from masquerade.results import CheckResult

_UTF8_BOM = b"\xef\xbb\xbf"


def check(path: str | Path) -> CheckResult:
    """Verify WebVTT by validating its header line.

    :param path: File path.
    :returns: Structured WebVTT validation outcome.
    """
    with Path(path).open("rb") as f:
        header = f.read(64)
        if not header:
            return CheckResult.rejected("empty_file", "WebVTT file is empty")

        header = header.removeprefix(_UTF8_BOM)

        # Allow leading whitespace before WEBVTT
        header = header.lstrip()
        if not header.startswith(b"WEBVTT"):
            return CheckResult.rejected(
                "missing_webvtt_header", "WebVTT header is missing"
            )
        return CheckResult.accepted()
