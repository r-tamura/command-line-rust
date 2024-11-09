use clap::{builder, Parser, ValueEnum};
use regex::Regex;

#[derive(Debug, Eq, PartialEq, Clone, ValueEnum)]
pub enum EntryType {
    Dir,
    File,
    Link,
}

#[derive(Parser)]
struct Args {
    paths: Vec<String>,
    #[arg[short, long]]
    names: Vec<String>,
    #[arg(short, long, value_parser = builder::PossibleValuesParser::new(&["f", "d", "l"]), num_args = 0..)]
    types: Vec<String>,
}

#[derive(Debug)]
pub struct Config {
    pub paths: Vec<String>,
    pub names: Vec<Regex>,
    pub entry_types: Vec<EntryType>,
}

pub fn get_args() -> Config {
    let args = Args::parse();

    let paths = if args.paths.is_empty() {
        vec![".".to_string()]
    } else {
        args.paths
    };

    let entry_types = args
        .types
        .iter()
        .map(|t| match t.as_str() {
            "f" => EntryType::File,
            "d" => EntryType::Dir,
            "l" => EntryType::Link,
            _ => unreachable!("Invalid type"),
        })
        .collect();

    let names = args
        .names
        .iter()
        .map(|n| Regex::new(n).map_err(|_| format!(r#"Invalid --name "{}"#, n)))
        .collect::<Result<Vec<_>, _>>()
        .unwrap_or_default();

    Config {
        paths,
        names,
        entry_types,
    }
}

pub fn run(config: &Config) {
    println!("{:?}", config);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_learning_unwrap_or_default() {
        let err_vec: Result<Vec<String>, _> = Err(vec!["abc".to_string()]);

        let res = err_vec.unwrap_or_default();

        assert_eq!(res, vec![] as Vec<String>);
    }

    #[test]
    fn test_learning_regx() {
        let re = Regex::new(".*[.]csv").unwrap();

        assert!(re.is_match("abc.csv"), "'abc.csv' should match");
        assert!(!re.is_match("abc.txt"), "'abc.txt' should not match");
    }

    #[test]
    fn test_コマンドライン引数のパース_探索対象のパスが指定されていないとき_現在のディレクトリを対象にする(
    ) {
        let config = get_args();

        assert_eq!(config.paths, vec![".".to_string()]);
    }
}
