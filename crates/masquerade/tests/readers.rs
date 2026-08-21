use std::io::{self, Cursor, Read, Seek, SeekFrom};

use masquerade::{Inspector, MediaType, ValidationStatus};

struct ShortReader {
    inner: Cursor<Vec<u8>>,
    maximum_read: usize,
}

impl Read for ShortReader {
    fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
        let length = buffer.len().min(self.maximum_read);
        self.inner.read(&mut buffer[..length])
    }
}

impl Seek for ShortReader {
    fn seek(&mut self, position: SeekFrom) -> io::Result<u64> {
        self.inner.seek(position)
    }
}

struct FailingReader {
    inner: Cursor<Vec<u8>>,
    successful_reads: usize,
}

impl Read for FailingReader {
    fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
        if self.successful_reads == 0 {
            return Err(io::Error::other("injected read failure"));
        }
        self.successful_reads -= 1;
        self.inner.read(buffer)
    }
}

impl Seek for FailingReader {
    fn seek(&mut self, position: SeekFrom) -> io::Result<u64> {
        self.inner.seek(position)
    }
}

#[test]
fn supports_readers_that_return_short_successful_reads() {
    let mut reader = ShortReader {
        inner: Cursor::new(minimal_bmp()),
        maximum_read: 1,
    };
    let result = Inspector::new()
        .inspect_reader(&mut reader, MediaType::Bmp)
        .unwrap();
    assert_eq!(result.status(), ValidationStatus::Valid);
}

#[test]
fn propagates_non_eof_reader_errors() {
    let mut reader = FailingReader {
        inner: Cursor::new(minimal_bmp()),
        successful_reads: 1,
    };
    let error = Inspector::new()
        .inspect_reader(&mut reader, MediaType::Bmp)
        .unwrap_err();
    assert_eq!(error.kind(), io::ErrorKind::Other);
}

#[test]
fn arbitrary_small_inputs_do_not_panic() {
    for media_type in MediaType::ALL {
        for length in 0..256 {
            let data = vec![u8::try_from(length).unwrap_or(u8::MAX); length];
            let _result = Inspector::new().inspect_bytes(&data, *media_type).unwrap();
        }
    }
}

fn minimal_bmp() -> Vec<u8> {
    let mut data = Vec::new();
    data.extend_from_slice(b"BM");
    data.extend_from_slice(&58_u32.to_le_bytes());
    data.extend_from_slice(&[0; 4]);
    data.extend_from_slice(&54_u32.to_le_bytes());
    data.extend_from_slice(&40_u32.to_le_bytes());
    data.extend_from_slice(&1_i32.to_le_bytes());
    data.extend_from_slice(&1_i32.to_le_bytes());
    data.extend_from_slice(&1_u16.to_le_bytes());
    data.extend_from_slice(&24_u16.to_le_bytes());
    data.extend_from_slice(&[0; 24]);
    data.extend_from_slice(&[0; 4]);
    data
}
