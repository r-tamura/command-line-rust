fn main() {
    let config = uniqr::get_args().expect("Error parsing arguments");
    let debug = config.debug;
    match uniqr::run(config.clone()) {
        Ok(result) => match config.out_file {
            Some(out_file) => {
                std::fs::write(out_file, result).expect("Error writing to file");
            }
            None => {
                print!("{}", result)
            }
        },
        Err(e) => {
            eprintln!("Error: {}", e);
            if debug == true {
                eprintln!("{:?}", e);
            }
            std::process::exit(1);
        }
    }
}
