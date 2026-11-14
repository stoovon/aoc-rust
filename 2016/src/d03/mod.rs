use svutils::SolveError;

fn parse_row(line: &str) -> Result<[u64; 3], SolveError> {
    let mut tokens = line.split_whitespace();
    let (a, b, c) = match (tokens.next(), tokens.next(), tokens.next(), tokens.next()) {
        (Some(a), Some(b), Some(c), None) => (a, b, c),
        _ => {
            return Err(SolveError::InvalidInput(
                "expected three side lengths".into(),
            ))
        }
    };

    let parse_side = |token: &str| -> Result<u64, SolveError> {
        let side = token.parse::<u64>()?;
        if side == 0 {
            return Err(SolveError::InvalidInput(
                "side lengths must be positive".into(),
            ));
        }
        Ok(side)
    };
    Ok([parse_side(a)?, parse_side(b)?, parse_side(c)?])
}

fn rows(
    input: &str,
) -> Result<impl Iterator<Item = Result<[u64; 3], SolveError>> + '_, SolveError> {
    if input.is_empty() {
        return Err(SolveError::EmptyInput);
    }
    Ok(input
        .lines()
        .enumerate()
        .map(|(index, line)| parse_row(line).map_err(|error| error.at_line(index + 1))))
}

fn valid_triangle(mut sides: [u64; 3]) -> bool {
    sides.sort_unstable();
    let [short, middle, long] = sides;
    short > long - middle
}

pub fn part1(input: &str) -> Result<String, SolveError> {
    let mut count = 0usize;
    for row in rows(input)? {
        count += usize::from(valid_triangle(row?));
    }
    Ok(count.to_string())
}

pub fn part2(input: &str) -> Result<String, SolveError> {
    let mut count = 0usize;
    let mut row_index = 0;
    let mut columns = [[0; 3]; 3];

    for row in rows(input)? {
        for (column, side) in columns.iter_mut().zip(row?) {
            column[row_index] = side;
        }
        row_index += 1;
        if row_index == 3 {
            count += columns
                .into_iter()
                .filter(|&sides| valid_triangle(sides))
                .count();
            row_index = 0;
        }
    }

    if row_index != 0 {
        return Err(SolveError::InvalidInput(
            "expected complete groups of three rows".into(),
        ));
    }
    Ok(count.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use svutils::{scaffold_test_result, Solver};

    #[test]
    fn row_and_column_triangles() {
        let input = " 2 3 5\r\n2 4 7\r\n3 5 4";
        assert_eq!(part1(input).unwrap(), "1");
        assert_eq!(part2(input).unwrap(), "3");
    }

    #[test]
    fn triangle_inequality_handles_large_lengths() {
        assert!(valid_triangle([u64::MAX, 1, u64::MAX]));
        assert!(!valid_triangle([u64::MAX, 1, u64::MAX - 1]));
    }

    #[test]
    fn rejects_bad_rows_in_both_parts() {
        for solve in [part1 as Solver, part2] {
            assert!(matches!(solve(""), Err(SolveError::EmptyInput)));
            for row in [
                "",
                "1 2",
                "1 2 3 4",
                "1 2 0",
                "1 -2 3",
                "x 2 3",
                "18446744073709551616 2 3",
            ] {
                let input = format!("1 1 1\n{row}\n1 1 1");
                assert!(matches!(
                    solve(&input),
                    Err(SolveError::AtLine { line: 2, .. })
                ));
            }
        }
    }

    #[test]
    fn rejects_incomplete_column_groups() {
        for input in ["1 1 1", "1 1 1\n1 1 1"] {
            assert!(matches!(part2(input), Err(SolveError::InvalidInput(_))));
        }
    }

    #[test]
    fn test_part1_input() {
        scaffold_test_result(2016, 3, "input.txt", "input-spec.1.txt", part1);
    }

    #[test]
    fn test_part2_input() {
        scaffold_test_result(2016, 3, "input.txt", "input-spec.2.txt", part2);
    }
}
