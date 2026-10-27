use svutils::SolveError;

#[derive(Clone, Copy)]
enum Direction {
    Up,
    Right,
    Down,
    Left,
}

#[derive(Clone, Copy)]
struct Instruction {
    direction: Direction,
    length: i64,
}

struct DigRecord<'a> {
    literal: Instruction,
    colour: &'a str,
}

fn parse_record(line: &str) -> Result<DigRecord<'_>, SolveError> {
    let mut fields = line.split_whitespace();
    let direction = match fields.next() {
        Some("U") => Direction::Up,
        Some("R") => Direction::Right,
        Some("D") => Direction::Down,
        Some("L") => Direction::Left,
        _ => {
            return Err(SolveError::InvalidInput(
                "expected digging direction U, R, D or L".into(),
            ))
        }
    };
    let length = fields
        .next()
        .ok_or_else(|| SolveError::InvalidInput("missing digging distance".into()))?;
    if !length.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err(SolveError::InvalidInput(
            "distance must be a nonnegative decimal integer".into(),
        ));
    }
    let length = length.parse()?;
    let colour = fields
        .next()
        .and_then(|field| field.strip_prefix("(#"))
        .and_then(|field| field.strip_suffix(')'))
        .ok_or_else(|| SolveError::InvalidInput("expected colour in '(#hhhhhh)' form".into()))?;
    if colour.len() != 6 || !colour.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(SolveError::InvalidInput(
            "colour must contain exactly six hexadecimal digits".into(),
        ));
    }
    if fields.next().is_some() {
        return Err(SolveError::InvalidInput(
            "unexpected field after colour".into(),
        ));
    }
    Ok(DigRecord {
        literal: Instruction { direction, length },
        colour,
    })
}

fn parse_records(input: &str) -> Result<Vec<DigRecord<'_>>, SolveError> {
    if input.trim().is_empty() {
        return Err(SolveError::EmptyInput);
    }
    input
        .lines()
        .enumerate()
        .map(|(index, line)| parse_record(line).map_err(|error| error.at_line(index + 1)))
        .collect()
}

impl DigRecord<'_> {
    fn decode_colour(&self) -> Result<Instruction, SolveError> {
        let colour = self.colour;
        // The parser establishes six ASCII hex digits, so these byte boundaries are valid.
        let direction = match colour.as_bytes()[5] {
            b'0' => Direction::Right,
            b'1' => Direction::Down,
            b'2' => Direction::Left,
            b'3' => Direction::Up,
            _ => {
                return Err(SolveError::InvalidInput(
                    "encoded direction must be 0, 1, 2 or 3".into(),
                ))
            }
        };
        Ok(Instruction {
            direction,
            length: i64::from_str_radix(&colour[..5], 16)?,
        })
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
struct Point {
    x: i128,
    y: i128,
}

// Axis-aligned segment stored as inclusive coordinate bounds.
struct Segment {
    min: Point,
    max: Point,
}

impl Segment {
    fn between(start: Point, end: Point) -> Self {
        Self {
            min: Point {
                x: start.x.min(end.x),
                y: start.y.min(end.y),
            },
            max: Point {
                x: start.x.max(end.x),
                y: start.y.max(end.y),
            },
        }
    }

    fn intersection(&self, other: &Self) -> Option<Self> {
        let min = Point {
            x: self.min.x.max(other.min.x),
            y: self.min.y.max(other.min.y),
        };
        let max = Point {
            x: self.max.x.min(other.max.x),
            y: self.max.y.min(other.max.y),
        };
        (min.x <= max.x && min.y <= max.y).then_some(Self { min, max })
    }
}

fn lagoon_size(instructions: &[Instruction]) -> Result<i64, SolveError> {
    let origin = Point { x: 0, y: 0 };
    let mut position = origin;
    let mut segments = Vec::with_capacity(instructions.len());
    let (mut perimeter, mut area) = (0_i128, 0_i128);
    for &Instruction { direction, length } in instructions {
        if length == 0 {
            continue;
        }
        let length = i128::from(length);
        let mut next = position;
        match direction {
            Direction::Up => {
                next.y = next
                    .y
                    .checked_sub(length)
                    .ok_or(SolveError::ArithmeticOverflow)?
            }
            Direction::Right => {
                next.x = next
                    .x
                    .checked_add(length)
                    .ok_or(SolveError::ArithmeticOverflow)?
            }
            Direction::Down => {
                next.y = next
                    .y
                    .checked_add(length)
                    .ok_or(SolveError::ArithmeticOverflow)?
            }
            Direction::Left => {
                next.x = next
                    .x
                    .checked_sub(length)
                    .ok_or(SolveError::ArithmeticOverflow)?
            }
        }
        // Signed area from vertical edges; horizontal edges contribute zero.
        let dy = next
            .y
            .checked_sub(position.y)
            .ok_or(SolveError::ArithmeticOverflow)?;
        let contribution = position
            .x
            .checked_mul(dy)
            .ok_or(SolveError::ArithmeticOverflow)?;
        area = area
            .checked_add(contribution)
            .ok_or(SolveError::ArithmeticOverflow)?;
        perimeter = perimeter
            .checked_add(length)
            .ok_or(SolveError::ArithmeticOverflow)?;
        segments.push(Segment::between(position, next));
        position = next;
    }
    if position != origin {
        return Err(SolveError::InvalidInput(
            "trench must return to its starting point".into(),
        ));
    }
    if area == 0 {
        return Err(SolveError::InvalidInput(
            "trench must enclose a nonzero area".into(),
        ));
    }
    for (i, segment) in segments.iter().enumerate() {
        for (j, other) in segments.iter().enumerate().skip(i + 1) {
            if let Some(intersection) = segment.intersection(other) {
                let adjacent = j == i + 1 || (i == 0 && j == segments.len() - 1);
                // Consecutive edges share one endpoint, never a stretch of trench.
                if !adjacent || intersection.min != intersection.max {
                    return Err(SolveError::InvalidInput("trench edges must not cross, overlap or touch away from adjacent endpoints".into()));
                }
            }
        }
    }
    // Pick's theorem: interior plus boundary = area + boundary / 2 + 1.
    let size = area
        .checked_abs()
        .and_then(|area| area.checked_add(perimeter / 2))
        .and_then(|size| size.checked_add(1))
        .ok_or(SolveError::ArithmeticOverflow)?;
    i64::try_from(size).map_err(|_| SolveError::ArithmeticOverflow)
}

pub fn part1(input: &str) -> Result<String, SolveError> {
    let records = parse_records(input)?;
    let plan: Vec<_> = records.into_iter().map(|record| record.literal).collect();
    Ok(lagoon_size(&plan)?.to_string())
}

pub fn part2(input: &str) -> Result<String, SolveError> {
    let records = parse_records(input)?;
    let plan = records
        .iter()
        .enumerate()
        .map(|(index, record)| {
            record
                .decode_colour()
                .map_err(|error| error.at_line(index + 1))
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(lagoon_size(&plan)?.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use svutils::scaffold_test_result;

    const YEAR: i16 = 2023;
    const DAY: i16 = 18;

    #[test]
    fn test_part1_input() {
        scaffold_test_result(YEAR, DAY, "input.txt", "input-spec.1.txt", part1);
    }

    #[test]
    fn test_part2_input() {
        scaffold_test_result(YEAR, DAY, "input.txt", "input-spec.2.txt", part2);
    }

    #[test]
    fn lagoon_size_does_not_depend_on_walking_orientation() {
        let clockwise: Vec<_> = [
            (Direction::Right, 2),
            (Direction::Down, 1),
            (Direction::Left, 2),
            (Direction::Up, 1),
        ]
        .into_iter()
        .map(|(direction, length)| Instruction { direction, length })
        .collect();
        let counterclockwise: Vec<_> = [
            (Direction::Down, 1),
            (Direction::Right, 2),
            (Direction::Up, 1),
            (Direction::Left, 2),
        ]
        .into_iter()
        .map(|(direction, length)| Instruction { direction, length })
        .collect();
        assert_eq!(lagoon_size(&clockwise).unwrap(), 6);
        assert_eq!(lagoon_size(&counterclockwise).unwrap(), 6);
    }

    #[test]
    fn malformed_lines_are_rejected_instead_of_silently_skipped() {
        for input in ["", " \t\r\n"] {
            assert!(matches!(part1(input), Err(SolveError::EmptyInput)));
            assert!(matches!(part2(input), Err(SolveError::EmptyInput)));
        }
        for input in [
            "nonsense",
            "X 1 (#000010)",
            "R -1 (#000010)",
            "R +1 (#000010)",
            "R nope (#000010)",
            "R 1",
            "R 1 #000010",
            "R 1 (#00001)",
            "R 1 (#00001g)",
            "R 1 (#é0000)",
            "R 1 (#000010) extra",
            "R 1 (#000010)\n\nL 1 (#000012)",
        ] {
            assert!(part1(input).is_err(), "{input:?}");
            assert!(part2(input).is_err(), "{input:?}");
        }
    }

    #[test]
    fn each_part_uses_its_own_decoding_and_ignores_zero_moves() {
        let input = "R 2 (#000030)\nD 1 (#000021)\nL 2 (#000032)\nU 1 (#000023)\nU 0 (#000000)";
        assert_eq!(part1(input).unwrap(), "6");
        assert_eq!(part2(input).unwrap(), "12");
        let crlf = input.replace('\n', "\r\n") + "\r\n";
        assert_eq!(part1(&crlf).unwrap(), "6");
        assert_eq!(part2(&crlf).unwrap(), "12");
    }

    #[test]
    fn a_valid_rgb_colour_need_not_encode_a_part_two_direction() {
        let input = "R 2 (#ffffff)\nD 1 (#000011)\nL 2 (#000022)\nU 1 (#000013)";
        assert_eq!(part1(input).unwrap(), "6");
        assert!(matches!(
            part2(input),
            Err(SolveError::AtLine { line: 1, .. })
        ));
    }

    fn size(moves: &[(Direction, i64)]) -> Result<i64, SolveError> {
        let plan: Vec<_> = moves
            .iter()
            .map(|&(direction, length)| Instruction { direction, length })
            .collect();
        lagoon_size(&plan)
    }

    #[test]
    fn legal_concave_and_collinear_boundaries_are_accepted() {
        use Direction::*;
        assert_eq!(
            size(&[(Right, 1), (Right, 1), (Down, 1), (Left, 2), (Up, 1)]).unwrap(),
            6
        );
        assert_eq!(
            size(&[
                (Right, 3),
                (Down, 1),
                (Left, 1),
                (Down, 1),
                (Left, 2),
                (Up, 2)
            ])
            .unwrap(),
            11
        );
        assert_eq!(
            size(&[(Right, 1), (Down, 1), (Left, 2), (Up, 1), (Right, 1)]).unwrap(),
            6
        );
    }

    #[test]
    fn invalid_geometry_is_rejected() {
        use Direction::*;
        for moves in [
            vec![],
            vec![(Up, 0)],
            vec![(Right, 2)],
            vec![(Right, 2), (Left, 2)],
            // Retracing, crossing, and touching at a nonadjacent endpoint.
            vec![(Right, 3), (Left, 1), (Down, 1), (Left, 2), (Up, 1)],
            vec![
                (Right, 4),
                (Down, 4),
                (Left, 2),
                (Up, 6),
                (Left, 2),
                (Down, 2),
            ],
            vec![
                (Right, 2),
                (Down, 2),
                (Left, 2),
                (Up, 2),
                (Left, 1),
                (Up, 1),
                (Right, 1),
                (Down, 1),
            ],
        ] {
            assert!(matches!(size(&moves), Err(SolveError::InvalidInput(_))));
        }
    }

    #[test]
    fn unrepresentable_sizes_return_overflow_errors() {
        use Direction::*;
        assert!(matches!(
            size(&[(Right, i64::MAX), (Down, 1), (Left, i64::MAX), (Up, 1)]),
            Err(SolveError::ArithmeticOverflow)
        ));
        // Coordinates fit i128, but the area multiplication does not.
        let mut moves = vec![(Right, i64::MAX); 3];
        moves.extend(vec![(Down, i64::MAX); 3]);
        moves.extend(vec![(Left, i64::MAX); 3]);
        moves.extend(vec![(Up, i64::MAX); 3]);
        assert!(matches!(size(&moves), Err(SolveError::ArithmeticOverflow)));
    }
}
