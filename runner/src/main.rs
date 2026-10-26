use std::{env, error::Error, fs, io, process::ExitCode};
use svutils::Solver;

struct Day {
    part1: Solver,
    part2: Option<Solver>,
}

impl Day {
    fn solver(&self, part: u16) -> io::Result<Solver> {
        match part {
            1 => Ok(self.part1),
            2 => self.part2.ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "part 2 is not available for this day",
                )
            }),
            _ => Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "part must be 1 or 2",
            )),
        }
    }
}

fn select_solver(year: u16, day: u16, part: u16) -> io::Result<Solver> {
    let registered = match (year, day) {
        (2023, 1) => Day {
            part1: aoc2023::d01::part1,
            part2: Some(aoc2023::d01::part2),
        },
        (2023, 2) => Day {
            part1: aoc2023::d02::part1,
            part2: Some(aoc2023::d02::part2),
        },
        (2023, 3) => Day {
            part1: aoc2023::d03::part1,
            part2: Some(aoc2023::d03::part2),
        },
        (2023, 4) => Day {
            part1: aoc2023::d04::part1,
            part2: Some(aoc2023::d04::part2),
        },
        (2023, 5) => Day {
            part1: aoc2023::d05::part1,
            part2: Some(aoc2023::d05::part2),
        },
        (2023, 6) => Day {
            part1: aoc2023::d06::part1,
            part2: Some(aoc2023::d06::part2),
        },
        _ => {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("no solver registered for {year} day {day} part {part}"),
            ))
        }
    };
    registered.solver(part)
}

fn run() -> Result<(), Box<dyn Error>> {
    let args: Vec<_> = env::args_os().skip(1).collect();
    if !(4..=5).contains(&args.len()) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "usage: aoc-runner <year> <day> <part> <input-file> [expected-answer-file]",
        )
        .into());
    }
    let parse_coordinate = |index: usize, name: &str| -> Result<u16, Box<dyn Error>> {
        let value = args[index].to_str().ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("{name} must be an integer"),
            )
        })?;
        Ok(value.parse().map_err(|_| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("{name} must be an integer from 0 to 65535"),
            )
        })?)
    };
    let year = parse_coordinate(0, "year")?;
    let day = parse_coordinate(1, "day")?;
    let part = parse_coordinate(2, "part")?;
    let solver = select_solver(year, day, part)?;
    let input = fs::read_to_string(&args[3])?;
    let answer = solver(&input)?;
    if let Some(path) = args.get(4) {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn race_solvers_are_registered_and_propagate_errors() {
        let input = "Time: 1 02\nDistance: 0 00";
        assert_eq!(select_solver(2023, 6, 1).unwrap()(input).unwrap(), "0");
        assert_eq!(select_solver(2023, 6, 2).unwrap()(input).unwrap(), "101");
        assert!(select_solver(2023, 6, 1).unwrap()("Time: 8 10\nDistance: 12").is_err());
    }

    #[test]
    fn almanac_solvers_are_registered_and_propagate_errors() {
        let input = "seeds: 12 3\n\nseed-to-location map:\n100 12 3";
        assert_eq!(select_solver(2023, 5, 1).unwrap()(input).unwrap(), "3");
        assert_eq!(select_solver(2023, 5, 2).unwrap()(input).unwrap(), "100");
        assert!(select_solver(2023, 5, 2).unwrap()("seeds: 12").is_err());
    }

    #[test]
    fn scratchcard_solvers_are_registered_and_propagate_errors() {
        let input = "Card 1: 1 | 2";
        assert_eq!(select_solver(2023, 4, 1).unwrap()(input).unwrap(), "0");
        assert_eq!(select_solver(2023, 4, 2).unwrap()(input).unwrap(), "1");
        assert!(select_solver(2023, 4, 2).unwrap()("Card 1: 1 | 1").is_err());
    }

    #[test]
    fn a_day_without_part_two_rejects_that_selection() {
        let day = Day {
            part1: aoc2023::d01::part1,
            part2: None,
        };
        assert!(day.solver(1).is_ok());
        assert_eq!(
            day.solver(2).unwrap_err().kind(),
            io::ErrorKind::InvalidInput
        );
        assert!(day.solver(0).is_err());
        assert!(day.solver(3).is_err());
    }
}
