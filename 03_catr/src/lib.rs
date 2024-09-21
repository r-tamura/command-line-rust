use std::error::Error;

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
}

pub fn get_args() -> MyResult<Config> {
    let args = Config::parse();
    Ok(args)
}

pub fn run(config: Config) -> MyResult<()> {
    for filename in config.files {
        println!("{}", filename);
    }
    Ok(())
}
