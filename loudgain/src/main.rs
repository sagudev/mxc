use std::process::ExitCode;

fn main() -> ExitCode {
    ExitCode::from(
        loudgain::loudgain()
            // first arg is executable name
            .args(std::env::args().skip(1))
            .status()
            .unwrap()
            .code()
            .unwrap() as u8,
    )
}
