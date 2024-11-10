use clap::{builder, Parser, ValueEnum};
use regex::Regex;

use anyhow::Context;
use walkdir::{DirEntry, WalkDir};

#[derive(Debug, Eq, PartialEq, Clone, ValueEnum)]
pub enum EntryType {
    Dir,
    File,
    Link,
}

#[derive(Parser)]
struct Args {
    paths: Vec<String>,
    #[arg[short, long = "name"]]
    names: Vec<String>,
    #[arg(short, long = "type", value_parser = builder::PossibleValuesParser::new(&["f", "d", "l"]), num_args = 0..)]
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
        .map(|n| Regex::new(n).with_context(|| format!("error: invalid value '{}'", n)))
        .collect::<Result<Vec<_>, _>>()
        .unwrap();

    Config {
        paths,
        names,
        entry_types,
    }
}

fn matches_entry_type(config: &Config, entry: &DirEntry) -> bool {
    let file_type = entry.file_type();
    let entry_type = if file_type.is_dir() {
        EntryType::Dir
    } else if file_type.is_file() {
        EntryType::File
    } else if file_type.is_symlink() {
        EntryType::Link
    } else {
        return false;
    };

    config.entry_types.contains(&entry_type)
}

pub fn run(config: &Config) {
    for path in &config.paths {
        for entry in WalkDir::new(path).into_iter().filter(|e| match e {
            Err(e) => {
                eprintln!("error: {}", e);
                return false;
            }
            Ok(e) => matches_entry_type(&config, &e),
        }) {
            let entry = match entry {
                Err(e) => {
                    eprintln!("error: {}", e);
                    continue;
                }
                Ok(e) => e,
            };

            println!("{}", entry.path().display());
        }
    }
}

#[cfg(test)]
mod learning_tests {
    use super::*;

    #[test]
    fn test_learning_unwrap_or_default() {
        let err_vec: Result<Vec<String>, _> = Err(vec!["abc".to_string()]);

        let res = err_vec.unwrap_or_default();

        assert_eq!(res, vec![] as Vec<String>);
    }

    #[test]
    fn test_learning_regex() {
        let re = Regex::new(".*[.]csv").unwrap();

        assert!(re.is_match("abc.csv"), "'abc.csv' should match");
        assert!(!re.is_match("abc.txt"), "'abc.txt' should not match");
    }

    #[test]
    fn test_learning_不正な正規表現文字列のパースに失敗する() {
        let re = Regex::new("*.txt");

        assert!(re.is_err());
    }

    #[test]
    fn learning_resultのvec型をのvecのresult型に変換するとき_ループは途中で中断される() {
        let vec = vec![1, 2, 3];

        let mut count = 0;
        let res: Result<Vec<i32>, String> = vec
            .into_iter()
            .map(|e| {
                println!("--- {}", e);
                count += 1;
                if e == 1 {
                    Err("one".into())
                } else {
                    Ok(e)
                }
            })
            .collect();

        assert_eq!(res, Err("one".into()));
        assert_eq!(count, 1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn コマンドライン引数のパース_探索対象のパスが指定されていないとき_現在のディレクトリを対象にする(
    ) {
        let config = get_args();

        assert_eq!(config.paths, vec![".".to_string()]);
    }
}
