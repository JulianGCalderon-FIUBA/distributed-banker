use std::error::Error;
use std::io::{self, BufReader, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::sync::atomic::{AtomicIsize, Ordering};
use std::sync::{Arc, Barrier};
use std::{env, thread};

use distributed_banker::read_usize;

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

    let listener = TcpListener::bind(addr).expect("Could not bind to address");

    let banker = Arc::new(Banker::new(investors + 1, 1000));

    let mut handles = Vec::new();

    let banker_clone = banker.clone();
    handles.push(thread::spawn(|| log(banker_clone)));

    for client in listener.incoming() {
        let client = client.expect("Could not accept connection");

        let handler = Handler::new(&banker, client).expect("Chould not create handler");

        let handle = thread::spawn(|| handler.handle());
        handles.push(handle);
    }

    for handle in handles {
        handle.join().expect("Could not join handle");
    }
}

struct Banker {
    read_barrier: Barrier,
    write_barrier: Barrier,
    investors: usize,
    money: AtomicIsize,
}

impl Banker {
    fn wait_read(&self) {
        self.read_barrier.wait();
    }

    fn wait_write(&self) {
        self.write_barrier.wait();
    }

    fn sub(&self, amount: usize) {
        self.money.fetch_sub(amount as isize, Ordering::Relaxed);
    }

    fn add(&self, amount: usize) {
        self.money.fetch_add(amount as isize, Ordering::Relaxed);
    }

    fn money(&self) -> isize {
        self.money.load(Ordering::Relaxed)
    }
}

impl Banker {
    fn new(investors: usize, initial: isize) -> Self {
        let read_barrier = Barrier::new(investors);
        let write_barrier = Barrier::new(investors);
        let money = AtomicIsize::new(initial);

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
            if let Err(error) = self.send_share() {
                eprintln!("Could not send money to client: {}", error);
            }

            match read_usize(&mut self.reader) {
                Ok(result) => {
                    self.banker.add(result);
                }
                Err(error) => {
                    eprintln!("Could not receive money from client: {}", error);
                    return;
                }
            };
        }
    }

    fn send_share(&mut self) -> io::Result<()> {
        self.banker.wait_read();

        let money = self.banker.money() as usize;
        let share = money / self.banker.investors;

        writeln!(&mut self.client, "{}", share)?;

        self.banker.wait_write();

        self.banker.sub(share);

        Ok(())
    }
}

fn log(banker: Arc<Banker>) {
    loop {
        banker.wait_read();

        println!("Starting Week with: {}", banker.money());

        banker.wait_write();
    }
}
