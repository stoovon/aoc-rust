use std::{env, error::Error, fs, io, process::ExitCode};
use svutils::Solver;

fn run() -> Result<(), Box<dyn Error>> {
    let args: Vec<_> = env::args_os().skip(1).collect();
    if !(2..=3).contains(&args.len()) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "usage: aoc <part: 1|2> <input-file> [expected-answer-file] (2023 day 3)",
        )
        .into());
    }
    let solver: Solver = match args[0].to_str() {
        Some("1") => aoc2023::d03::part1,
        Some("2") => aoc2023::d03::part2,
        _ => return Err(io::Error::new(io::ErrorKind::InvalidInput, "part must be 1 or 2").into()),
    };
    let input = fs::read_to_string(&args[1])?;
    let answer = solver(&input)?;
    if let Some(path) = args.get(2) {
        let expected = fs::read_to_string(path)?;
        if answer != expected.trim() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "computed answer does not match the expected-answer file",
            )
            .into());
        }
    }
    println!("{answer}");
    Ok(())
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("error: {error}");
            ExitCode::FAILURE
        }
    }
}
