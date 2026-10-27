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
        (2023, 7) => Day {
            part1: aoc2023::d07::part1,
            part2: Some(aoc2023::d07::part2),
        },
        (2023, 8) => Day {
            part1: aoc2023::d08::part1,
            part2: Some(aoc2023::d08::part2),
        },
        (2023, 9) => Day {
            part1: aoc2023::d09::part1,
            part2: Some(aoc2023::d09::part2),
        },
        (2023, 10) => Day {
            part1: aoc2023::d10::part1,
            part2: Some(aoc2023::d10::part2),
        },
        (2023, 11) => Day {
            part1: aoc2023::d11::part1,
            part2: Some(aoc2023::d11::part2),
        },
        (2023, 12) => Day {
            part1: aoc2023::d12::part1,
            part2: Some(aoc2023::d12::part2),
        },
        (2023, 13) => Day {
            part1: aoc2023::d13::part1,
            part2: Some(aoc2023::d13::part2),
        },
        (2023, 14) => Day {
            part1: aoc2023::d14::part1,
            part2: Some(aoc2023::d14::part2),
        },
        (2023, 15) => Day {
            part1: aoc2023::d15::part1,
            part2: Some(aoc2023::d15::part2),
        },
        (2023, 16) => Day {
            part1: aoc2023::d16::part1,
            part2: Some(aoc2023::d16::part2),
        },
        (2023, 17) => Day {
            part1: aoc2023::d17::part1,
            part2: Some(aoc2023::d17::part2),
        },
        (2023, 18) => Day {
            part1: aoc2023::d18::part1,
            part2: Some(aoc2023::d18::part2),
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
    fn lagoon_solvers_are_registered_and_propagate_errors() {
        let input = "R 2 (#000020)\nD 1 (#000011)\nL 2 (#000022)\nU 1 (#000013)";
        for part in [1, 2] {
            let solver = select_solver(2023, 18, part).unwrap();
            assert_eq!(solver(input).unwrap(), "6");
            assert!(solver("nonsense").is_err());
            assert!(solver("R 2 (#000020)").is_err());
        }
    }

    #[test]
    fn crucible_solvers_are_registered_and_propagate_errors() {
        let input = "11111\n11111\n11111\n11111\n11111";
        assert_eq!(select_solver(2023, 17, 1).unwrap()(input).unwrap(), "8");
        assert_eq!(select_solver(2023, 17, 2).unwrap()(input).unwrap(), "8");
        for part in [1, 2] {
            assert!(select_solver(2023, 17, part).unwrap()("x").is_err());
        }
        assert!(select_solver(2023, 17, 2).unwrap()("11111\n11111").is_err());
    }

    #[test]
    fn beam_solvers_are_registered_and_propagate_errors() {
        let input = "..\n..\n..";
        assert_eq!(select_solver(2023, 16, 1).unwrap()(input).unwrap(), "2");
        assert_eq!(select_solver(2023, 16, 2).unwrap()(input).unwrap(), "3");
        for part in [1, 2] {
            assert!(select_solver(2023, 16, part).unwrap()("x").is_err());
        }
    }

    #[test]
    fn lens_solvers_are_registered_and_propagate_errors() {
        assert_eq!(select_solver(2023, 15, 1).unwrap()("a=1").unwrap(), "175");
        assert_eq!(select_solver(2023, 15, 2).unwrap()("a=1").unwrap(), "114");
        for part in [1, 2] {
            assert!(select_solver(2023, 15, part).unwrap()("a=0").is_err());
        }
    }

    #[test]
    fn platform_solvers_are_registered_and_propagate_errors() {
        assert_eq!(select_solver(2023, 14, 1).unwrap()("O\n.").unwrap(), "2");
        assert_eq!(select_solver(2023, 14, 2).unwrap()("O\n.").unwrap(), "1");
        for part in [1, 2] {
            assert!(select_solver(2023, 14, part).unwrap()(".").is_err());
        }
    }

    #[test]
    fn mirror_solvers_are_registered_and_propagate_errors() {
        assert_eq!(select_solver(2023, 13, 1).unwrap()("##").unwrap(), "1");
        assert_eq!(select_solver(2023, 13, 2).unwrap()("#.").unwrap(), "1");
        for part in [1, 2] {
            assert!(select_solver(2023, 13, part).unwrap()("#").is_err());
        }
    }

    #[test]
    fn spring_solvers_are_registered_and_propagate_errors() {
        assert_eq!(select_solver(2023, 12, 1).unwrap()("?? 1").unwrap(), "2");
        assert_eq!(select_solver(2023, 12, 2).unwrap()("?? 1").unwrap(), "252");
        for part in [1, 2] {
            assert!(select_solver(2023, 12, part).unwrap()("# 2").is_err());
        }
    }

    #[test]
    fn galaxy_solvers_are_registered_and_propagate_errors() {
        let input = "#.\n..\n.#";
        assert_eq!(select_solver(2023, 11, 1).unwrap()(input).unwrap(), "4");
        assert_eq!(
            select_solver(2023, 11, 2).unwrap()(input).unwrap(),
            "1000002"
        );
        for part in [1, 2] {
            assert!(select_solver(2023, 11, part).unwrap()("#").is_err());
        }
    }

    #[test]
    fn pipe_solvers_are_registered_and_propagate_errors() {
        let input = "S--7\n|..|\nL--J";
        assert_eq!(select_solver(2023, 10, 1).unwrap()(input).unwrap(), "5");
        assert_eq!(select_solver(2023, 10, 2).unwrap()(input).unwrap(), "2");
        for part in [1, 2] {
            assert!(select_solver(2023, 10, part).unwrap()("S--7\n|..-\nL--J").is_err());
        }
    }

    #[test]
    fn history_solvers_are_registered_and_propagate_errors() {
        let input = "1 4 9 16";
        assert_eq!(select_solver(2023, 9, 1).unwrap()(input).unwrap(), "25");
        assert_eq!(select_solver(2023, 9, 2).unwrap()(input).unwrap(), "0");
        for part in [1, 2] {
            assert!(select_solver(2023, 9, part).unwrap()("2 nope 8").is_err());
        }
    }

    #[test]
    fn network_solvers_are_registered_and_propagate_errors() {
        let input = "LR\n\nAAA = (AAA, ZZZ)\nZZZ = (ZZZ, ZZZ)";
        assert_eq!(select_solver(2023, 8, 1).unwrap()(input).unwrap(), "2");
        assert_eq!(select_solver(2023, 8, 2).unwrap()(input).unwrap(), "2");
        assert!(select_solver(2023, 8, 1).unwrap()("L\n\nAAA = (AAA, AAA)").is_err());
        assert!(select_solver(2023, 8, 2).unwrap()("L\n\nAAA = (AAA, AAA)").is_err());
    }

    #[test]
    fn camel_card_solvers_are_registered_and_propagate_errors() {
        let input = "JJJJ2 10\nQQQQ2 1";
        assert_eq!(select_solver(2023, 7, 1).unwrap()(input).unwrap(), "12");
        assert_eq!(select_solver(2023, 7, 2).unwrap()(input).unwrap(), "21");
        assert!(select_solver(2023, 7, 1).unwrap()("AAAAA 1\nAAAAA 2").is_err());
    }

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
