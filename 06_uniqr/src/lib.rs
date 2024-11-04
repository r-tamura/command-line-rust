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

struct UniqIterItem {
    count: u32,
    line: String,
}

struct UniqIter<R: std::io::BufRead> {
    reader: R,
    prev_line: String,
    line: String,
    count: u32,
    finished: bool,
}

impl<R: std::io::BufRead> UniqIter<R> {
    fn new(reader: R) -> Self {
        UniqIter {
            reader,
            prev_line: String::new(),
            line: String::new(),
            count: 1,
            finished: false,
        }
    }

    fn found(&self, count: u32, line: String) -> Option<Result<UniqIterItem, UniqrError>> {
        Some(Ok(UniqIterItem { count, line }))
    }

    fn finish(&mut self) -> Option<Result<UniqIterItem, UniqrError>> {
        self.finished = true;
        self.found(self.count, self.prev_line.clone())
    }

    pub fn init(&mut self) -> Result<(), UniqrError> {
        let mut prev_line = String::new();
        let read = self.reader.read_line(&mut prev_line)?;

        if read == 0 {
            self.finished = true;
        }
        self.prev_line = prev_line;
        Ok(())
    }
}

impl<R: std::io::BufRead> Iterator for UniqIter<R> {
    type Item = Result<UniqIterItem, UniqrError>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.finished {
            return None;
        }

        let read = match self.reader.read_line(&mut self.line) {
            Ok(read) => read,
            Err(e) => return Some(Err(UniqrError::UnexpectedError(Box::new(e)))),
        };
        if read == 0 {
            return self.finish();
        }

        while self.prev_line.trim() == self.line.trim() {
            self.count += 1;
            self.line.clear();
            let read = match self.reader.read_line(&mut self.line) {
                Ok(read) => read,
                Err(e) => return Some(Err(UniqrError::UnexpectedError(Box::new(e)))),
            };
            if read == 0 {
                return self.finish();
            }
        }

        let item = UniqIterItem {
            count: self.count,
            line: self.prev_line.clone(),
        };

        self.prev_line = self.line.clone();
        self.line.clear();
        self.count = 1;

        self.found(item.count, item.line.clone())
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
    let reader = r;
    let mut result = String::new();

    let mut uniq_iter = UniqIter::new(reader);
    uniq_iter.init()?;
    for item in uniq_iter {
        let item = item?;
        let line = item.line;
        let count = item.count;
        result.push_str(format_line(count, line.as_str(), config.count).as_str());
    }

    Ok(result)
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
