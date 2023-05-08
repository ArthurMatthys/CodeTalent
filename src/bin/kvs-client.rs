use clap::{arg, builder::Command};
use kvs::{address_parser, Result};
use std::{
    io::{BufWriter, Write},
    net::TcpStream,
};

fn main() -> Result<()> {
    // let log = Logger::root(Fuse(EprintlnDrain), o!("version" => "yop"));
    let m = Command::new("kvs")
        .author(env!("CARGO_PKG_AUTHORS"))
        .version(env!("CARGO_PKG_VERSION"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .arg(
            arg!(--addr <ADDR> "IP:Port")
                .required(false)
                .default_value("127.0.0.1:4000"),
        )
        .subcommand_required(true)
        .subcommands([
            Command::new("get")
                .about("Get the string value of a given string key")
                .arg(arg!(<KEY>).required(true)),
            Command::new("set")
                .about("Set the value of a string key to a string")
                .args([arg!(<KEY>).required(true), arg!(<VALUE>).required(true)]),
            Command::new("rm")
                .about("Remove a given key")
                .arg(arg!(<KEY>).required(true)),
        ])
        .get_matches();

    // let mut kvs = KvStore::open(current_dir()?)?;

    let (addr, port) = address_parser(m.get_one::<String>("addr").expect("Default value present"))?;

    let stream = TcpStream::connect(format!("{}:{}", addr, port))?;
    let mut writer = BufWriter::new(stream);
    match m.subcommand() {
        Some(("get", args)) => {
            let key = args.get_one::<String>("KEY").unwrap();
            let cmd = kvs::Command::Get { key: key.clone() };
            serde_json::to_writer(&mut writer, &cmd)?;
            writer.flush()?;
        }
        Some(("set", args)) => {
            let key = args.get_one::<String>("KEY").unwrap();
            let value = args.get_one::<String>("VALUE").unwrap();
            let cmd = kvs::Command::Set {
                key: key.clone(),
                value: value.clone(),
            };
            serde_json::to_writer(&mut writer, &cmd)?;
            writer.flush()?;
        }
        Some(("rm", args)) => {
            let key = args.get_one::<String>("KEY").unwrap();
            let cmd = kvs::Command::Rm { key: key.clone() };
            serde_json::to_writer(&mut writer, &cmd)?;
            writer.flush()?;
        }
        _ => unreachable!(),
    };
    Ok(())
}
