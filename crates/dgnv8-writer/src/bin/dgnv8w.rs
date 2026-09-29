//! Command line entry point.
//!
//! ```text
//! dgnv8w inspect <seed.dgn> [model-index]   print what the writer sees in a seed
//! dgnv8w build <job.json>                   append the job's elements, write output
//! ```

use std::path::Path;
use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let result = match args.first().map(String::as_str) {
        Some("inspect") if args.len() >= 2 => inspect(&args[1], args.get(2)),
        Some("build") if args.len() == 2 => build(&args[1]),
        Some("--version") => {
            println!("dgnv8w {}", env!("CARGO_PKG_VERSION"));
            Ok(())
        }
        _ => {
            eprintln!(
                "usage: dgnv8w inspect <seed.dgn> [model-index]\n       dgnv8w build <job.json>"
            );
            return ExitCode::from(2);
        }
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("error: {error}");
            ExitCode::FAILURE
        }
    }
}

fn inspect(path: &str, model: Option<&String>) -> dgnv8_writer::Result<()> {
    let model = match model {
        Some(text) => Some(text.parse().map_err(|_| {
            dgnv8_writer::WriteError::InvalidInput(format!("bad model index {text:?}"))
        })?),
        None => None,
    };
    let info = dgnv8_writer::analyse(&std::fs::read(path)?, model)?;
    println!("{}", serde_json::to_string_pretty(&info).unwrap());
    Ok(())
}

fn build(job: &str) -> dgnv8_writer::Result<()> {
    let (output, report) = dgnv8_writer::spec::run_job(Path::new(job))?;
    println!("wrote {}", output.display());
    println!("{}", serde_json::to_string_pretty(&report).unwrap());
    Ok(())
}
