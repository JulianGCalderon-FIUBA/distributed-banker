use std::error::Error;
use std::io::{self, BufReader, Write};
use std::net::{SocketAddr, TcpStream};
use std::time::Duration;
use std::{env, thread};

use distributed_banker::read_usize;

pub fn parse_arguments() -> Result<SocketAddr, Box<dyn Error>> {
    let mut args = env::args();

    let addr = args
        .nth(1)
        .ok_or("expected socket address as first argument")?
        .parse()?;

    return Ok(addr);
}

fn main() {
    let addr = parse_arguments().expect("Could not parse arguments");

    let banker = TcpStream::connect(addr).expect("Could not connect to banker");

    let investor = Investor::new(banker).expect("Could not create investor");

    investor.invest()
}

struct Investor {
    banker: TcpStream,
    reader: BufReader<TcpStream>,
}

impl Investor {
    pub fn new(banker: TcpStream) -> io::Result<Self> {
        let read_banker = banker.try_clone()?;
        let reader = BufReader::new(read_banker);

        Ok(Self { banker, reader })
    }

    fn invest(mut self) {
        loop {
            let money = match read_usize(&mut self.reader) {
                Ok(money) => money,
                Err(error) => {
                    eprintln!("Could not parse money from banker: {}", error);
                    return;
                }
            };

            let money = self.invest_money(money);

            thread::sleep(Duration::from_secs(1));

            if let Err(error) = writeln!(&mut self.banker, "{}", money) {
                eprintln!("Could not send money to banker: {}", error);
                return;
            }
        }
    }

    fn invest_money(&self, money: usize) -> usize {
        let ratio = rand::random::<f64>() * 0.2 + 0.92;

        return (money as f64 * ratio) as usize;
    }
}
