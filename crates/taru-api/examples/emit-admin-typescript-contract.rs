use std::{env, ffi::OsString, fs, path::PathBuf, process::ExitCode};

fn main() -> ExitCode {
    let args = env::args_os().skip(1).collect::<Vec<_>>();
    if args.is_empty() {
        print!("{}", taru_api::admin_contract::admin_typescript_contract());
        return ExitCode::SUCCESS;
    }

    if args.len() == 2 && args[0] == OsString::from("--output") {
        return write_contract(PathBuf::from(&args[1]));
    }

    eprintln!("usage: emit-admin-typescript-contract [--output <path>]");
    ExitCode::from(2)
}

fn write_contract(path: PathBuf) -> ExitCode {
    if let Some(parent) = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        && let Err(error) = fs::create_dir_all(parent)
    {
        eprintln!("failed to create {}: {error}", parent.display());
        return ExitCode::FAILURE;
    }

    if let Err(error) = fs::write(&path, taru_api::admin_contract::admin_typescript_contract()) {
        eprintln!("failed to write {}: {error}", path.display());
        return ExitCode::FAILURE;
    }

    ExitCode::SUCCESS
}
