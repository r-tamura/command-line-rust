use std::{
    error::Error,
    io::{self, BufRead, BufReader},
};

use clap::{command, Parser};
type MyResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug, Parser)]
#[command(version, about, long_about=None)]
pub struct Config {
    #[arg(value_name = "FILE", default_value = "-", num_args=1..)]
    files: Vec<String>,
    #[arg(short = 'n', long, conflicts_with = "number_nonblank_lines")]
    number_lines: bool,
    #[arg(short = 'b', long, conflicts_with = "number_lines")]
    number_nonblank_lines: bool,
    #[arg(short = 's', long)]
    squeeze_blank: bool,
}

pub fn get_args() -> MyResult<Config> {
    let args = Config::parse();
    Ok(args)
}

fn open(filename: &str) -> MyResult<Box<dyn BufRead>> {
    match filename {
        "-" => Ok(Box::new(BufReader::new(io::stdin()))),
        _ => Ok(Box::new(BufReader::new(std::fs::File::open(filename)?))),
    }
}

enum LineNumbering {
    None,
    All,
    Nonblank,
}

struct State {
    prev_line_num: usize,
    is_prev_blank: bool,
}

impl State {
    fn new() -> Self {
        Self {
            prev_line_num: 1,
            is_prev_blank: false,
        }
    }

    fn increment(self) -> Self {
        Self {
            prev_line_num: self.prev_line_num + 1,
            ..self
        }
    }

    fn blank(self) -> Self {
        Self {
            is_prev_blank: true,
            ..self
        }
    }

    fn nonblank(self) -> Self {
        Self {
            is_prev_blank: false,
            ..self
        }
    }
}

pub fn run(config: Config) -> MyResult<()> {
    for filename in config.files {
        match open(&filename) {
            Err(err) => eprintln!("{}: {}", filename, err),
            Ok(file) => {
                let mut state = State::new();
                for (line_num, line) in file.lines().enumerate() {
                    let line = line?;
                    let numbering_type = if config.number_lines {
                        LineNumbering::All
                    } else if config.number_nonblank_lines {
                        LineNumbering::Nonblank
                    } else {
                        LineNumbering::None
                    };
                    let is_current_blank = line.trim().is_empty();

                    if config.squeeze_blank && state.is_prev_blank && is_current_blank {
                        continue;
                    }

                    match numbering_type {
                        LineNumbering::All => {
                            println!("{:6}\t{line}", line_num + 1);
                        }
                        LineNumbering::Nonblank => {
                            if is_current_blank {
                                println!();
                            } else {
                                println!("{:6}\t{line}", state.prev_line_num);
                                state = state.increment();
                            }
                        }
                        LineNumbering::None => {
                            println!("{}", line);
                        }
                    }
                    state = if is_current_blank {
                        state.blank()
                    } else {
                        state.nonblank()
                    };
                }
            }
        }
    }
    Ok(())
}
