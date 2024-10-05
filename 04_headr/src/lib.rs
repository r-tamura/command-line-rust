use std::{io::BufRead, path::Path};

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
    let file = std::fs::File::open(filepath)?;
    let reader = std::io::BufReader::new(file);
    Ok(Box::new(reader))
}

pub fn run(args: Args) {
    for file in &args.files {
        let mut file = match open(&file) {
            Ok(file) => file,
            Err(_) => {
                eprint!("{}: .* (os error 2)", file);
                return;
            }
        };

        match &args.mode() {
            Mode::Lines(n) => {
                for _ in 0..*n {
                    let mut line = String::new();
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
                }
            }
            Mode::Bytes(n) => {
                let mut buffer = vec![0; *n + 1];
                let read = match file.read(&mut buffer) {
                    Ok(read) => read,
                    Err(_) => {
                        eprintln!("error: unexpected error");
                        return;
                    }
                };
                if read == 0 {
                    break;
                }

                let mut buffer = buffer.into_iter().filter(|b| *b != 0).collect::<Vec<u8>>();
                buffer.pop();
                print!("{}", String::from_utf8_lossy(&buffer));
            }
        }
    }
}
