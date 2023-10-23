use std::io::{BufRead, BufReader, Write};
use std::net::TcpStream;

fn main() {
    let addr = match distributed_banker::parse_arguments() {
        Ok(addr) => addr,
        Err(error) => {
            eprintln!("Could not parse arguments: {}", error);
            return;
        }
    };

    let banker = match TcpStream::connect(addr) {
        Ok(listener) => listener,
        Err(error) => {
            eprintln!("Could not connect to banker: {}", error);
            return;
        }
    };

    invest(banker);
}

fn invest(mut write_banker: TcpStream) {
    let read_banker = match write_banker.try_clone() {
        Ok(read_banker) => read_banker,
        Err(error) => {
            eprintln!("Could not clone banker read stream: {}", error);
            return;
        }
    };

    let mut reader = BufReader::new(read_banker);

    loop {
        let mut message = String::new();
        if let Err(error) = reader.read_line(&mut message) {
            eprintln!("Could not receive money from banker: {}", error);
            return;
        }

        if let Some(b'\n') = message.bytes().last() {
            message.pop();
        }

        let mut money = match message.parse::<usize>() {
            Ok(money) => money,
            Err(error) => {
                eprintln!("Could not parse money from banker: {}", error);
                return;
            }
        };

        money += 100;

        if let Err(error) = writeln!(&mut write_banker, "{}", money) {
            eprintln!("Could not send money to banker: {}", error);
            return;
        }
    }
}
