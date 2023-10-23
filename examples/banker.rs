use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::Barrier;
use std::thread;

fn main() {
    let addr = match distributed_banker::parse_arguments() {
        Ok(addr) => addr,
        Err(error) => {
            eprintln!("Could not parse arguments: {}", error);
            return;
        }
    };

    let listener = match TcpListener::bind(addr) {
        Ok(listener) => listener,
        Err(error) => {
            eprintln!("Could not bind to address: {}", error);
            return;
        }
    };

    let mut handles = Vec::new();

    for client in listener.incoming() {
        let client = match client {
            Ok(client) => client,
            Err(err) => {
                eprintln!("Could not accept connection: {}", err);
                continue;
            }
        };

        let handle = thread::spawn(|| handle_client(client));
        handles.push(handle);
    }

    for handle in handles {
        if let Err(error) = handle.join() {
            eprintln!("Could not join thread: {:?}", error)
        }
    }
}

fn handle_client(mut write_client: TcpStream) {
    let read_client = match write_client.try_clone() {
        Ok(read_client) => read_client,
        Err(error) => {
            eprintln!("Could not clone client read stream: {}", error);
            return;
        }
    };

    let mut reader = BufReader::new(read_client);

    let mut money = 1000;
    let read_barrier = Barrier::new(1);
    let write_barrier = Barrier::new(1);

    loop {
        read_barrier.wait();

        println!("Empezando semana con: {}", money);

        if let Err(error) = writeln!(&mut write_client, "{}", money) {
            eprintln!("Could not send money to client: {}", error);
            return;
        }

        write_barrier.wait();

        let mut message = String::new();
        if let Err(error) = reader.read_line(&mut message) {
            eprintln!("Could not receive money from client: {}", error);
            return;
        }

        if let Some(b'\n') = message.bytes().last() {
            message.pop();
        }

        money = match message.parse() {
            Ok(money) => money,
            Err(error) => {
                eprintln!("Could not parse moeny from client: {}", error);
                return;
            }
        };
    }
}
