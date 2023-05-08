use std::{
    fs::OpenOptions,
    io::{BufRead, BufReader, BufWriter, Read, Write},
    net::{SocketAddr, TcpListener, TcpStream},
    path::Path,
};

use clap::{arg, builder::Command};
use kvs::{address_parser, KvsEngine, MyError, Result};
extern crate slog_term;
use slog::{info, o, Drain, Logger};

const ENGINE_PATH: &str = "engine";

fn check_engine(arg_engine: Option<String>) -> Result<String> {
    match OpenOptions::new().read(true).open(ENGINE_PATH) {
        Ok(f) => {
            let mut reader = BufReader::new(f);
            let mut engine = String::new();
            reader.read_line(&mut engine)?;
            if let Some(target_engine) = arg_engine {
                if target_engine == engine {
                    Ok(engine)
                } else {
                    Err(MyError::WrongEngine.into())
                }
            } else {
                Ok(engine)
            }
        }
        Err(_) => {
            let file = OpenOptions::new()
                .write(true)
                .create(true)
                .open(ENGINE_PATH)?;
            let engine = arg_engine.unwrap_or("kvs".to_string());
            let mut writer = BufWriter::new(file);
            writer.write_all(engine.as_bytes())?;
            writer.flush()?;
            Ok(engine)
        }
    }
}

fn handle_client(kvs: &mut kvs::KvStore, logger: &Logger, stream: &mut TcpStream) -> Result<()> {
    info!(logger, "We've got a connection from : {:?}", stream);

    let mut buffer = vec![];
    stream.read_to_end(&mut buffer)?;

    let cmd = serde_json::from_slice::<kvs::Command>(&buffer)?;

    let mut writer = BufWriter::new(stream);

    info!(logger, "Read {:?}", cmd);
    match cmd {
        kvs::Command::Get { key } => {
            if let Some(v) = kvs.get(key)? {
                writer.write_all(v.as_bytes())?;
            } else {
                writer.write_all(b"Key not found")?;
            }
        }
        kvs::Command::Set { key, value } => {
            kvs.set(key, value)?;
        }
        kvs::Command::Rm { key } => {
            if kvs.remove(key).is_err() {
                writer.write_all(b"Key not found")?;
            }
        }
    }
    writer.flush()?;

    Ok(())
}

fn main() -> Result<()> {
    let plain = slog_term::PlainSyncDecorator::new(std::io::stderr());
    let logger = Logger::root(slog_term::FullFormat::new(plain).build().fuse(), o!());
    let m = Command::new("kvs")
        .author(env!("CARGO_PKG_AUTHORS"))
        .version(env!("CARGO_PKG_VERSION"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .args([
            arg!(--addr <addr> "IP:Port")
                .required(false)
                .default_value("127.0.0.1:4000"),
            arg!(--engine <engine> "IP:Port").required(false),
        ])
        .get_matches();

    let (addr, ip) = address_parser(m.get_one::<String>("addr").expect("Default value present"))?;
    let engine = check_engine(m.get_one::<String>("engine").cloned())?;

    let logs = Path::new(".");

    let mut kvs = kvs::KvStore::open(logs)?;

    info!(logger, env!("CARGO_PKG_VERSION"));
    info!(logger, "{:?}", engine);
    info!(logger, "{:?}:{:?}", addr, ip);

    let target_addr = SocketAddr::new(addr, ip);
    let listener = TcpListener::bind(target_addr)?;

    for stream in listener.incoming() {
        handle_client(&mut kvs, &logger, &mut stream?)?;
    }

    Ok(())
}
