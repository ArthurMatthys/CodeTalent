use serde_json::Deserializer;
use std::{
    collections::HashMap,
    fs::{self, File, OpenOptions},
    io::{BufWriter, Read, Seek, SeekFrom, Write},
    path::{Path, PathBuf},
};

use crate::{
    error::MyError, reader::ReaderWithPos, writer::WriterWithPos, Command, CommandPos, Result,
};

const MAX_SPACE: u64 = 1024 * 1024;

pub struct KvStore {
    path: PathBuf,
    hmap: HashMap<String, CommandPos>,
    unused_space: u64,
    writer: WriterWithPos<File>,
    reader: ReaderWithPos<File>,
}
impl KvStore {
    fn new<T>(pathbuf: T) -> Result<Self>
    where
        T: Into<PathBuf>,
    {
        let path: PathBuf = pathbuf.into();
        let log_path = get_log_file(&path, false);

        let tmp = OpenOptions::new()
            .create(true)
            .write(true)
            .append(true)
            .open(path.clone())?;

        let mut content = HashMap::default();
        let mut reader = ReaderWithPos::new(File::open(log_path).unwrap()).unwrap();
        let size = read_log(&mut content, &mut reader)?;
        let writer = WriterWithPos::new(tmp)?;

        Ok(Self {
            path,
            hmap: content,
            reader,
            writer,
            unused_space: size,
        })
    }

    pub fn set(&mut self, key: String, value: String) -> Result<()> {
        let cmd = Command::set(key, value);
        let pos = self.writer.pos;
        // let written_bytes = self.writer.write(serde_json::to_string(&cmd)?.as_bytes())?;
        serde_json::to_writer(&mut self.writer, &cmd)?;
        self.writer.flush()?;
        match cmd {
            Command::Set { key, .. } => {
                if let Some(cmd) = self.hmap.insert(
                    key,
                    CommandPos {
                        offset: pos,
                        size: self.writer.pos - pos,
                    },
                ) {
                    self.unused_space += cmd.size;
                    if self.unused_space > MAX_SPACE {
                        self.compact()?;
                    }
                };
            }
            _ => unreachable!(),
        };
        Ok(())
    }

    pub fn get(&mut self, key: String) -> Result<Option<String>> {
        match self.hmap.get(&key) {
            Some(v) => {
                let reader = &mut self.reader;
                reader.seek(SeekFrom::Start(v.offset))?;
                let cmd_reader = reader.take(v.size);
                if let Command::Set { value, .. } = serde_json::from_reader(cmd_reader)? {
                    Ok(Some(value))
                } else {
                    Err(MyError::UnexpectedCommand)?
                }
            }
            None => Ok(None),
        }
    }

    pub fn remove(&mut self, key: String) -> Result<()> {
        if self.hmap.contains_key(&key) {
            let cmd = Command::rm(key);
            serde_json::to_writer(&mut self.writer, &cmd)?;
            match cmd {
                Command::Rm { key } => {
                    if let Some(cmd) = self.hmap.remove(&key) {
                        self.unused_space += cmd.size;
                    };
                    if self.unused_space > MAX_SPACE {
                        self.compact()?;
                    }
                    Ok(())
                }
                _ => unreachable!(),
            }
        } else {
            Err(MyError::KeyNotFound)?
        }
    }

    fn compact(&mut self) -> Result<()> {
        let tmp_file = get_log_file(&self.path, true);

        let tmp_file = OpenOptions::new()
            .create(true)
            .write(true)
            .append(true)
            .open(tmp_file)?;
        let new_writer = BufWriter::new(tmp_file);

        fs::rename(
            get_log_file(&self.path, true),
            get_log_file(&self.path, false),
        )?;
        self.unused_space = 0;
        Ok(())
    }

    pub fn open<T>(path: T) -> Result<Self>
    where
        T: Into<PathBuf>,
    {
        Self::new(path)
    }
}

fn read_log(
    content: &mut HashMap<String, CommandPos>,
    reader: &mut ReaderWithPos<File>,
) -> Result<u64> {
    let mut pos = reader.seek(SeekFrom::Start(0))?;
    let mut stream = Deserializer::from_reader(reader).into_iter::<Command>();
    let mut unused_space = 0;
    while let Some(cmd) = stream.next() {
        let offset = stream.byte_offset() as u64;
        match cmd? {
            Command::Set { key, value: _ } => {
                if let Some(old) = content.insert(
                    key,
                    CommandPos {
                        size: offset - pos,
                        offset: pos,
                    },
                ) {
                    unused_space += old.size;
                };
            }
            Command::Rm { key } => {
                if let Some(old) = content.remove(&key) {
                    unused_space += old.size;
                }
                unused_space += offset - pos;
            }
        }
        pos = offset;
    }
    Ok(pos)
}

fn get_log_file(path: &Path, backup: bool) -> PathBuf {
    let new_path = path.join("kvs.log");
    if backup {
        new_path.join(".tmp")
    } else {
        new_path
    }
}
