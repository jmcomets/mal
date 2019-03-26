#[macro_use] extern crate lazy_static;

// #![deny(warnings)]

mod printer;
mod reader;
mod types;

use std::io::{
    self,
    Write
};

fn read(s: &str) -> Result<Option<types::MalType>, types::MalError> {
    reader::read_str(s)
}

fn eval(t: types::MalType) -> types::MalType {
    t
}

fn print(t: types::MalType) -> String {
    printer::pr_str(&t, true)
}

fn rep(s: &str) -> String {
    match read(s) {
        Ok(Some(t))                            => print(eval(t)),
        Ok(None)                               => "EOF".to_string(),
        Err(types::MalError::UnbalancedString) => "unbalanced string".to_string(),
        Err(types::MalError::UnbalancedList)   => "unbalanced list".to_string(),
        _                                      => unimplemented!(),
    }
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
        let line = line.trim_end();
        writeln!(&mut output, "{}", rep(line))?;
        output.flush()?;
    }

    Ok(())
}
