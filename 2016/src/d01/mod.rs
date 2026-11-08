use svutils::SolveError;

const MAX_INSTRUCTIONS: usize = 10_000;
const MAX_INTERSECTION_CHECKS: usize = 10_000_000;

fn parse(input: &str) -> Result<Vec<Instruction>, SolveError> {
    if input.trim().is_empty() {
        return Err(SolveError::EmptyInput);
    }
    input
        .split(',')
        .enumerate()
        .map(|(index, text)| {
            if index == MAX_INSTRUCTIONS {
                return Err(SolveError::ResourceLimit("too many route instructions"));
            }
            let record = || -> Result<Instruction, SolveError> {
                let text = text.trim();
                let (turn, distance) = text.as_bytes().split_first().ok_or_else(|| {
                    SolveError::InvalidInput("expected a turn and distance".into())
                })?;
                if !matches!(turn, b'L' | b'R') {
                    return Err(SolveError::InvalidInput("expected turn L or R".into()));
                }
                if distance.is_empty() || !distance.iter().all(u8::is_ascii_digit) {
                    return Err(SolveError::InvalidInput(
                        "expected a nonnegative decimal distance".into(),
                    ));
                }
                let distance = text[1..].parse::<i64>()?;
                Ok(if *turn == b'L' {
                    Instruction::Left(distance)
                } else {
                    Instruction::Right(distance)
                })
            };
            record().map_err(|error| error.at_step(index + 1))
        })
        .collect()
}

#[derive(Debug)]
enum Instruction {
    Left(i64),
    Right(i64),
}

impl Instruction {
    fn follow(&self, cur_dir: &Direction) -> (Direction, i64) {
        match self {
            Instruction::Left(n) => (cur_dir.turn_left(), *n),
            Instruction::Right(n) => (cur_dir.turn_right(), *n),
        }
    }
}

enum Direction {
    North,
    South,
    East,
    West,
}

impl Direction {
    fn turn_right(&self) -> Direction {
        match self {
            Direction::North => Direction::East,
            Direction::South => Direction::West,
            Direction::East => Direction::South,
            Direction::West => Direction::North,
        }
    }

    fn turn_left(&self) -> Direction {
        match self {
            Direction::North => Direction::West,
            Direction::South => Direction::East,
            Direction::East => Direction::North,
            Direction::West => Direction::South,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Position {
    x: i64,
    y: i64,
}

impl Position {
    fn advance(self, direction: &Direction, distance: i64) -> Result<Self, SolveError> {
        let mut next = self;
        let coordinate = match direction {
            Direction::North => self.y.checked_sub(distance).map(|y| {
                next.y = y;
            }),
            Direction::South => self.y.checked_add(distance).map(|y| {
                next.y = y;
            }),
            Direction::East => self.x.checked_add(distance).map(|x| {
                next.x = x;
            }),
            Direction::West => self.x.checked_sub(distance).map(|x| {
                next.x = x;
            }),
        };
        coordinate.ok_or(SolveError::ArithmeticOverflow)?;
        Ok(next)
    }

    fn distance(self) -> Result<u64, SolveError> {
        self.x
            .unsigned_abs()
            .checked_add(self.y.unsigned_abs())
            .ok_or(SolveError::ArithmeticOverflow)
    }
}

struct Segment {
    start: Position,
    end: Position,
}

impl Segment {
    fn first_intersection(&self, previous: &Self) -> Option<Position> {
        let left = self
            .start
            .x
            .min(self.end.x)
            .max(previous.start.x.min(previous.end.x));
        let right = self
            .start
            .x
            .max(self.end.x)
            .min(previous.start.x.max(previous.end.x));
        let top = self
            .start
            .y
            .min(self.end.y)
            .max(previous.start.y.min(previous.end.y));
        let bottom = self
            .start
            .y
            .max(self.end.y)
            .min(previous.start.y.max(previous.end.y));
        if left > right || top > bottom {
            return None;
        }
        // Exclude the starting point: a revisit requires taking at least one step.
        if self.end.x > self.start.x {
            let x = left.max(self.start.x + 1);
            (x <= right).then_some(Position { x, y: self.start.y })
        } else if self.end.x < self.start.x {
            let x = right.min(self.start.x - 1);
            (x >= left).then_some(Position { x, y: self.start.y })
        } else if self.end.y > self.start.y {
            let y = top.max(self.start.y + 1);
            (y <= bottom).then_some(Position { x: self.start.x, y })
        } else if self.end.y < self.start.y {
            let y = bottom.min(self.start.y - 1);
            (y >= top).then_some(Position { x: self.start.x, y })
        } else {
            None
        }
    }
}

pub fn part1(input: &str) -> Result<String, SolveError> {
    let instructions = parse(input)?;
    let mut position = Position { x: 0, y: 0 };
    let mut direction = Direction::North;
    for (index, instruction) in instructions.iter().enumerate() {
        let (next_direction, distance) = instruction.follow(&direction);
        direction = next_direction;
        position = position
            .advance(&direction, distance)
            .map_err(|error| error.at_step(index + 1))?;
    }
    position.distance().map(|distance| distance.to_string())
}

pub fn part2(input: &str) -> Result<String, SolveError> {
    let instructions = parse(input)?;
    let mut position = Position { x: 0, y: 0 };
    let mut direction = Direction::North;
    let mut visited = vec![Segment {
        start: position,
        end: position,
    }];
    let mut checks = 0;
    for (index, instruction) in instructions.iter().enumerate() {
        let (next_direction, distance) = instruction.follow(&direction);
        direction = next_direction;
        let end = position
            .advance(&direction, distance)
            .map_err(|error| error.at_step(index + 1))?;
        let segment = Segment {
            start: position,
            end,
        };
        let mut first: Option<Position> = None;
        for previous in &visited {
            if checks == MAX_INTERSECTION_CHECKS {
                return Err(SolveError::ResourceLimit(
                    "route intersection budget exhausted",
                ));
            }
            checks += 1;
            if let Some(hit) = segment.first_intersection(previous) {
                let steps = |point: Position| {
                    position
                        .x
                        .abs_diff(point.x)
                        .max(position.y.abs_diff(point.y))
                };
                if first.is_none_or(|current| steps(hit) < steps(current)) {
                    first = Some(hit);
                }
            }
        }
        if let Some(first) = first {
            return first.distance().map(|distance| distance.to_string());
        }
        if distance != 0 {
            visited.push(segment);
        }
        position = end;
    }
    Err(SolveError::NoSolution("route never revisits a location"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use svutils::scaffold_test_result;

    const YEAR: i16 = 2016;
    const DAY: i16 = 1;

    #[test]
    fn parser_accepts_outer_and_record_whitespace() {
        let instructions = parse(" \tL17 ,\r\n R0, L23 \n").unwrap();
        assert!(matches!(
            instructions.as_slice(),
            [
                Instruction::Left(17),
                Instruction::Right(0),
                Instruction::Left(23)
            ]
        ));
        assert_eq!(part1(" \tL17 ,\r\n R0, L23 \n").unwrap(), "40");
        assert!(matches!(
            parse("R9223372036854775807").unwrap().as_slice(),
            [Instruction::Right(i64::MAX)]
        ));
    }

    #[test]
    fn parser_reports_malformed_instruction_numbers_without_panicking() {
        for solve in [part1, part2] {
            for input in ["", " \r\n"] {
                assert!(matches!(solve(input), Err(SolveError::EmptyInput)));
            }
            for bad in [
                "",
                "L",
                "Q7",
                "l7",
                "é7",
                "Ｒ7",
                "R-7",
                "R+7",
                "L1.5",
                "L 7",
                "R７",
                "L7 extra",
                "L9223372036854775808",
            ] {
                let input = format!("L0,{bad},R0");
                assert!(
                    matches!(solve(&input), Err(SolveError::AtStep { step: 2, .. })),
                    "{bad:?}"
                );
            }
            assert!(matches!(
                solve("L0,"),
                Err(SolveError::AtStep { step: 2, .. })
            ));
            assert!(matches!(
                solve(",L0"),
                Err(SolveError::AtStep { step: 1, .. })
            ));
        }
    }

    #[test]
    fn no_revisit_is_distinct_from_returning_to_the_origin() {
        assert!(matches!(part2("L17, R23"), Err(SolveError::NoSolution(_))));
        assert!(matches!(part2("L0, R0"), Err(SolveError::NoSolution(_))));
        assert_eq!(part2("R7, R7, R7, R7").unwrap(), "0");
    }

    fn block_reference(input: &str) -> Option<u64> {
        use std::collections::HashSet;
        let mut visited = HashSet::from([(0i64, 0i64)]);
        let (mut x, mut y, mut facing) = (0i64, 0i64, 0usize);
        for token in input.split(',') {
            let token = token.trim();
            facing = (facing + if token.starts_with('R') { 1 } else { 3 }) % 4;
            let (dx, dy) = [(0, -1), (1, 0), (0, 1), (-1, 0)][facing];
            for _ in 0..token[1..].parse::<usize>().unwrap() {
                x += dx;
                y += dy;
                if !visited.insert((x, y)) {
                    return Some(x.unsigned_abs() + y.unsigned_abs());
                }
            }
        }
        None
    }

    #[test]
    fn segment_routes_match_block_by_block_walks() {
        for length in 1..=6 {
            for mut code in 0..6usize.pow(length) {
                let mut tokens = Vec::new();
                for _ in 0..length {
                    tokens.push(["L0", "L1", "L3", "R0", "R1", "R3"][code % 6]);
                    code /= 6;
                }
                let input = tokens.join(",");
                match (part2(&input), block_reference(&input)) {
                    (Ok(actual), Some(expected)) => {
                        assert_eq!(actual, expected.to_string(), "{input}")
                    }
                    (Err(SolveError::NoSolution(_)), None) => (),
                    (actual, expected) => panic!("{input}: {actual:?} != {expected:?}"),
                }
            }
        }
    }

    #[test]
    fn segment_intersections_match_small_lattice_points() {
        let mut segments = Vec::new();
        for fixed in -2..=2 {
            for start in -2..=2 {
                for end in -2..=2 {
                    segments.push(Segment {
                        start: Position { x: start, y: fixed },
                        end: Position { x: end, y: fixed },
                    });
                    segments.push(Segment {
                        start: Position { x: fixed, y: start },
                        end: Position { x: fixed, y: end },
                    });
                }
            }
        }
        for current in &segments {
            for previous in &segments {
                let mut point = current.start;
                let mut expected = None;
                while point != current.end {
                    point.x += (current.end.x - current.start.x).signum();
                    point.y += (current.end.y - current.start.y).signum();
                    if point.x >= previous.start.x.min(previous.end.x)
                        && point.x <= previous.start.x.max(previous.end.x)
                        && point.y >= previous.start.y.min(previous.end.y)
                        && point.y <= previous.start.y.max(previous.end.y)
                    {
                        expected = Some(point);
                        break;
                    }
                }
                assert_eq!(current.first_intersection(previous), expected);
            }
        }
    }

    #[test]
    fn huge_distances_do_not_require_visiting_each_block() {
        assert_eq!(part2("R1000000000000,R7,R1000000000000,R7").unwrap(), "0");
        assert!(matches!(
            part2("L1000000000000,R1000000000000"),
            Err(SolveError::NoSolution(_))
        ));
    }

    #[test]
    fn movement_overflow_and_instruction_limits_return_errors() {
        for solve in [part1, part2] {
            assert!(matches!(
                solve("R9223372036854775807,L0,R1"),
                Err(SolveError::AtStep { step: 3, .. })
            ));
            assert!(matches!(
                solve(&"L0,".repeat(MAX_INSTRUCTIONS)),
                Err(SolveError::ResourceLimit(_))
            ));
        }
        assert_eq!(
            Position { x: i64::MIN, y: 0 }.distance().unwrap(),
            1u64 << 63
        );
        assert!(matches!(
            Position {
                x: i64::MIN,
                y: i64::MIN
            }
            .distance(),
            Err(SolveError::ArithmeticOverflow)
        ));
    }

    #[test]
    fn intersection_budget_bounds_long_nonintersecting_routes() {
        let input = (0..5_000)
            .map(|index| format!("R{}", index / 2 + 1))
            .collect::<Vec<_>>()
            .join(",");
        assert!(matches!(part2(&input), Err(SolveError::ResourceLimit(_))));
    }

    #[test]
    fn test_fn1_input() {
        scaffold_test_result(YEAR, DAY, "input.txt", "input-spec.1.txt", part1);
    }

    #[test]
    fn test_fn2_input() {
        scaffold_test_result(YEAR, DAY, "input.txt", "input-spec.2.txt", part2);
    }
}
