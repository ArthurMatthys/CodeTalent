use crate::Result;
use std::io::{BufWriter, Seek, Write};

#[derive(Debug)]
pub struct WriterWithPos<W: Write + Seek> {
    writer: BufWriter<W>,
    pub(crate) pos: u64,
}

impl<W: Write + Seek> WriterWithPos<W> {
    pub fn new(mut buf: W) -> Result<Self>
    where
        W: Write + Seek,
    {
        let pos = buf.stream_position()?;
        let writer = BufWriter::new(buf);

        Ok(Self { writer, pos })
    }
}

impl<W: Write + Seek> Write for WriterWithPos<W> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let count = self.writer.write(buf)?;
        self.pos += count as u64;
        self.flush()?;

        Ok(count)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.writer.flush()
    }
}
