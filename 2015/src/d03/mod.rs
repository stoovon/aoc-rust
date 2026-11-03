use std::collections::HashSet;
use svutils::SolveError;

type Position = (i64, i64);

fn directions(input: &str) -> Result<&[u8], SolveError> {
    let input = input.trim();
    if input.is_empty() {
        return Err(SolveError::EmptyInput);
    }
    for (index, byte) in input.bytes().enumerate() {
        if !matches!(byte, b'^' | b'>' | b'v' | b'<') {
            return Err(
                SolveError::InvalidInput("expected '^', '>', 'v' or '<'".into()).at_step(index + 1),
            );
        }
    }
    Ok(input.as_bytes())
}

fn step((x, y): Position, direction: u8) -> Result<Position, SolveError> {
    let position = match direction {
        b'^' => y.checked_add(1).map(|y| (x, y)),
        b'>' => x.checked_add(1).map(|x| (x, y)),
        b'v' => y.checked_sub(1).map(|y| (x, y)),
        b'<' => x.checked_sub(1).map(|x| (x, y)),
        _ => return Err(SolveError::InvalidInput("unknown direction".into())),
    };
    position.ok_or(SolveError::ArithmeticOverflow)
}

pub fn part1(input: &str) -> Result<String, SolveError> {
    let mut position = (0, 0);
    let mut houses = HashSet::from([position]);
    for &direction in directions(input)? {
        position = step(position, direction)?;
        houses.insert(position);
    }
    Ok(houses.len().to_string())
}

pub fn part2(input: &str) -> Result<String, SolveError> {
    let mut positions = [(0, 0); 2];
    let mut houses = HashSet::from([(0, 0)]);
    for (index, &direction) in directions(input)?.iter().enumerate() {
        let active = &mut positions[index % 2];
        *active = step(*active, direction)?;
        houses.insert(*active);
    }
    Ok(houses.len().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use svutils::scaffold_test_result;

    fn reference(route: &str, couriers: usize) -> usize {
        let mut visited = vec![(0i64, 0i64)];
        for courier in 0..couriers {
            let (mut x, mut y) = (0, 0);
            for direction in route.chars().skip(courier).step_by(couriers) {
                x += i64::from(direction == '>') - i64::from(direction == '<');
                y += i64::from(direction == '^') - i64::from(direction == 'v');
                visited.push((x, y));
            }
        }
        visited.sort_unstable();
        visited.dedup();
        visited.len()
    }

    #[test]
    fn generated_routes_match_separate_courier_walks() {
        let alphabet = [b'^', b'>', b'v', b'<'];
        for length in 1..=7 {
            for mut encoded in 0..4usize.pow(length) {
                let route: String = (0..length)
                    .map(|_| {
                        let direction = alphabet[encoded % 4];
                        encoded /= 4;
                        char::from(direction)
                    })
                    .collect();
                assert_eq!(part1(&route).unwrap(), reference(&route, 1).to_string());
                assert_eq!(part2(&route).unwrap(), reference(&route, 2).to_string());
            }
        }
    }

    #[test]
    fn validation_and_outer_whitespace() {
        let route = ">>v<<^^";
        for solver in [part1, part2] {
            assert_eq!(
                solver(&format!(" \t\r\n{route}\r\n")).unwrap(),
                solver(route).unwrap()
            );
            for input in ["", " \r\n"] {
                assert!(matches!(solver(input), Err(SolveError::EmptyInput)));
            }
            for input in [">x", "> >", ">\n<", ">é"] {
                assert!(solver(input)
                    .unwrap_err()
                    .to_string()
                    .starts_with("step 2:"));
            }
        }
    }

    #[test]
    fn movement_checks_each_coordinate_boundary() {
        for (position, direction) in [
            ((i64::MAX, 0), b'>'),
            ((i64::MIN, 0), b'<'),
            ((0, i64::MAX), b'^'),
            ((0, i64::MIN), b'v'),
        ] {
            assert!(matches!(
                step(position, direction),
                Err(SolveError::ArithmeticOverflow)
            ));
        }
        assert_eq!(
            step((i64::MAX, i64::MIN), b'<').unwrap(),
            (i64::MAX - 1, i64::MIN)
        );
        assert_eq!(
            step((i64::MAX, i64::MIN), b'^').unwrap(),
            (i64::MAX, i64::MIN + 1)
        );
        assert!(step((0, 0), b'?').is_err());
    }

    #[test]
    fn test_part1_input() {
        scaffold_test_result(2015, 3, "input.txt", "input-spec.1.txt", part1);
    }

    #[test]
    fn test_part2_input() {
        scaffold_test_result(2015, 3, "input.txt", "input-spec.2.txt", part2);
    }
}
