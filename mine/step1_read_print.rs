#[macro_use]
extern crate lazy_static;

// #![deny(warnings)]

mod reader;
mod types;

use std::io::{
    self,
    Write
};

fn read(s: &str) -> &str {
    s
}

fn eval(s: &str) -> &str {
    s
}

fn print(s: &str) -> &str {
    s
}

fn rep(s: &str) -> &str {
    print(eval(read(s)))
}

fn main() -> io::Result<()> {
    let input = io::stdin();
    let mut output = io::stdout();

    let mut line = String::new();
    loop {
        write!(&mut output, "user> ")?;
        output.flush()?;

        line.clear();
        let nb_bytes_read = input.read_line(&mut line)?;
        if nb_bytes_read == 0 {
            break;
        }
        let line = line.trim_right();
        writeln!(&mut output, "{}", rep(line))?;
        output.flush()?;
    }

    Ok(())
}
