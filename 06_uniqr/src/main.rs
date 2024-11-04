use std::io::Write;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = uniqr::get_args().expect("Error parsing arguments");
    let debug = config.debug;
    match uniqr::run(config.clone()) {
        Ok(result) => {
            let mut out_file: Box<dyn Write> = match config.out_file {
                Some(out_file) => Box::new(std::fs::File::create(out_file)?),
                None => Box::new(std::io::stdout()),
            };

            write!(out_file, "{}", result)?;
            Ok(())
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            if debug == true {
                eprintln!("{:?}", e);
            }
            std::process::exit(1);
        }
    }
}
