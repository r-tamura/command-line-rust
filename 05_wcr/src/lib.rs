use std::{
    error::Error,
    fs::File,
    io::{self, BufRead, BufReader},
    path::Path,
};

use clap::{command, Parser};

#[derive(Debug, PartialEq)]
pub struct FileInfo {
    num_lines: usize,
    num_words: usize,
    num_bytes: usize,
    num_chars: usize,
}

pub fn count(mut handle: impl BufRead) -> Result<FileInfo, Box<dyn Error>> {
    let mut num_lines = 0;
    let mut num_words = 0;
    let mut num_bytes = 0;
    let mut num_chars = 0;

    let mut line = String::new();
    loop {
        let read = handle.read_line(&mut line)?;
        if read == 0 {
            break;
        }
        num_lines += 1;
        num_bytes += line.len();
        num_chars += line.chars().count();
        num_words += line.split_whitespace().count();

        line.clear();
    }

    Ok(FileInfo {
        num_lines,
        num_words,
        num_bytes,
        num_chars,
    })
}

#[cfg(test)]
mod tests {
    use super::{count, FileInfo};
    use std::io::Cursor;

    #[test]
    fn test_count() {
        // Act
        let text = "I don't want the world. I just want your half.\r\n";
        let info = count(Cursor::new(text));

        // Assert
        let expected = FileInfo {
            num_lines: 1,
            num_words: 10,
            num_bytes: 48,
            num_chars: 48,
        };
        assert_eq!(info.unwrap(), expected);
    }
}

#[derive(Debug, Parser)]
#[command(version, about = "Rust wc", long_about = None)]
pub struct Args {
    #[arg(num_args = 0..)]
    files: Vec<String>,
    #[arg(short, long, help = "Show line count")]
    lines: bool,
    #[arg(short, long, help = "Show word count")]
    words: bool,
    #[arg(short = 'c', long, help = "Show byte count", conflicts_with = "chars")]
    bytes: bool,
    #[arg(
        short = 'm',
        long,
        help = "Show character count",
        conflicts_with = "bytes"
    )]
    chars: bool,
}

#[derive(Debug)]
pub enum ByteCharMode {
    Bytes,
    Chars,
}

#[derive(Debug)]
pub struct Config {
    pub files: Vec<String>,
    pub lines: bool,
    pub words: bool,
    pub mode: ByteCharMode,
}

pub fn get_args() -> Result<Config, Box<dyn Error>> {
    let args = Args::parse();

    let mut lines = args.lines;
    let mut words = args.words;
    let mut mode = if args.bytes {
        ByteCharMode::Bytes
    } else {
        ByteCharMode::Chars
    };

    if [lines, words, args.bytes, args.chars].iter().all(|&x| !x) {
        lines = true;
        words = true;
        mode = ByteCharMode::Bytes;
    }

    Ok(Config {
        files: args.files,
        lines: lines,
        words: words,
        mode,
    })
}

fn open(filepath: impl AsRef<Path>) -> Result<Box<dyn BufRead>, Box<dyn Error>> {
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

pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
    for filepath in &config.files {
        match open(filepath) {
            Err(err) => eprintln!("{}: {}", filepath, err),
            Ok(handle) => {
                let info = count(handle)?;
                println!(
                    "\t{}\t{}\t{} {}",
                    info.num_lines, info.num_words, info.num_bytes, filepath
                );
            }
        }
    }
    Ok(())
}
