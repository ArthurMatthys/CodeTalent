use clap::{arg, builder::Command};
use kvs::{KvStore, Result};
use std::{env::current_dir, process::exit};

fn main() -> Result<()> {
    let m = Command::new("kvs")
        .author(env!("CARGO_PKG_AUTHORS"))
        .version(env!("CARGO_PKG_VERSION"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
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

    let mut kvs = KvStore::open(current_dir()?)?;

    // kvs.set("key1".to_string(), "value1".to_string())?;
    // let v = kvs.remove("key1".to_string());
    // eprintln!("{v:?}");
    // let v = kvs.get("key1".to_string());
    // eprintln!("{v:?}");

    match m.subcommand() {
        Some(("get", args)) => {
            let key = args.get_one::<String>("KEY").unwrap();
            if let Some(v) = kvs.get(key.to_string())? {
                println!("{}", v)
            } else {
                println!("Key not found");
            }
        }
        Some(("set", args)) => {
            let key = args.get_one::<String>("KEY").unwrap();
            let value = args.get_one::<String>("VALUE").unwrap();
            kvs.set(key.to_string(), value.to_string())?;
        }
        Some(("rm", args)) => {
            let key = args.get_one::<String>("KEY").unwrap();
            if kvs.remove(key.to_string()).is_err() {
                println!("Key not found");
                exit(1)
            }
        }
        _ => unreachable!(),
    };
    Ok(())
}
