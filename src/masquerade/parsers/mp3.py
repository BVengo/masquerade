"""MP3 file structure verification by checking frame alignment.

Specs: https://web.archive.org/web/20231125190134/https://mp3guessenc.sourceforge.io/MPEG%20Layer3%20Bitstream%20Syntax%20and%20Decoding.pdf

An MP3 file is composed of a sequence of frames, each starting with a
4-byte header that encodes metadata about the frame, including bitrate,
sampling rate, and padding. This module reads the file, locates a number
of these headers, and verifies that the frames are correctly aligned
according to the MP3 specification.

Audio frame structure:
- header (4 bytes)
- error check (optional, 17 or 32 bytes)
- audio data (variable length)

Header structure:
| Field          | Bits | Description                                         |
|----------------|------|-----------------------------------------------------|
| sync           | 11   | Frame sync (all bits set to 1)                      |
| IDex           | 1    | Extended algorithm ID (1=MPEG1 or MPEG2, 0=MPEG2.5) |
| ID             | 1    | ID of the algorithm (1=MPEG1, 0=MPEG2 or MPEG2.5)   |
| Layer          | 2    | Indicate which layer is used.                       |
| protection bit | 1    | Protection bit (0 if protected by CRC)              |
| bitrate index  | 4    | Bitrate index - all 0 indicates free format         |
| sampling freq  | 2    | Sampling frequency index                            |
| padding bit    | 1    | Padding bit - if 1, frame has an additional slot    |
| private bit    | 1    | Private bit - private use, not used by ISO/IEC      |
| mode           | 2    | Channel mode (stereo, joint, dual, single)          |
| mode extension | 2    | Mode extension (only if Joint stereo)               |
| copyright      | 1    | Copyright (1 if protected by copyright)             |
| original       | 1    | 0 if a copy, and 1 if an original                   |
| emphasis       | 2    | The type of de-emphasis that shall be used          |

"""

from __future__ import annotations

from typing import TYPE_CHECKING, BinaryIO, NamedTuple

from masquerade.results import CheckResult

if TYPE_CHECKING:
    from pathlib import Path


class MpegHeader(NamedTuple):
    """Parsed MPEG audio frame header fields."""

    version_id: int
    layer_id: int
    bitrate_index: int
    sampling_rate_index: int
    padding_bit: int


# fmt: off
BITRATE_TABLE = {
    # version_id: {layer_id: [bitrate_kbps by index]}
    # Index `0` is freeformat, index `15` is forbidden
    # We forbid freeformat because it is poorly supported and very rare.
    0b00: {  # MPEG-2.5
        0b01: [
            None, 8, 16, 24, 32, 40, 48, 56, 64,
            None, None, None, None, None, None, None,
        ],
    },
    # 0b01: { reserved },
    0b10: {  # MPEG-2
        0b01: [
            None, 8, 16, 24, 32, 40, 48, 56, 64,
            80, 96, 112, 128, 144, 160, None,
        ],
    },
    0b11: {  # MPEG-1
        0b01: [
            None, 32, 40, 48, 56, 64, 80, 96, 112,
            128, 160, 192, 224, 256, 320, None,
        ],
    },
}
# fmt: on

# 'None' is a reserved value
SAMPLERATE_TABLE = {
    0b00: [11025, 12000, 8000, None],  # MPEG-2.5
    0b10: [22050, 24000, 16000, None],  # MPEG-2
    0b11: [44100, 48000, 32000, None],  # MPEG-1
}


def check(
    path: str | Path,
    *,
    frames_to_check: int = 5,
    min_frames: int = 2,
) -> CheckResult:
    """Verify MP3 by checking consecutive frame alignment.

    The function attempts to validate up to `frames_to_check` frames
    but will accept the file if at least `min_frames` consecutive valid
    frames are found before EOF.

    :param path: Path to file.
    :param frames_to_check: Target number of frames to validate. 5 is a
        good balance without being too slow.
    :param min_frames: Minimum consecutive valid frames required, in
        case of small files that don't have `frames_to_check` frames.
    :returns: Structured MP3 validation outcome.
    """
    with path.open("rb") as f:
        start = _skip_id3v2(f)
        first_header_pos = _find_first_header(f, start)
        if first_header_pos is None:
            return CheckResult.rejected(
                "missing_frame_header",
                "No valid MPEG Layer III frame header was found",
            )

        valid = _validate_consecutive_frames(
            f,
            first_header_pos,
            frames_to_check=frames_to_check,
            min_frames=min_frames,
        )
        if not valid:
            return CheckResult.rejected(
                "insufficient_valid_frames",
                f"Fewer than {min_frames} consecutive valid MP3 frames found",
            )
        return CheckResult.accepted()


def _validate_consecutive_frames(
    stream: BinaryIO,
    pos: int,
    *,
    frames_to_check: int,
    min_frames: int,
) -> bool:
    """Validate sequential MP3 frames and stop at EOF.

    :param stream: File stream.
    :param pos: Byte position of first frame.
    :param frames_to_check: Max frames to attempt.
    :param min_frames: Minimum required for success.
    :returns: True if enough valid frames are found.
    """
    verified = 0

    for _ in range(frames_to_check):
        stream.seek(pos)
        header_bytes = stream.read(4)
        if len(header_bytes) < 4:
            break  # EOF reached

        word = int.from_bytes(header_bytes, "big")
        if not _is_valid_header(word):
            break

        header = _parse_header(word)
        frame_len = _frame_length(header)
        if frame_len <= 0:
            break

        verified += 1
        pos += frame_len

    return verified >= min_frames


def _skip_id3v2(stream: BinaryIO) -> int:
    """Skip ID3v2 tag if present and return first audio byte offset.

    ID3v2 tags are flexible metadata containers located at the start of
    MP3 files. They begin with the "ID3" identifier followed by a
    version, flags, and a size field that indicates the total tag size.
    This function reads the ID3v2 header, calculates the tag size, and
    returns the byte offset where the actual audio data begins.

    :param stream: File stream.
    :returns: Byte offset of first audio data after ID3v2 tag.
    """
    stream.seek(0)
    header = stream.read(10)

    # Check for "ID3" identifier. If not present, return start of file.
    if len(header) < 10 or header[:3] != b"ID3":
        stream.seek(0)
        return 0

    # Extract size from synchsafe integer (4 bytes, at offset 6)
    size = _synchsafe_to_int(header[6:10])
    stream.seek(10 + size)
    return 10 + size


def _synchsafe_to_int(b: bytes) -> int:
    """Convert 4-byte synchsafe integer to standard int.

    Synchsafe ints use only 7 bits of each byte to avoid false syncs.

    :param b: 4-byte synchsafe integer.
    :returns: Converted integer.
    """
    return (
        ((b[0] & 0x7F) << 21)
        | ((b[1] & 0x7F) << 14)
        | ((b[2] & 0x7F) << 7)
        | (b[3] & 0x7F)
    )


def _find_first_header(stream: BinaryIO, start: int) -> int | None:
    """Scan for the first valid MPEG Layer III header.

    :param stream: File stream.
    :param start: Byte offset to start scanning from.
    :returns: Byte offset of first valid header, or None if not found.
    """
    stream.seek(start)
    data = stream.read(65536)

    for i in range(len(data) - 3):
        word = int.from_bytes(data[i : i + 4], "big")
        if _is_valid_header(word):
            return start + i
    return None


def _is_valid_header(word: int) -> bool:
    """Validate the basic MPEG Layer III header fields.

    :param word: 4-byte header as integer.
    :returns: True if header is valid, False otherwise.
    """
    # Check sync bits (first 11 bits must be all 1s)
    if ((word >> 21) & 0x7FF) != 0x7FF:
        return False

    # Extract key fields
    version_id = (word >> 19) & 0b11
    layer_id = (word >> 17) & 0b11
    bitrate_index = (word >> 12) & 0b1111
    sampling_index = (word >> 10) & 0b11

    # Validate fields according to MPEG Layer III spec
    # version 0b01 is reserved
    # layer must be 0b01 for Layer III
    if version_id == 0b01 or layer_id != 0b01:
        return False

    # bitrate 0 is free format, which we do not accept here
    # bitrate 15 is forbidden for all versions/layers
    # sampling rate 3 is reserved for all versions
    return not (bitrate_index in (0, 15) or sampling_index == 3)


def _parse_header(word: int) -> MpegHeader:
    """Extract key fields from header word.

    :param word: 4-byte header as integer.
    :returns: Parsed MpegHeader dataclass.
    """
    return MpegHeader(
        version_id=(word >> 19) & 0b11,
        layer_id=(word >> 17) & 0b11,
        bitrate_index=(word >> 12) & 0b1111,
        sampling_rate_index=(word >> 10) & 0b11,
        padding_bit=(word >> 9) & 0b1,
    )


def _frame_length(header: MpegHeader) -> int:
    """Compute MP3 frame length in bytes from header fields.

    :param header: Parsed MpegHeader.
    :returns: Frame length in bytes, or 0 if invalid.
    """
    # Lookup bitrate and sample rate from tables
    bitrate_kbps = BITRATE_TABLE[header.version_id][header.layer_id][
        header.bitrate_index
    ]
    sample_rate = SAMPLERATE_TABLE[header.version_id][
        header.sampling_rate_index
    ]

    # If either is None, the header is invalid. We did a generic range
    # check earlier,  but now we check the specific table entries.
    if bitrate_kbps is None or sample_rate is None:
        return 0

    bitrate = bitrate_kbps * 1000

    if header.version_id == 0b11:  # MPEG-1
        return (144 * bitrate) // sample_rate + header.padding_bit
    return (72 * bitrate) // sample_rate + header.padding_bit
