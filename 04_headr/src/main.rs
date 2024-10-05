use std::process::ExitCode;

fn main() -> ExitCode {
    let args = headr::get_args();
    headr::run(args);
    ExitCode::SUCCESS
}
