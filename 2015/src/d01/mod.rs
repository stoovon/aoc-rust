use svutils::SolveError;

fn instructions(input: &str) -> Result<&[u8], SolveError> {
    let input = input.trim();
    if input.is_empty() {
        return Err(SolveError::EmptyInput);
    }
    for (index, byte) in input.bytes().enumerate() {
        if !matches!(byte, b'(' | b')') {
            return Err(SolveError::InvalidInput("expected '(' or ')'".into()).at_step(index + 1));
        }
    }
    Ok(input.as_bytes())
}

fn next_floor(floor: i64, instruction: u8) -> Result<i64, SolveError> {
    floor
        .checked_add(if instruction == b'(' { 1 } else { -1 })
        .ok_or(SolveError::ArithmeticOverflow)
}

pub fn part1(input: &str) -> Result<String, SolveError> {
    instructions(input)?
        .iter()
        .try_fold(0, |floor, &instruction| next_floor(floor, instruction))
        .map(|floor| floor.to_string())
}

pub fn part2(input: &str) -> Result<String, SolveError> {
    let mut floor = 0;
    for (index, &instruction) in instructions(input)?.iter().enumerate() {
        floor = next_floor(floor, instruction)?;
        if floor == -1 {
            return Ok((index + 1).to_string());
        }
    }
    Err(SolveError::InvalidInput(
        "instructions never enter the basement".into(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use svutils::scaffold_test_result;

    #[test]
    fn independent_generated_walks() {
        for length in 1..=10 {
            for bits in 0usize..1 << length {
                let input: String = (0..length)
                    .map(|index| if bits & (1 << index) == 0 { '(' } else { ')' })
                    .collect();
                let up = input.bytes().filter(|&byte| byte == b'(').count();
                assert_eq!(
                    part1(&input).unwrap(),
                    (2 * up as i64 - length as i64).to_string()
                );
                let first = (1..=length).find(|&end| {
                    let up = input[..end].bytes().filter(|&byte| byte == b'(').count();
                    2 * up < end
                });
                match first {
                    Some(position) => assert_eq!(part2(&input).unwrap(), position.to_string()),
                    None => assert!(matches!(part2(&input), Err(SolveError::InvalidInput(_)))),
                }
            }
        }
    }

    #[test]
    fn validation_and_outer_whitespace() {
        let valid = ")(((())))";
        for solver in [part1, part2] {
            assert_eq!(
                solver(&format!(" \t\r\n{valid}\r\n ")).unwrap(),
                solver(valid).unwrap()
            );
            for input in ["", " \r\n", "( )", "(\n)", ")x", ")é"] {
                assert!(solver(input).is_err(), "{input:?}");
            }
            assert!(solver(")x").unwrap_err().to_string().starts_with("step 2:"));
        }
    }

    #[test]
    fn checked_floor_boundaries() {
        assert!(next_floor(i64::MAX, b'(').is_err());
        assert!(next_floor(i64::MIN, b')').is_err());
        assert_eq!(next_floor(i64::MAX, b')').unwrap(), i64::MAX - 1);
    }

    #[test]
    fn test_part1_input() {
        scaffold_test_result(2015, 1, "input.txt", "input-spec.1.txt", part1);
    }

    #[test]
    fn test_part2_input() {
        scaffold_test_result(2015, 1, "input.txt", "input-spec.2.txt", part2);
    }
}
