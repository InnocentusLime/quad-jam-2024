use std::process::ExitCode;

#[cfg(feature = "dev-env")]
mod devenv;

fn main() -> ExitCode {
    #[cfg(feature = "dev-env")]
    return devenv::run();
    #[cfg(not(feature = "dev-env"))]
    return ExitCode::SUCCESS;
}
