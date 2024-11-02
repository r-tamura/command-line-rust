use clap::{arg, Parser};

#[derive(Debug, Parser)]
pub struct Args {
    in_file: String,
    out_file: String,
    #[arg(short, long)]
    count: bool,
}

pub struct Config {
    pub in_file: String,
    pub out_file: String,
    pub count: bool,
}

impl From<Args> for Config {
    fn from(args: Args) -> Self {
        Config {
            in_file: args.in_file,
            out_file: args.out_file,
            count: args.count,
        }
    }
}

#[derive(Debug)]
pub struct GetArgsError;

pub fn get_args() -> Result<Config, GetArgsError> {
    let args = Args::parse();
    Ok(Config::from(args))
}

pub enum UniqrError {
    UnexpectedError(Box<dyn std::error::Error>),
}

impl std::fmt::Display for UniqrError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            UniqrError::UnexpectedError(_) => write!(f, "処理中に不明なエラーが発生しました"),
        }
    }
}

impl std::error::Error for UniqrError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            UniqrError::UnexpectedError(e) => Some(e.as_ref()),
        }
    }
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

pub fn run(_config: Config) -> Result<(), UniqrError> {
    Ok(())
}
