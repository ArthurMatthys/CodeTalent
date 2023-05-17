use std::{
    fs::OpenOptions,
    io::{BufRead, BufReader, BufWriter, Read, Write},
    net::{SocketAddr, TcpListener, TcpStream},
    path::Path,
};

use clap::{arg, command, Parser};
use kvs::{address_parser, KvsEngine, MyError, Result};
extern crate slog_term;
use slog::{debug, info, o, Drain, Logger};

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

    let addr = stream.peer_addr()?;
    let mut reader = BufReader::new(stream.try_clone()?);
    let mut writer = BufWriter::new(stream);
    // serde_json::to_writer(writer, &buffer)?;
    let mut buffer = vec![];
    reader.read_until(b'\n', &mut buffer)?;
    info!(logger, "{:?}", buffer);

    // stream.write(&buffer[..a])?;

    // let cmd = serde_json::from_reader::<_, kvs::Command>(&mut reader)?;
    let cmd = serde_json::from_slice::<kvs::Command>(&buffer)?;

    match cmd {
        kvs::Command::Get { key } => {
            if let Some(v) = kvs.get(key)? {
                let resp = v;
                serde_json::to_writer(&mut writer, &resp)?;
                debug!(logger, "Response sent to {} : {:?}", addr, resp);
            } else {
                let resp = b"Key not found";
                serde_json::to_writer(&mut writer, &resp)?;
                debug!(logger, "Response sent to {} : {:?}", addr, resp);
            }
        }
        kvs::Command::Set { key, value } => {
            kvs.set(key, value)?;
        }
        kvs::Command::Rm { key } => {
            if kvs.remove(key).is_err() {
                let resp = b"Key not found";
                serde_json::to_writer(&mut writer, &resp)?;
                debug!(logger, "Response sent to {} : {:?}", addr, resp);
            }
        }
    }
    writer.write_all(b"\n")?; // Rajoute cette ligne
    writer.flush()?;

    Ok(())
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[arg(short, long, value_name = "addr", default_value = "127.0.0.1:4000")]
    addr: String,

    #[arg(short, long, value_name = "engine")]
    engine: Option<String>,
}

fn main() -> Result<()> {
    let plain = slog_term::PlainSyncDecorator::new(std::io::stderr());
    let logger = Logger::root(slog_term::FullFormat::new(plain).build().fuse(), o!());

    let cli = Cli::parse();

    let (addr, ip) = address_parser(&cli.addr)?;
    let engine = check_engine(cli.engine)?;

    let logs = Path::new(".");

    let mut kvs = kvs::KvStore::open(logs)?;

    info!(logger, env!("CARGO_PKG_VERSION"));
    info!(logger, "{:?}", engine);
    info!(logger, "{:?}:{:?}", addr, ip);

    let target_addr = SocketAddr::new(addr, ip);
    let listener = TcpListener::bind(target_addr)?;

    for stream in listener.incoming() {
        handle_client(&mut kvs, &logger, &mut stream?)?;
        debug!(logger, "Client handled {}", addr);
    }

    Ok(())
}
