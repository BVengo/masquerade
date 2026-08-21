use std::io::{self, Read, Seek, SeekFrom};

pub(crate) fn read_exact_or_eof<R: Read + ?Sized>(
    reader: &mut R,
    buffer: &mut [u8],
) -> io::Result<bool> {
    match reader.read_exact(buffer) {
        Ok(()) => Ok(true),
        Err(error) if error.kind() == io::ErrorKind::UnexpectedEof => Ok(false),
        Err(error) => Err(error),
    }
}

pub(crate) fn read_prefix<R: Read + Seek + ?Sized>(
    reader: &mut R,
    limit: usize,
) -> io::Result<Vec<u8>> {
    reader.seek(SeekFrom::Start(0))?;
    let mut data = Vec::with_capacity(limit);
    reader
        .take(u64::try_from(limit).unwrap_or(u64::MAX))
        .read_to_end(&mut data)?;
    Ok(data)
}
