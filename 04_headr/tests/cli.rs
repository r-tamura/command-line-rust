use std::{
    fs::{self, File},
    io::Read,
    os::unix::process::ExitStatusExt,
    process,
};

use assert_cmd::Command;
use predicates::prelude::*;
use pretty_assertions::assert_eq;
use rand::{distributions::Alphanumeric, Rng};

const PRG: &str = "headr";
const EMPTY: &str = "./tests/inputs/empty.txt";
const ONE: &str = "./tests/inputs/one.txt";
const TWO: &str = "./tests/inputs/two.txt";
const THREE: &str = "./tests/inputs/three.txt";
const TWELVE: &str = "./tests/inputs/twelve.txt";

fn random_string() -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(7)
        .map(char::from)
        .collect()
}

fn gen_bad_file() -> String {
    loop {
        let filename = random_string();
        if fs::metadata(&filename).is_err() {
            return filename;
        }
    }
}

#[test]
fn dies_bad_bytes() {
    let bad = random_string();
    let expected = format!(
        "invalid value '{bad}' for \
    '--bytes <BYTES>': invalid digit found in string"
    );

    Command::cargo_bin(PRG)
        .expect("Failed to run command")
        .args(["-c", &bad, EMPTY])
        .assert()
        .failure()
        .stderr(predicate::str::contains(expected));
}

#[test]
fn dies_bad_lines() {
    let bad = random_string();
    let expected = format!(
        "error: invalid value '{bad}' for \
        '--lines <LINES>': invalid digit found in string"
    );
    Command::cargo_bin(PRG)
        .expect("Failed to run command")
        .args(["-n", &bad, EMPTY])
        .assert()
        .failure()
        .stderr(predicate::str::contains(expected));
}

#[test]
fn dies_bytes_and_lines() {
    let msg = "the argument '--lines <LINES>' cannot be \
               used with '--bytes <BYTES>'";

    Command::cargo_bin(PRG)
        .expect("Failed to run command")
        .args(["-n", "1", "-c", "2"])
        .assert()
        .failure()
        .stderr(predicate::str::contains(msg));
}

#[test]
fn skips_bad_file() {
    let bad = gen_bad_file();
    let expected = format!("{bad}: .* [(]os error 2[)]");

    Command::cargo_bin(PRG)
        .expect("Failed to run command")
        .arg(&bad)
        .assert()
        .success()
        .stderr(predicate::str::is_match(expected).unwrap());
}

fn run_file(args: &[&str]) -> String {
    let output = Command::cargo_bin(PRG)
        .expect("failed to run command")
        .args(args)
        .output()
        .expect("fail");
    assert_eq!(output.status, process::ExitStatus::from_raw(0));
    String::from_utf8_lossy(&output.stdout).into()
}

fn run_stdin(args: &[&str], stdin: &str) -> String {
    let mut args = args.to_vec();
    args.push("-");
    let output = Command::cargo_bin(PRG)
        .expect("failed to run command")
        .args(&args)
        .write_stdin(stdin)
        .output()
        .expect("fail");
    assert_eq!(output.status, process::ExitStatus::from_raw(0));
    String::from_utf8_lossy(&output.stdout).into()
}

fn assert_eq_with_file(actual: impl AsRef<str>, expected_file: impl AsRef<str>) {
    let mut file = File::open(expected_file.as_ref()).expect("file not found");
    let mut buffer = String::new();
    file.read_to_string(&mut buffer)
        .expect("Failed to read file");
    let expected: &str = buffer.as_ref();

    // assert_eq!(actual.as_ref().as_bytes(), expected.as_bytes());
    assert_eq!(actual.as_ref(), expected);
}

#[test]
fn empty() {
    // Act
    let actual = run_file(&["-n", "1", EMPTY]);
    // Assert
    assert_eq_with_file(actual, "tests/expected/empty.txt.out");
}

#[test]
fn 空ファイルに行数オプションを指定したとき_空ファイルが出力される() {
    // Act
    let actual = run_file(&["-n", "2", EMPTY]);
    // Assert
    assert_eq_with_file(actual, "tests/expected/empty.txt.n2.out");
}

#[test]
fn 空ファイルにバイト数オプションを指定したとき_空ファイルが出力される() {
    // Act
    let actual = run_file(&["-c", "2", EMPTY]);
    // Assert
    assert_eq_with_file(actual, "tests/expected/empty.txt.c2.out");
}

#[test]
fn 行数オプションで指定された行数より行数の入力データの行数が小さいとき_入力ファイルのすべての行が出力される(
) {
    // Act
    let actual = run_file(&["-n", "2", ONE]);
    // Assert
    assert_eq_with_file(actual, "tests/expected/one.txt.n2.out");
}

#[test]
fn 行数オプションで指定された行数より行数の入力データの行数が大きいとき_入力ファイルのすべての行が出力される(
) {
    // Act
    let actual = run_file(&["-n", "2", TWELVE]);
    // Assert
    assert_eq_with_file(actual, "tests/expected/twelve.txt.n2.out");
}

#[test]
fn 入力ファイルの改行コードと同じ改行コードが出力される() {
    // Act
    let actual = run_stdin(&["-n", "2"], "abc\r\ndef\n");
    // Assert
    println!("===================");
    println!("{}", actual);
    println!("===================");
    assert_eq!(actual, "abc\r\ndef\n");
}

#[test]
fn バイト数オプションで指定されたバイト数より入力データのバイト数が大きいとき_入力ファイルのすべてのバイトが出力される(
) {
    // Act
    let actual = run_file(&["-c", "100", ONE]);
    // Assert
    assert_eq_with_file(actual, ONE);
}

#[test]
fn バイト数オプションで指定されたバイト数nが入力データの総バイト数mより小さいとき_入力ファイルのうち先頭nバイトが出力される(
) {
    // Act
    let actual = run_file(&["-c", "2", ONE]);
    // Assert
    assert_eq_with_file(actual, "tests/expected/one.txt.c2.out");
}

#[test]
fn 複数のファイルを指定されたとき_それぞれのファイルがセパレータで区切られて出力される() {
    // Act
    let actual = run_file(&[EMPTY, ONE, TWO, THREE, TWELVE]);
    // Assert
    assert_eq_with_file(actual, "tests/expected/all.out");
}

#[test]
fn 複数ファイルを指定されたとき_それぞれのファイルに対して指定されたオプションが適用される() {
    // Act
    let actual = run_file(&["-n", "2", EMPTY, ONE, TWO, THREE, TWELVE]);
    // Assert
    assert_eq_with_file(actual, "tests/expected/all.n2.out");
}
