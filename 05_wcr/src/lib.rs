use std::{
    error::Error,
    fs::File,
    io::{self, BufRead, BufReader},
    path::Path,
};

use clap::{command, Parser};

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
            Ok(_) => println!("Opened {}", filepath),
        }
    }
    Ok(())
}
