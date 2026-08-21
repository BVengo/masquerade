use std::env;
use std::process::ExitCode;

fn main() -> ExitCode {
    match masquerade_cli::run(env::args_os().skip(1)) {
        Ok(code) => ExitCode::from(code),
        Err(message) => {
            eprintln!("masquerade: {message}");
            ExitCode::from(2)
        }
    }
}
