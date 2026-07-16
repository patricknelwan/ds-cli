use std::process::ExitCode;

fn main() -> ExitCode {
    match ds_cli::run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("ds: {error}");
            ExitCode::FAILURE
        }
    }
}
