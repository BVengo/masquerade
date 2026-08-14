"""BMP file verification by checking the bitmap file header.

The BMP file header has the following structure (14 bytes total):

| Field         | Bytes | Description                                 |
|---------------|-------|---------------------------------------------|
| signature     | 2     | "BM"                                        |
| file size     | 4     | Size of the BMP file in bytes               |
| reserved      | 4     | For future use.                             |
| pixel offset  | 4     | Offset to the pixel data in the file        |

The declared file size must match the actual file size, and the pixel
offset must be at least 14 bytes.
"""

from pathlib import Path

from masquerade.results import CheckResult


def check(path: str | Path) -> CheckResult:
    """Verify BMP by validating the signature and size fields.

    :param path: File path.
    :returns: Structured BMP validation outcome.
    """
    with Path(path).open("rb") as f:
        header = f.read(14)
        if len(header) < 14:
            return CheckResult.rejected(
                "incomplete_header", "BMP file header is incomplete"
            )

        if header[0:2] != b"BM":
            return CheckResult.rejected(
                "invalid_signature", "BMP signature is missing or invalid"
            )

        declared_size = int.from_bytes(header[2:6], "little")
        pixel_offset = int.from_bytes(header[10:14], "little")

        if declared_size < 14 or pixel_offset < 14:
            return CheckResult.rejected(
                "invalid_header_bounds",
                "BMP file size or pixel offset is outside valid bounds",
            )

        f.seek(0, 2)
        actual_size = f.tell()

        if declared_size != actual_size:
            return CheckResult.rejected(
                "file_size_mismatch",
                "BMP declared file size does not match its actual size",
            )
        return CheckResult.accepted()
