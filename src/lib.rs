use std::error::Error;
use std::io::{self, BufRead};

pub fn read_usize<R: BufRead>(mut r: R) -> io::Result<usize> {
    let mut message = String::new();
    r.read_line(&mut message)?;

    if let Some(b'\n') = message.bytes().last() {
        message.pop();
    }

    let result = message.parse().map_err(invalid_data_map)?;

    Ok(result)
}

fn invalid_data_map<E: Into<Box<dyn Error + Send + Sync>>>(err: E) -> io::Error {
    std::io::Error::new(io::ErrorKind::InvalidData, err)
}
