use std::error::Error;
use std::io::{self, BufRead, BufReader, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::sync::{Arc, Barrier, Mutex};
use std::{env, thread};

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

    let listener = match TcpListener::bind(addr) {
        Ok(listener) => listener,
        Err(error) => {
            eprintln!("Could not bind to address: {}", error);
            return;
        }
    };

    let banker = Arc::new(Banker::new(investors + 1, 1000));

    let mut handles = Vec::new();

    let banker_clone = banker.clone();
    handles.push(thread::spawn(|| log(banker_clone)));

    for client in listener.incoming() {
        let client = match client {
            Ok(client) => client,
            Err(err) => {
                eprintln!("Could not accept connection: {}", err);
                continue;
            }
        };

        let handler = match Handler::new(&banker, client) {
            Ok(handler) => handler,
            Err(error) => {
                eprintln!("Could not create handler: {}", error);
                continue;
            }
        };

        let handle = thread::spawn(|| handler.handle());
        handles.push(handle);
    }

    for handle in handles {
        if let Err(error) = handle.join() {
            eprintln!("Could not join thread: {:?}", error)
        }
    }
}

struct Banker {
    read_barrier: Barrier,
    write_barrier: Barrier,
    investors: usize,
    money: Mutex<usize>,
}

impl Banker {
    fn new(investors: usize, initial: usize) -> Self {
        let read_barrier = Barrier::new(investors);
        let write_barrier = Barrier::new(investors);
        let money = Mutex::new(initial);

        return Self {
            read_barrier,
            write_barrier,
            investors,
            money,
        };
    }
}

struct Handler {
    banker: Arc<Banker>,
    client: TcpStream,
    reader: BufReader<TcpStream>,
}

impl Handler {
    pub fn new(banker: &Arc<Banker>, client: TcpStream) -> io::Result<Self> {
        let banker = banker.clone();

        let read_client = client.try_clone()?;
        let reader = BufReader::new(read_client);

        return Ok(Self {
            banker,
            client,
            reader,
        });
    }

    pub fn handle(mut self) {
        loop {
            self.banker.read_barrier.wait();

            let money = *self.banker.money.lock().expect("Should never be poisoned");
            let share = money / self.banker.investors;

            if let Err(error) = writeln!(&mut self.client, "{}", share) {
                eprintln!("Could not send money to client: {}", error);
                return;
            }

            self.banker.write_barrier.wait();

            *self.banker.money.lock().expect("Should never be poisoned") -= share;

            let mut message = String::new();
            if let Err(error) = self.reader.read_line(&mut message) {
                eprintln!("Could not receive money from client: {}", error);
                return;
            }

            if let Some(b'\n') = message.bytes().last() {
                message.pop();
            }

            {
                let mut money = self
                    .banker
                    .money
                    .lock()
                    .expect("Lock should never be poisoned");

                match message.parse::<isize>() {
                    Ok(gain) => *money = money.saturating_add_signed(gain),
                    Err(error) => {
                        eprintln!("Could not parse moeny from client: {}", error);
                        return;
                    }
                };
            }
        }
    }
}

fn log(banker: Arc<Banker>) {
    loop {
        banker.read_barrier.wait();

        let money = *banker.money.lock().expect("Should never be poisoned");
        println!("Starting Week with: {}", money);

        banker.write_barrier.wait();
    }
}
