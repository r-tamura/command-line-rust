fn main() {
    let config = uniqr::get_args().expect("Error parsing arguments");
    match uniqr::run(config) {
        Ok(config) => println!("{:?}", config),
        Err(e) => eprintln!("Error: {:?}", e),
    }
}
