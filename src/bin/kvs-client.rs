use clap::{arg, command, Parser};
use kvs::{address_parser, Command, Result};
use std::{
    io::{BufRead, BufReader, BufWriter, Write},
    net::TcpStream,
};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[arg(short, long, value_name = "addr", default_value = "127.0.0.1:4000")]
    addr: String,

    #[command(subcommand)]
    command: Command,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let (addr, port) = address_parser(&cli.addr)?;

    let stream = TcpStream::connect(format!("{}:{}", addr, port))?;
    let mut writer = BufWriter::new(stream.try_clone()?);
    let mut reader = BufReader::new(stream);

    serde_json::to_writer(&mut writer, &cli.command)?;
    writer.write_all(b"\n")?; // Rajoute cette ligne
    writer.flush()?;

    // let mut buffer = vec![];
    // reader.read_until(b'\n', &mut buffer)?;
    // println!("buf : {:?}", buffer);
    let v = serde_json::from_reader::<_, String>(&mut reader)?;
    println!("v : {:?}", v);

    // match cli.command {
    //     Command::Get { key } => {
    //         serde_json::to_writer(&mut writer, &cmd)?;
    //         writer.flush()?;
    //     }
    //     Command::Set { key, value } => {
    //         serde_json::to_writer(&mut writer, &cmd)?;
    //         writer.flush()?;
    //     }
    //     Command::Rm { key } => {
    //         serde_json::to_writer(&mut writer, &cmd)?;
    //         writer.flush()?;
    //     }
    //     _ => unreachable!(),
    // };
    Ok(())
}
