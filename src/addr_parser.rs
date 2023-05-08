use crate::Result;
use std::{net::IpAddr, process::exit, str::FromStr};

pub fn address_parser(add_ip: &str) -> Result<(IpAddr, u16)> {
    match add_ip.split(':').collect::<Vec<_>>().as_slice() {
        [addr, ip] => Ok((IpAddr::from_str(addr)?, ip.parse::<u16>()?)),
        _ => {
            eprintln!("Address should be formated like this: IP:Port");
            exit(1);
        }
    }
}
