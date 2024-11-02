fn main() {
    let config = uniqr::get_args().expect("Error parsing arguments");
    let debug = config.debug;
    match uniqr::run(config) {
        Ok(result) => println!("{}", result),
        Err(e) => {
            eprintln!("Error: {}", e);
            if debug == true {
                eprintln!("{:?}", e);
            }
            std::process::exit(1);
        }
    }
}
