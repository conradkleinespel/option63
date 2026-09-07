use std::fs;
use std::io::{self, Read, Write};
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Input {
    Stdin,
    File(PathBuf),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Output {
    Stdout,
    File(PathBuf),
}

pub fn resolve_input(arg: Option<&str>) -> io::Result<Input> {
    match arg {
        None => Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "no input provided: pass a file path or '-' to read from stdin",
        )),
        Some("-") => Ok(Input::Stdin),
        Some(path) => Ok(Input::File(PathBuf::from(path))),
    }
}

pub fn resolve_output(arg: Option<&str>) -> Output {
    match arg {
        Some("-") | None => Output::Stdout,
        Some(path) => Output::File(PathBuf::from(path)),
    }
}

pub fn read_input(input: &Input) -> io::Result<Vec<u8>> {
    match input {
        Input::Stdin => {
            let mut buf = Vec::new();
            io::stdin().lock().read_to_end(&mut buf)?;
            Ok(buf)
        }
        Input::File(path) => fs::read(path),
    }
}

pub fn write_output(output: &Output, bytes: &[u8]) -> io::Result<()> {
    match output {
        Output::Stdout => io::stdout().lock().write_all(bytes),
        Output::File(path) => fs::write(path, bytes),
    }
}
