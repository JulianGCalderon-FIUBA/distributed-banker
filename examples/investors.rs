use std::env;
use std::error::Error;
use std::net::SocketAddr;
use std::process::Command;

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
    let (addr, investors) = match parse_arguments() {
        Ok((addr, investors)) => (addr, investors),
        Err(error) => {
            eprintln!("Could not parse arguments: {}", error);
            return;
        }
    };

    let mut childs = Vec::new();

    for _ in 0..investors {
        let child = match Command::new("cargo")
            .arg("run")
            .args(["--example", "investor"])
            .arg(addr.to_string())
            .spawn()
        {
            Ok(child) => child,
            Err(error) => {
                eprintln!("Could not spawn investor: {}", error);
                return;
            }
        };

        childs.push(child);
    }

    for mut child in childs {
        match child.wait() {
            Ok(status) => println!("Child exited with status: {}", status),
            Err(error) => eprintln!("Could not wait child: {}", error),
        };
    }
}
