"""ISO Base Media File Format (BMFF) helpers for parsing top-level boxes.

Specs: https://www.loc.gov/preservation/digital/formats/fdd/fdd000079.shtml

The ISO BMFF is a container format used by various media file types,
including MP4 and AVIF. This module provides utilities to read and
validate the box structure of BMFF files.

Each BMFF box (atom) has the following header structure:
| Field      | Bytes | Description                              |
|------------|-------|------------------------------------------|
| size       | 4     | Box size (or 1 to signal 64-bit size)    |
| type       | 4     | Box type (FourCC)                        |
| largesize  | 8     | Extended size (only if size == 1)        |
| data       | var   | Box payload                              |

Top-level files are a sequence of boxes, commonly starting with "ftyp".
"""

from __future__ import annotations

from typing import TYPE_CHECKING, BinaryIO, NamedTuple

from masquerade.exceptions import InvalidStructureError

if TYPE_CHECKING:
    from collections.abc import Iterable, Iterator


class BmffBox(NamedTuple):
    """Represents a parsed BMFF box header."""

    size: int
    type: str
    start: int
    header_size: int


def iter_top_level_boxes(
    stream: BinaryIO, max_boxes: int
) -> Iterator[BmffBox]:
    """Iterate through top-level BMFF boxes.

    :param stream: File stream.
    :param max_boxes: Limit to avoid infinite loops on corrupt files.
    """
    stream.seek(0)
    file_size = _file_size(stream)
    count = 0

    while stream.tell() < file_size and count < max_boxes:
        start = stream.tell()
        size, box_type, header_size = read_box_header(stream)
        if size < header_size or start + size > file_size:
            break

        yield BmffBox(
            size=size, type=box_type, start=start, header_size=header_size
        )

        stream.seek(start + size)
        count += 1


def read_box_header(stream: BinaryIO) -> tuple[int, str, int]:
    """Read BMFF box header.

    :returns: (box_size, box_type, header_size)
    """
    header = stream.read(8)
    if len(header) < 8:
        raise InvalidStructureError("Incomplete box header")

    size = int.from_bytes(header[:4], "big")
    box_type = header[4:8].decode("ascii", errors="replace")
    header_size = 8

    if size == 1:
        largesize_data = stream.read(8)
        if len(largesize_data) < 8:
            raise InvalidStructureError("Incomplete extended box size")
        largesize = int.from_bytes(largesize_data, "big")
        size = largesize
        header_size = 16

    return size, box_type, header_size


def valid_ftyp(
    stream: BinaryIO, box: BmffBox, required_brands: Iterable[str]
) -> bool:
    """Validate the ftyp box brands.

    :param stream: File stream.
    :param box: ftyp box info.
    :param required_brands: Brands that must appear as major or
        compatible.
    """
    stream.seek(box.start + box.header_size)
    data = stream.read(box.size - box.header_size)

    if len(data) < 8:
        return False

    major_brand = data[0:4].decode("ascii", errors="ignore")
    compatible = [
        data[i : i + 4].decode("ascii", errors="ignore")
        for i in range(8, len(data), 4)
    ]

    required = set(required_brands)
    if major_brand in required:
        return True

    return any(brand in required for brand in compatible)


def _file_size(stream: BinaryIO) -> int:
    """Return total file size.

    :param stream: File stream.
    :returns: Size in bytes.
    """
    current = stream.tell()
    stream.seek(0, 2)
    size = stream.tell()
    stream.seek(current)
    return size
