use std::io::{BufReader, Read, Seek};

use crate::Result;

#[derive(Debug)]
pub struct ReaderWithPos<W: Read + Seek> {
    reader: BufReader<W>,
    pub pos: u64,
}

impl<W: Read + Seek> ReaderWithPos<W> {
    pub fn new(mut buf: W) -> Result<Self> {
        let pos = buf.stream_position()?;
        let reader = BufReader::new(buf);
        Ok(Self { reader, pos })
    }
}

impl<W: Read + Seek> Read for ReaderWithPos<W> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let count = self.reader.read(buf)?;
        self.pos += count as u64;
        Ok(count)
    }
}

impl<W: Read + Seek> Seek for ReaderWithPos<W> {
    fn seek(&mut self, pos: std::io::SeekFrom) -> std::io::Result<u64> {
        self.pos = self.reader.seek(pos)?;
        Ok(self.pos)
    }
}
