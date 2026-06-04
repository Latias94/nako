use std::{env, fs, path::PathBuf, process::ExitCode};

use nako_transcode::{FfmpegHardwareAccelerationDetector, HardwareAccelerationDetector};

const USAGE: &str = "\
Usage: cargo run -p nako-transcode --example hardware-report -- [--ffmpeg PATH] [--output PATH]

Writes the redacted FFmpeg hardware acceleration report as JSON.
";

#[derive(Debug)]
struct Args {
    ffmpeg_path: PathBuf,
    output_path: Option<PathBuf>,
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("{message}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), String> {
    let Some(args) = parse_args(env::args().skip(1))? else {
        println!("{USAGE}");
        return Ok(());
    };

    let report = FfmpegHardwareAccelerationDetector::new(args.ffmpeg_path).detect();
    let report_json = serde_json::to_string_pretty(&report)
        .map_err(|err| format!("failed to serialize hardware report: {err}"))?;

    if let Some(output_path) = args.output_path {
        if let Some(parent) = output_path
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
        {
            fs::create_dir_all(parent).map_err(|err| {
                format!(
                    "failed to create hardware report directory {}: {err}",
                    parent.display()
                )
            })?;
        }

        fs::write(&output_path, format!("{report_json}\n")).map_err(|err| {
            format!(
                "failed to write hardware report {}: {err}",
                output_path.display()
            )
        })?;
        println!("Hardware report written to {}", output_path.display());
    } else {
        println!("{report_json}");
    }

    Ok(())
}

fn parse_args(args: impl IntoIterator<Item = String>) -> Result<Option<Args>, String> {
    let mut ffmpeg_path = PathBuf::from("ffmpeg");
    let mut output_path = None;
    let mut args = args.into_iter();

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "-h" | "--help" => return Ok(None),
            "--ffmpeg" => {
                let value = args
                    .next()
                    .ok_or_else(|| format!("missing value for --ffmpeg\n\n{USAGE}"))?;
                ffmpeg_path = PathBuf::from(value);
            }
            "--output" => {
                let value = args
                    .next()
                    .ok_or_else(|| format!("missing value for --output\n\n{USAGE}"))?;
                output_path = Some(PathBuf::from(value));
            }
            _ => return Err(format!("unknown argument: {arg}\n\n{USAGE}")),
        }
    }

    Ok(Some(Args {
        ffmpeg_path,
        output_path,
    }))
}
