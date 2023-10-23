use std::env;
use std::error::Error;
use std::net::SocketAddr;

pub fn parse_arguments() -> Result<SocketAddr, Box<dyn Error>> {
    let mut args = env::args();

    let addr = args
        .nth(1)
        .ok_or("expected socket address as first argument")?;

    let addr = addr.parse::<SocketAddr>()?;

    return Ok(addr);
}
