use shared_child::SharedChild;
use std::env;
use std::error::Error;
use std::net::SocketAddr;
use std::process::Command;
use std::sync::Arc;

pub fn parse_arguments() -> Result<(SocketAddr, usize), Box<dyn Error>> {
    let mut args = env::args();

    let addr = args
        .nth(1)
        .ok_or("expected socket address as first argument")?
        .parse()?;

    let investors = args
        .next()
        .ok_or("expected number of investors as seconds argument")?
        .parse()?;

    return Ok((addr, investors));
}

fn main() {
    let (addr, investors) = parse_arguments().expect("Could not parse arguments");

    let mut childs = Vec::new();

    for _ in 0..investors {
        let mut command = investor_command(addr);

        let child = SharedChild::spawn(&mut command).expect("Could not spawn investor");

        childs.push(Arc::new(child));
    }

    let childs_clone = childs.clone();
    let _ = ctrlc::set_handler(move || {
        for child in &childs_clone {
            let _ = child.kill();
        }
    });

    for child in &childs {
        let _ = child.wait();
    }
}

fn investor_command(addr: SocketAddr) -> Command {
    let mut command = Command::new("cargo");

    command
        .arg("run")
        .args(["--example", "investor"])
        .arg(addr.to_string());

    command
}
