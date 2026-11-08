use svutils::SolveError;

const MAX_INPUT_BYTES: usize = 1_000_000;

fn lines(input: &str) -> Result<std::str::Lines<'_>, SolveError> {
    if input.is_empty() {
        return Err(SolveError::EmptyInput);
    }
    if input.len() > MAX_INPUT_BYTES {
        return Err(SolveError::ResourceLimit(
            "keypad input size limit exceeded",
        ));
    }
    Ok(input.lines())
}

#[derive(Clone, Copy, Debug)]
enum Instruction {
    Up,
    Right,
    Down,
    Left,
}

impl Instruction {
    fn destination(self, position: Position) -> Position {
        let (dx, dy) = match self {
            Self::Up => (0, -1),
            Self::Right => (1, 0),
            Self::Down => (0, 1),
            Self::Left => (-1, 0),
        };
        Position {
            x: position.x + dx,
            y: position.y + dy,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Position {
    x: i8,
    y: i8,
}

struct Keypad {
    rows: &'static [&'static [u8]],
    start: Position,
}

const SQUARE: Keypad = Keypad {
    rows: &[b"123", b"456", b"789"],
    start: Position { x: 1, y: 1 },
};

const DIAMOND: Keypad = Keypad {
    rows: &[b"  1  ", b" 234 ", b"56789", b" ABC ", b"  D  "],
    start: Position { x: 0, y: 2 },
};

impl Keypad {
    fn follow_line(&self, mut position: Position, line: &str) -> Result<Position, SolveError> {
        if line.is_empty() {
            return Err(SolveError::InvalidInput("blank instruction line".into()));
        }
        for (column, direction) in line.chars().enumerate() {
            let instruction = match direction {
                'U' => Instruction::Up,
                'R' => Instruction::Right,
                'D' => Instruction::Down,
                'L' => Instruction::Left,
                _ => {
                    return Err(SolveError::InvalidInput(format!(
                        "column {}: expected U, D, L or R",
                        column + 1
                    )))
                }
            };
            position = self.advance(position, instruction);
        }
        Ok(position)
    }

    fn button(&self, position: Position) -> Option<u8> {
        let row = usize::try_from(position.y).ok()?;
        let column = usize::try_from(position.x).ok()?;
        self.rows
            .get(row)?
            .get(column)
            .copied()
            .filter(|&button| button != b' ')
    }

    fn advance(&self, position: Position, instruction: Instruction) -> Position {
        let candidate = instruction.destination(position);
        if self.button(candidate).is_some() {
            candidate
        } else {
            position
        }
    }
}

pub fn part1(input: &str) -> Result<String, SolveError> {
    let mut position = SQUARE.start;
    let mut code = String::new();
    for (index, line) in lines(input)?.enumerate() {
        position = SQUARE
            .follow_line(position, line)
            .map_err(|error| error.at_line(index + 1))?;
        code.push(char::from(SQUARE.button(position).unwrap()));
    }
    Ok(code)
}

pub fn part2(input: &str) -> Result<String, SolveError> {
    let mut position = DIAMOND.start;
    let mut code = String::new();
    for (index, line) in lines(input)?.enumerate() {
        position = DIAMOND
            .follow_line(position, line)
            .map_err(|error| error.at_line(index + 1))?;
        code.push(char::from(DIAMOND.button(position).unwrap()));
    }
    Ok(code)
}

#[cfg(test)]
mod tests {
    use super::*;
    use svutils::scaffold_test_result;

    const YEAR: i16 = 2016;
    const DAY: i16 = 2;

    #[test]
    fn every_button_move_matches_independent_key_coordinates() {
        let square = (0..9)
            .map(|index| {
                (
                    b'1' + index,
                    Position {
                        x: (index % 3) as i8,
                        y: (index / 3) as i8,
                    },
                )
            })
            .collect::<Vec<_>>();
        let diamond = b"123456789ABCD"
            .iter()
            .enumerate()
            .map(|(index, &button)| {
                let (x, y) = match index {
                    0 => (2, 0),
                    1..=3 => (index as i8, 1),
                    4..=8 => ((index - 4) as i8, 2),
                    9..=11 => ((index - 8) as i8, 3),
                    _ => (2, 4),
                };
                (button, Position { x, y })
            })
            .collect::<Vec<_>>();
        for (keypad, buttons) in [(&SQUARE, square), (&DIAMOND, diamond)] {
            assert_eq!(keypad.button(keypad.start), Some(b'5'));
            for &(button, position) in &buttons {
                assert_eq!(keypad.button(position), Some(button));
                for (instruction, dx, dy) in [
                    (Instruction::Up, 0, -1),
                    (Instruction::Right, 1, 0),
                    (Instruction::Down, 0, 1),
                    (Instruction::Left, -1, 0),
                ] {
                    let expected = buttons
                        .iter()
                        .find(|(_, candidate)| {
                            candidate.x == position.x + dx && candidate.y == position.y + dy
                        })
                        .map_or(position, |&(_, candidate)| candidate);
                    assert_eq!(keypad.advance(position, instruction), expected);
                }
            }
        }
    }

    #[test]
    fn parts_carry_positions_and_build_codes_as_text() {
        assert_eq!(part1("RR\nDD\nLL\nUU").unwrap(), "6971");
        assert_eq!(part2("RR\nDD\nLL\nUU").unwrap(), "7DD7");
        assert_eq!(part1(&"R\n".repeat(40)).unwrap(), "6".repeat(40));
    }

    #[test]
    fn blank_lines_and_invalid_directions_are_syntax_errors() {
        for solve in [part1, part2] {
            assert!(matches!(solve(""), Err(SolveError::EmptyInput)));
            for input in ["R\n\nD", "R\n\n", "R\n \nD"] {
                assert!(matches!(
                    solve(input),
                    Err(SolveError::AtLine { line: 2, .. })
                ));
            }
            assert!(matches!(
                solve("\nR"),
                Err(SolveError::AtLine { line: 1, .. })
            ));
            for bad in ['X', 'r', '7', ' ', '\t', 'é', 'Ｕ'] {
                let error = solve(&format!("R\nUL{bad}D")).unwrap_err();
                assert!(matches!(error, SolveError::AtLine { line: 2, .. }));
                assert!(error.to_string().contains("column 3:"));
            }
            assert_eq!(solve("RR\r\nDD\r\n").unwrap(), solve("RR\nDD").unwrap());
        }
    }

    #[test]
    fn input_size_bounds_work_and_code_storage() {
        let oversized = "R".repeat(MAX_INPUT_BYTES + 1);
        for solve in [part1, part2] {
            assert!(matches!(
                solve(&oversized),
                Err(SolveError::ResourceLimit(_))
            ));
            assert!(matches!(
                solve("R\nD\nL\nU\nX"),
                Err(SolveError::AtLine { line: 5, .. })
            ));
        }
        assert!(part1(&"R".repeat(MAX_INPUT_BYTES)).is_ok());
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
