mod error;

use crate::error::UniqrError;

use clap::{arg, Parser};

#[derive(Debug, Parser)]
pub struct Args {
    in_file: Option<String>,
    out_file: Option<String>,
    #[arg(short = 'c', long)]
    count: bool,
    #[arg(long)]
    debug: bool,
}

#[derive(Clone, Debug)]
pub struct Config {
    pub in_file: Option<String>,
    pub out_file: Option<String>,
    pub count: bool,
    pub debug: bool,
}

impl Config {
    pub fn new(in_file: Option<String>, out_file: Option<String>, count: bool) -> Self {
        Config {
            in_file,
            out_file,
            count,
            debug: false,
        }
    }
}

pub struct ConfigBuilder {
    in_file: Option<String>,
    out_file: Option<String>,
    count: bool,
    debug: bool,
}

impl ConfigBuilder {
    pub fn new() -> Self {
        ConfigBuilder::default()
    }

    pub fn stdin(mut self) -> Self {
        self.in_file = Some("-".to_string());
        self
    }

    pub fn in_file(mut self, in_file: Option<String>) -> Self {
        self.in_file = in_file;
        self
    }

    pub fn out_file(mut self, out_file: Option<String>) -> Self {
        self.out_file = out_file;
        self
    }

    pub fn count(mut self, count: bool) -> Self {
        self.count = count;
        self
    }

    pub fn debug(mut self, debug: bool) -> Self {
        self.debug = debug;
        self
    }

    pub fn build(self) -> Config {
        Config {
            in_file: self.in_file,
            out_file: self.out_file,
            count: self.count,
            debug: self.debug,
        }
    }
}

impl Default for ConfigBuilder {
    fn default() -> Self {
        ConfigBuilder {
            in_file: None,
            out_file: None,
            count: false,
            debug: false,
        }
    }
}

impl From<Args> for Config {
    fn from(args: Args) -> Self {
        Config {
            in_file: args.in_file,
            out_file: args.out_file,
            count: args.count,
            debug: args.debug,
        }
    }
}

#[derive(Debug)]
pub struct GetArgsError;

pub fn get_args() -> Result<Config, GetArgsError> {
    let args = Args::parse();
    Ok(Config::from(args))
}

fn error_chain(e: impl std::error::Error, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    let mut source = e.source();
    while let Some(e) = source {
        write!(f, "Caused by: {}", e)?;
        source = e.source();
    }
    Ok(())
}

impl std::fmt::Debug for UniqrError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        error_chain(self, f)
    }
}

fn open(filepath: &str) -> Result<Box<dyn std::io::BufRead>, UniqrError> {
    match filepath {
        "-" => {
            let stdin = std::io::stdin();
            Ok(Box::new(std::io::BufReader::new(stdin.lock())))
        }
        _ => {
            let file = std::fs::File::open(filepath)
                .map_err(|e| UniqrError::FileNotFoundError(e, filepath.to_string()))?;
            Ok(Box::new(std::io::BufReader::new(file)))
        }
    }
}

fn format_line(count: u32, line: &str, show_count: bool) -> String {
    if show_count {
        format!("{:4} {}", count, line)
    } else {
        line.to_string()
    }
}

pub fn _run<R: std::io::BufRead>(config: &Config, r: R) -> Result<String, UniqrError> {
    let mut reader = r;
    let mut line = String::new();

    let mut result = String::new();
    let mut prev_line = String::new();
    let mut count = 1;

    let read = reader.read_line(&mut prev_line)?;
    if read == 0 {
        return Ok(result);
    }

    loop {
        let read = reader.read_line(&mut line)?;

        if read == 0 {
            result.push_str(&format_line(count, prev_line.as_str(), config.count));
            return Ok(result);
        }

        while prev_line.trim() == line.trim() {
            count += 1;
            line.clear();
            let read = reader.read_line(&mut line)?;
            if read == 0 {
                result.push_str(&format_line(count, prev_line.as_str(), config.count));
                return Ok(result);
            }
        }

        result.push_str(&format_line(count, prev_line.as_str(), config.count));

        prev_line = line.clone();
        line.clear();
        count = 1;
    }
}

pub fn run(config: Config) -> Result<String, UniqrError> {
    let in_file = (&config).in_file.clone().unwrap_or("-".to_string());
    let mut reader = open(in_file.as_str())?;
    _run(&config, reader.as_mut())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_two_count_same() {
        let config = ConfigBuilder::default().count(true).build();
        let input = "a\na\n";
        let expected = "   2 a\n";
        let actual = _run(&config, Cursor::new(input)).unwrap();
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_two_count_different() {
        let config = ConfigBuilder::default().count(true).build();
        let input = "a\nb\n";
        let expected = "   1 a\n   1 b\n";
        let actual = _run(&config, Cursor::new(input)).unwrap();
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_three_count_different() {
        let config = ConfigBuilder::default().count(true).build();
        let input = "a\na\nb\n";
        let expected = "   2 a\n   1 b\n";
        let actual = _run(&config, Cursor::new(input)).unwrap();
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_改行コードがある行とない行では区別されない() {
        let config = ConfigBuilder::default().count(true).build();
        let input = "a\na";
        let expected = "   2 a\n";
        let actual = _run(&config, Cursor::new(input)).unwrap();
        assert_eq!(actual, expected);
    }
}
