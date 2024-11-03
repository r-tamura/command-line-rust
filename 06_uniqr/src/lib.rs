mod error;

use crate::error::UniqrError;

use clap::{arg, Parser};

#[derive(Debug, Parser)]
pub struct Args {
    in_file: Option<String>,
    out_file: Option<String>,
    #[arg(short, long)]
    count: bool,
    #[arg(long)]
    debug: bool,
}

pub struct Config {
    pub in_file: Option<String>,
    pub out_file: Option<String>,
    pub count: bool,
    pub debug: bool,
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

pub fn run(config: Config) -> Result<String, UniqrError> {
    let mut reader = open(config.in_file.unwrap_or("-".to_string()).as_str())?;
    let mut line = String::new();

    let mut result = String::new();
    loop {
        let read = reader.read_line(&mut line)?;
        if read == 0 {
            break;
        }
        result.push_str(line.as_str());
        line.clear();
    }

    Ok(result)
}
