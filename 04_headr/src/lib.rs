use std::{
    fs::File,
    io::{self, BufRead, BufReader, Read},
    path::Path,
};

use anyhow::Context;
use clap::Parser;

fn positive_num(value: &str) -> Result<usize, String> {
    match value.parse::<usize>() {
        Ok(n) if n > 0 => Ok(n),
        Ok(n) => Err(format!("Value must be a positive integer, found: {}", n)),
        _ => Err("invalid digit found in string".to_string()),
    }
}

#[derive(Debug, Parser)]
pub struct Args {
    #[arg(num_args = 1..)]
    files: Vec<String>,
    #[arg(short = 'n', long, value_parser = positive_num, conflicts_with = "bytes")]
    lines: Option<usize>,
    #[arg(short = 'c', long, value_parser = positive_num, conflicts_with = "lines")]
    bytes: Option<usize>,
}

enum Mode {
    Lines(usize),
    Bytes(usize),
}

impl Args {
    fn mode(&self) -> Mode {
        if let Some(bytes) = self.bytes {
            Mode::Bytes(bytes)
        } else {
            Mode::Lines(self.lines.unwrap_or(10))
        }
    }
}

pub fn get_args() -> Args {
    let args = Args::parse();
    args
}

fn open(filepath: impl AsRef<Path>) -> anyhow::Result<Box<dyn BufRead>> {
    let filepath = filepath.as_ref();
    match filepath.to_str() {
        Some("-") => {
            let stdin = io::stdin();
            Ok(Box::new(BufReader::new(stdin.lock())))
        }
        _ => {
            let file = File::open(filepath)?;
            Ok(Box::new(BufReader::new(file)))
        }
    }
}

fn head_bytes(file: &mut impl BufRead, n: usize) -> Result<String, anyhow::Error> {
    let mut buf = vec![0; n];
    let read = file
        .take(n as u64)
        .read(&mut buf)
        .context("Failed to read")?;
    Ok(String::from_utf8_lossy(&buf[..read]).into())
}

fn head_lines(file: &mut impl BufRead, n: u64) {
    let mut line = String::new();
    for _ in 0..n {
        let read = match file.read_line(&mut line) {
            Ok(read) => read,
            Err(_) => {
                eprintln!("error: unexpected error");
                return;
            }
        };
        if read == 0 {
            break;
        }
        print!("{}", line);
        line.clear();
    }
}

pub fn run(args: Args) {
    let file_count = args.files.len();
    for (i, filepath) in args.files.iter().enumerate() {
        if file_count > 1 {
            println!("==> {} <==", filepath);
        }
        let mut file = match open(&filepath) {
            Ok(file) => file,
            Err(_) => {
                eprint!("{}: .* (os error 2)", filepath);
                return;
            }
        };

        match &args.mode() {
            Mode::Lines(n) => {
                head_lines(&mut file, *n as u64);
            }
            Mode::Bytes(n) => {
                let haeded = head_bytes(&mut file, *n).unwrap();
                print!("{}", haeded);
            }
        }
        let is_last = i == file_count - 1;
        if file_count > 1 && !is_last {
            println!();
        }
    }
}

#[cfg(test)]
mod tests {

    use tempfile::tempdir;

    use super::*;

    // TODO: ファイルアクセスを行わないテストへ変更する
    #[test]
    fn バイト数オプション_ファイルのバイト数より指定されたバイト数のが多いとき_ファイルのデータをすべて出力する(
    ) {
        // Arrange
        let dir = tempdir().unwrap();
        let filepath = dir.path().join("one.txt");
        std::fs::write(&filepath, "Öne line, four words.\n").unwrap();

        // Act
        let mut file = std::io::BufReader::new(std::fs::File::open(&filepath).unwrap());
        let actual = head_bytes(&mut file, 100).unwrap();

        // Assert
        assert_eq!(actual, "Öne line, four words.\n");
    }
}
