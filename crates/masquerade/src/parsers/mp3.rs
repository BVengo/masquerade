//! MP3 file structure verification by checking frame alignment.
//!
//! Specs: <https://mp3guessenc.sourceforge.io/MPEG%20Layer3%20Bitstream%20Syntax%20and%20Decoding.pdf>

// An MP3 file is composed of a sequence of frames, each starting with a
// 4-byte header that encodes metadata about the frame, including bitrate,
// sampling rate, and padding. This module reads the file, locates a number
// of these headers, and verifies that the frames are correctly aligned
// according to the MP3 specification.
//
// Audio frame structure:
// - header (4 bytes)
// - error check (optional, 17 or 32 bytes)
// - audio data (variable length)
//
// Header structure:
// | Field          | Bits | Description                                         |
// |----------------|------|-----------------------------------------------------|
// | sync           | 11   | Frame sync (all bits set to 1)                      |
// | IDex           | 1    | Extended algorithm ID (1=MPEG1 or MPEG2, 0=MPEG2.5) |
// | ID             | 1    | ID of the algorithm (1=MPEG1, 0=MPEG2 or MPEG2.5)   |
// | Layer          | 2    | Indicate which layer is used.                       |
// | protection bit | 1    | Protection bit (0 if protected by CRC)              |
// | bitrate index  | 4    | Bitrate index - all 0 indicates free format         |
// | sampling freq  | 2    | Sampling frequency index                            |
// | padding bit    | 1    | Padding bit - if 1, frame has an additional slot    |
// | private bit    | 1    | Private bit - private use, not used by ISO/IEC      |
// | mode           | 2    | Channel mode (stereo, joint, dual, single)          |
// | mode extension | 2    | Mode extension (only if Joint stereo)               |
// | copyright      | 1    | Copyright (1 if protected by copyright)             |
// | original       | 1    | 0 if a copy, and 1 if an original                   |
// | emphasis       | 2    | The type of de-emphasis that shall be used          |

use std::io::{self, Read, Seek, SeekFrom};

use crate::{
    CheckResult, DiagnosticCode, ValidationLimits,
    io_util::{read_exact_or_eof, read_prefix},
};

#[derive(Clone, Copy)]
struct FrameHeader {
    version: u8,
    bitrate_index: usize,
    sample_rate_index: usize,
    padding: u64,
}

fn has_signature(data: &[u8]) -> bool {
    if data.starts_with(b"ID3") {
        return data.len() >= 10
            && matches!(data[3], 2..=4)
            && data[4] != 0xff
            && data[6..10].iter().all(|byte| *byte < 0x80);
    }
    data.get(..4).is_some_and(|bytes| {
        valid_header(u32::from_be_bytes(bytes.try_into().expect("four bytes")))
    })
}

pub(crate) fn signature<R: Read + Seek + ?Sized>(
    reader: &mut R,
    _limits: &ValidationLimits,
) -> io::Result<bool> {
    Ok(has_signature(&read_prefix(reader, 4_096)?))
}

pub(crate) fn validate_structure<R: Read + Seek + ?Sized>(
    reader: &mut R,
    limits: &ValidationLimits,
) -> io::Result<CheckResult> {
    let audio_start = skip_id3v2(reader)?;
    let Some(first_frame) = find_first_frame(reader, audio_start, limits.max_mp3_scan_bytes())?
    else {
        return Ok(CheckResult::invalid(
            DiagnosticCode::MissingFrameHeader,
            "No valid MPEG Layer III frame header was found",
        ));
    };
    let verified = count_consecutive_frames(reader, first_frame, limits.mp3_frames_to_check())?;
    Ok(if verified >= limits.mp3_min_frames() {
        CheckResult::valid()
    } else {
        CheckResult::invalid(
            DiagnosticCode::InsufficientValidFrames,
            format!(
                "Fewer than {} consecutive valid MP3 frames found",
                limits.mp3_min_frames()
            ),
        )
    })
}

fn skip_id3v2<R: Read + Seek + ?Sized>(reader: &mut R) -> io::Result<u64> {
    let header = read_prefix(reader, 10)?;
    if header.len() != 10 || !header.starts_with(b"ID3") {
        return Ok(0);
    }
    let size = header[6..10]
        .iter()
        .fold(0_u64, |value, byte| (value << 7) | u64::from(byte & 0x7f));
    // TODO: Validate the ID3 version, flags and synchsafe bytes, account for
    // ID3v2.4 footers, and ensure the tag size is inside the file when Python does.
    Ok(10 + size)
}

fn find_first_frame<R: Read + Seek + ?Sized>(
    reader: &mut R,
    start: u64,
    scan_bytes: usize,
) -> io::Result<Option<u64>> {
    reader.seek(SeekFrom::Start(start))?;
    let mut data = Vec::with_capacity(scan_bytes);
    reader
        .take(u64::try_from(scan_bytes).unwrap_or(u64::MAX))
        .read_to_end(&mut data)?;
    Ok(data
        .windows(4)
        .position(|bytes| valid_header(u32::from_be_bytes(bytes.try_into().expect("four bytes"))))
        .map(|offset| start + u64::try_from(offset).expect("scan offset fits u64")))
}

fn count_consecutive_frames<R: Read + Seek + ?Sized>(
    reader: &mut R,
    mut position: u64,
    maximum: usize,
) -> io::Result<usize> {
    // TODO: Require stable MPEG version and sample rate across consecutive
    // frames, and validate optional CRC data when Python gains those checks.
    let mut verified = 0;
    for _ in 0..maximum {
        reader.seek(SeekFrom::Start(position))?;
        let mut bytes = [0_u8; 4];
        if !read_exact_or_eof(reader, &mut bytes)? {
            break;
        }
        let word = u32::from_be_bytes(bytes);
        if !valid_header(word) {
            break;
        }
        let header = parse_header(word);
        let Some(length) = frame_length(header) else {
            break;
        };
        let Some(next) = position.checked_add(length) else {
            break;
        };
        position = next;
        verified += 1;
    }
    Ok(verified)
}

fn valid_header(word: u32) -> bool {
    let version = (word >> 19) & 0b11;
    let layer = (word >> 17) & 0b11;
    let bitrate = (word >> 12) & 0b1111;
    let sample_rate = (word >> 10) & 0b11;
    ((word >> 21) & 0x7ff) == 0x7ff
        && version != 0b01
        && layer == 0b01
        && !matches!(bitrate, 0 | 15)
        && sample_rate != 3
}

fn parse_header(word: u32) -> FrameHeader {
    FrameHeader {
        version: ((word >> 19) & 0b11) as u8,
        bitrate_index: ((word >> 12) & 0b1111) as usize,
        sample_rate_index: ((word >> 10) & 0b11) as usize,
        padding: u64::from((word >> 9) & 1),
    }
}

fn frame_length(header: FrameHeader) -> Option<u64> {
    const MPEG_1: [u64; 16] = [
        0, 32, 40, 48, 56, 64, 80, 96, 112, 128, 160, 192, 224, 256, 320, 0,
    ];
    const MPEG_2: [u64; 16] = [
        0, 8, 16, 24, 32, 40, 48, 56, 64, 80, 96, 112, 128, 144, 160, 0,
    ];
    const MPEG_25: [u64; 16] = [0, 8, 16, 24, 32, 40, 48, 56, 64, 0, 0, 0, 0, 0, 0, 0];
    const SAMPLE_1: [u64; 4] = [44_100, 48_000, 32_000, 0];
    const SAMPLE_2: [u64; 4] = [22_050, 24_000, 16_000, 0];
    const SAMPLE_25: [u64; 4] = [11_025, 12_000, 8_000, 0];
    let (bitrates, samples, multiplier) = match header.version {
        0b11 => (&MPEG_1, &SAMPLE_1, 144),
        0b10 => (&MPEG_2, &SAMPLE_2, 72),
        0b00 => (&MPEG_25, &SAMPLE_25, 72),
        _ => return None,
    };
    let bitrate = bitrates[header.bitrate_index] * 1_000;
    let sample_rate = samples[header.sample_rate_index];
    (bitrate != 0 && sample_rate != 0).then(|| multiplier * bitrate / sample_rate + header.padding)
}
