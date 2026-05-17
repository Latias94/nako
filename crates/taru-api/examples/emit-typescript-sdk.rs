use std::{env, ffi::OsString, fs, path::PathBuf, process::ExitCode};

fn main() -> ExitCode {
    let args = env::args_os().skip(1).collect::<Vec<_>>();
    if args.is_empty() {
        print!("{}", taru_api::sdk::typescript_sdk());
        return ExitCode::SUCCESS;
    }

    if args.len() == 2 && args[0] == OsString::from("--output") {
        return write_sdk(PathBuf::from(&args[1]));
    }

    eprintln!("usage: emit-typescript-sdk [--output <path>]");
    ExitCode::from(2)
}

fn write_sdk(path: PathBuf) -> ExitCode {
    if let Some(parent) = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        && let Err(error) = fs::create_dir_all(parent)
    {
        eprintln!("failed to create {}: {error}", parent.display());
        return ExitCode::FAILURE;
    }

    if let Err(error) = fs::write(&path, taru_api::sdk::typescript_sdk()) {
        eprintln!("failed to write {}: {error}", path.display());
        return ExitCode::FAILURE;
    }

    ExitCode::SUCCESS
}
