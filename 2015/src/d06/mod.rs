use svutils::SolveError;

const SIDE: usize = 1000;

struct Instruction {
    action: Action,
    start: (usize, usize),
    end: (usize, usize),
}

#[derive(Clone, Copy)]
enum Action {
    TurnOn,
    TurnOff,
    Toggle,
}

fn coordinate(input: &str) -> Result<(usize, usize), SolveError> {
    let (x, y) = input
        .split_once(',')
        .ok_or_else(|| SolveError::InvalidInput("expected x,y coordinates".into()))?;
    let component = |value: &str| -> Result<usize, SolveError> {
        if value.is_empty() || !value.bytes().all(|byte| byte.is_ascii_digit()) {
            return Err(SolveError::InvalidInput(
                "coordinates must be nonnegative decimal integers".into(),
            ));
        }
        let value = value.parse()?;
        if value >= SIDE {
            return Err(SolveError::InvalidInput(
                "coordinates must be between 0 and 999".into(),
            ));
        }
        Ok(value)
    };
    Ok((component(x)?, component(y)?))
}

impl Instruction {
    fn parse(line: &str) -> Result<Self, SolveError> {
        let (action, rest) = if let Some(rest) = line.strip_prefix("turn on ") {
            (Action::TurnOn, rest)
        } else if let Some(rest) = line.strip_prefix("turn off ") {
            (Action::TurnOff, rest)
        } else if let Some(rest) = line.strip_prefix("toggle ") {
            (Action::Toggle, rest)
        } else {
            return Err(SolveError::InvalidInput(
                "expected turn on, turn off or toggle".into(),
            ));
        };
        let (start, end) = rest.split_once(" through ").ok_or_else(|| {
            SolveError::InvalidInput("expected two coordinates separated by ' through '".into())
        })?;
        let start = coordinate(start)?;
        let end = coordinate(end)?;
        if start.0 > end.0 || start.1 > end.1 {
            return Err(SolveError::InvalidInput(
                "rectangle start must not exceed its end".into(),
            ));
        }
        Ok(Self { action, start, end })
    }

    fn rows(&self) -> impl Iterator<Item = std::ops::RangeInclusive<usize>> + '_ {
        (self.start.1..=self.end.1).map(|y| y * SIDE + self.start.0..=y * SIDE + self.end.0)
    }
}

fn instructions(input: &str) -> Result<Vec<Instruction>, SolveError> {
    if input.trim().is_empty() {
        return Err(SolveError::EmptyInput);
    }
    input
        .lines()
        .enumerate()
        .map(|(index, line)| Instruction::parse(line).map_err(|error| error.at_line(index + 1)))
        .collect()
}

fn brightness(value: u64, action: Action) -> Result<u64, SolveError> {
    match action {
        Action::TurnOn => value.checked_add(1).ok_or(SolveError::ArithmeticOverflow),
        Action::TurnOff => Ok(value.saturating_sub(1)),
        Action::Toggle => value.checked_add(2).ok_or(SolveError::ArithmeticOverflow),
    }
}

fn total_brightness(grid: &[u64]) -> Result<u64, SolveError> {
    grid.iter().try_fold(0u64, |sum, &value| {
        sum.checked_add(value).ok_or(SolveError::ArithmeticOverflow)
    })
}

pub fn part1(input: &str) -> Result<String, SolveError> {
    let instructions = instructions(input)?;
    let mut grid = vec![false; SIDE * SIDE];
    for instruction in instructions {
        for row in instruction.rows() {
            for cell in &mut grid[row] {
                *cell = match instruction.action {
                    Action::TurnOn => true,
                    Action::TurnOff => false,
                    Action::Toggle => !*cell,
                };
            }
        }
    }
    Ok(grid.into_iter().filter(|&on| on).count().to_string())
}

pub fn part2(input: &str) -> Result<String, SolveError> {
    let instructions = instructions(input)?;
    let mut grid = vec![0u64; SIDE * SIDE];
    for instruction in instructions {
        for row in instruction.rows() {
            for cell in &mut grid[row] {
                *cell = brightness(*cell, instruction.action)?;
            }
        }
    }
    total_brightness(&grid).map(|total| total.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use svutils::scaffold_test_result;

    #[test]
    fn generated_rectangles_match_independent_cell_simulation() {
        let mut seed = 6u64;
        for _ in 0..12 {
            let mut on = [[false; 4]; 4];
            let mut levels = [[0u64; 4]; 4];
            let mut input = String::new();
            for _ in 0..40 {
                let mut next = || {
                    seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
                    (seed >> 32) as usize
                };
                let action = next() % 3;
                let (a, b, c, d) = (next() % 4, next() % 4, next() % 4, next() % 4);
                let (x1, x2, y1, y2) = (a.min(b), a.max(b), c.min(d), c.max(d));
                let name = ["turn on", "turn off", "toggle"][action];
                input.push_str(&format!("{name} {x1},{y1} through {x2},{y2}\n"));
                for x in 0..4 {
                    for y in 0..4 {
                        if x >= x1 && x <= x2 && y >= y1 && y <= y2 {
                            match action {
                                0 => {
                                    on[x][y] = true;
                                    levels[x][y] += 1;
                                }
                                1 => {
                                    on[x][y] = false;
                                    if levels[x][y] > 0 {
                                        levels[x][y] -= 1;
                                    }
                                }
                                _ => {
                                    on[x][y] = !on[x][y];
                                    levels[x][y] += 2;
                                }
                            }
                        }
                    }
                }
            }
            let expected_on = on.iter().flatten().filter(|&&cell| cell).count();
            let expected_brightness: u64 = levels.iter().flatten().sum();
            assert_eq!(part1(&input).unwrap(), expected_on.to_string());
            assert_eq!(part2(&input).unwrap(), expected_brightness.to_string());
        }
    }

    #[test]
    fn inclusive_edges_floor_and_brightness_beyond_i8() {
        let input = "turn off 0,0 through 999,999\r\ntoggle 998,998 through 999,999\r\nturn on 999,999 through 999,999\r\n";
        assert_eq!(part1(input).unwrap(), "4");
        assert_eq!(part2(input).unwrap(), "9");
        let input = "toggle 999,999 through 999,999\n".repeat(65);
        assert_eq!(part1(&input).unwrap(), "1");
        assert_eq!(part2(&input).unwrap(), "130");
        assert_eq!(part2("turn off 999,999 through 999,999").unwrap(), "0");
    }

    #[test]
    fn malformed_instructions_have_line_context() {
        for solver in [part1, part2] {
            for input in ["", " \r\n"] {
                assert!(matches!(solver(input), Err(SolveError::EmptyInput)));
            }
            for bad in [
                "",
                "on 0,0 through 1,1",
                "toggle 0,0 to 1,1",
                "toggle 0,0 through 1000,0",
                "toggle 0,0 through 0,1000",
                "toggle 1,0 through 0,0",
                "toggle 0,1 through 0,0",
                "toggle -1,0 through 0,0",
                "toggle 0,+1 through 0,1",
                "toggle 0,0 through 1,1 extra",
                "toggle 0,0,0 through 1,1",
                "toggle ０,0 through 1,1",
                "toggle 99999999999999999999999,0 through 1,1",
            ] {
                let input = format!("turn off 0,0 through 0,0\n{bad}\n");
                assert!(
                    solver(&input)
                        .unwrap_err()
                        .to_string()
                        .starts_with("line 2:"),
                    "{bad:?}"
                );
            }
        }
    }

    #[test]
    fn brightness_arithmetic_is_checked() {
        assert!(brightness(u64::MAX, Action::TurnOn).is_err());
        assert!(brightness(u64::MAX - 1, Action::Toggle).is_err());
        assert_eq!(brightness(0, Action::TurnOff).unwrap(), 0);
        assert_eq!(brightness(u64::MAX - 2, Action::Toggle).unwrap(), u64::MAX);
        assert!(total_brightness(&[u64::MAX, 1]).is_err());
        assert_eq!(total_brightness(&[u64::MAX - 1, 1]).unwrap(), u64::MAX);
    }

    #[test]
    fn test_part1_input() {
        scaffold_test_result(2015, 6, "input.txt", "input-spec.1.txt", part1);
    }

    #[test]
    fn test_part2_input() {
        scaffold_test_result(2015, 6, "input.txt", "input-spec.2.txt", part2);
    }
}
