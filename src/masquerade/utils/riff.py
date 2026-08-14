"""Parse Resource Interchange File Format (RIFF) containers.

RIFF is a multimedia container format used by several file types, such
as AVI and WAV. This module verifies RIFF container headers for specific
file types.

Specs: https://web.archive.org/web/20260102221641/https://www.tactilemedia.com/info/MCI_Control_Info.html

File structure:
- header (12 bytes)
- data (chunks and lists, in any order)

Header:
| Field         | Bytes | Description                                         |
|---------------|-------|-----------------------------------------------------|
| FOURCC code   | 4     | The literal FOURCC identifier 'RIFF'                |
| file size     | 4     | Size of the file minus 8 bytes. Includes all data   |
|               |       | past this point, including the file type identifier |
| file type     | 4     | FOURCC identifier ('AVI ', 'WAVE', etc.)            |

Chunk:
| Field   | Bytes | Description                                               |
|---------|-------|-----------------------------------------------------------|
| ckID    | 4     | FOURCC code identifying the chunk type                    |
| ckSize  | 4     | Size of the chunk data in bytes                           |
| ckData  | var   | The actual chunk data                                     |

List Chunks:
| Field    | Bytes | Description                                              |
|----------|-------|----------------------------------------------------------|
| code     | 4     | The literal FOURCC identifier 'LIST'                     |
| listSize | 4     | Size of the list data in bytes, including listType       |
| listType | 4     | FOURCC code identifying the type of list                 |
| listData | var   | The actual list data                                     |


"""

from __future__ import annotations

from typing import TYPE_CHECKING, BinaryIO, NamedTuple

from masquerade.exceptions import InvalidStructureError

if TYPE_CHECKING:
    from collections.abc import Iterator


class ChunkHeader(NamedTuple):
    """Parsed RIFF chunk header fields."""

    ck_id: bytes
    ck_size: int


class ListHeader(NamedTuple):
    """Parsed RIFF list header fields."""

    list_type: bytes
    list_size: int


class RiffHeader(NamedTuple):
    """Parsed RIFF main header fields."""

    fourcc: bytes
    file_size: int
    file_type: bytes


def read_riff_header(stream: BinaryIO) -> RiffHeader | None:
    """Read the RIFF main header from a stream.

    :param stream: File stream.
    :returns: RiffHeader if valid, else None.
    """
    stream.seek(0)
    header = stream.read(12)
    if len(header) < 12 or header[0:4] != b"RIFF":
        return None

    return RiffHeader(
        fourcc=header[0:4],
        file_size=int.from_bytes(header[4:8], "little"),
        file_type=header[8:12],
    )


def validate_riff_header(stream: BinaryIO) -> RiffHeader | None:
    """Validate the RIFF header and size bounds.

    :param stream: File stream.
    :returns: RiffHeader if valid, else None.
    """
    header = read_riff_header(stream)
    if header is None:
        return None

    stream.seek(0, 2)
    actual_size = stream.tell()
    declared_size = header.file_size + 8

    if declared_size > actual_size:
        return None

    stream.seek(12)
    return header


def iter_riff_chunks(
    stream: BinaryIO, *, file_size: int
) -> Iterator[ChunkHeader | ListHeader]:
    """Yield RIFF chunk/list headers from a stream.

    :param stream: File stream positioned anywhere; it will be rewound
        to the first chunk.
    :param file_size: RIFF size field from the main header.
    :yields: ChunkHeader or ListHeader entries.
    :raises InvalidStructureError: If a chunk header is incomplete or invalid.
    """
    stream.seek(12)
    total_size = file_size + 8
    bytes_read = 12

    while bytes_read + 8 <= total_size:
        chunk_header = stream.read(8)
        if len(chunk_header) < 8:
            raise InvalidStructureError("Incomplete chunk header")

        ck_id = chunk_header[0:4]
        ck_size = int.from_bytes(chunk_header[4:8], "little")
        bytes_read += 8

        if ck_id == b"LIST":
            if ck_size < 4:
                raise InvalidStructureError("Invalid LIST chunk size")
            list_type_data = stream.read(4)
            if len(list_type_data) < 4:
                raise InvalidStructureError("Incomplete LIST type")
            list_size = ck_size - 4
            yield ListHeader(list_type=list_type_data, list_size=list_size)
            stream.seek(list_size, 1)
            bytes_read += ck_size
        else:
            yield ChunkHeader(ck_id=ck_id, ck_size=ck_size)
            stream.seek(ck_size, 1)
            bytes_read += ck_size

        if ck_size % 2 == 1 and bytes_read < total_size:
            stream.seek(1, 1)
            bytes_read += 1

        if bytes_read > total_size:
            raise InvalidStructureError("Chunk exceeds declared RIFF size")
