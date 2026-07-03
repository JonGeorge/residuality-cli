use std::process::ExitCode;

fn main() -> ExitCode {
    match residuality::run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("{e}");
            ExitCode::FAILURE
        }
    }
}
