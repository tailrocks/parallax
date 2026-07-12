use std::process::ExitCode;

fn main() -> ExitCode {
    match parallax_xtask::run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("error: {error:#}");
            ExitCode::FAILURE
        }
    }
}
