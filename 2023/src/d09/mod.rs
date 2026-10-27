use svutils::SolveError;

fn parse_history(line: &str) -> Result<Vec<i64>, SolveError> {
    if line.trim().is_empty() {
        return Err(SolveError::InvalidInput("history row is blank".into()));
    }
    Ok(line
        .split_whitespace()
        .map(str::parse::<i64>)
        .collect::<Result<Vec<_>, _>>()?)
}

fn difference_edges(
    mut history: Vec<i64>,
    edge: impl Fn(&[i64]) -> i64,
) -> Result<Vec<i64>, SolveError> {
    let mut next = Vec::new();
    let mut edges = Vec::new();
    while history.iter().any(|&n| n != 0) {
        edges.push(edge(&history));
        for pair in history.windows(2) {
            next.push(
                pair[1]
                    .checked_sub(pair[0])
                    .ok_or(SolveError::ArithmeticOverflow)?,
            );
        }
        (history, next) = (next, history);
        next.clear();
    }
    Ok(edges)
}

fn following_value(history: Vec<i64>) -> Result<i64, SolveError> {
    difference_edges(history, |row| row[row.len() - 1])?
        .into_iter()
        .rev()
        .try_fold(0_i64, |acc, end| {
            end.checked_add(acc).ok_or(SolveError::ArithmeticOverflow)
        })
}

fn preceding_value(history: Vec<i64>) -> Result<i64, SolveError> {
    difference_edges(history, |row| row[0])?
        .into_iter()
        .rev()
        .try_fold(0_i64, |acc, start| {
            start.checked_sub(acc).ok_or(SolveError::ArithmeticOverflow)
        })
}

fn sum_histories(
    input: &str,
    predict: impl Fn(Vec<i64>) -> Result<i64, SolveError>,
) -> Result<String, SolveError> {
    if input.trim().is_empty() {
        return Err(SolveError::EmptyInput);
    }
    let total = input.lines().try_fold(0_i64, |total, line| {
        let value = predict(parse_history(line)?)?;
        total
            .checked_add(value)
            .ok_or(SolveError::ArithmeticOverflow)
    })?;
    Ok(total.to_string())
}

pub fn part1(input: &str) -> Result<String, SolveError> {
    sum_histories(input, following_value)
}

pub fn part2(input: &str) -> Result<String, SolveError> {
    sum_histories(input, preceding_value)
}

#[cfg(test)]
mod tests {
    use super::*;
    use svutils::scaffold_test_result;

    const YEAR: i16 = 2023;
    const DAY: i16 = 9;

    #[test]
    fn test_part1_input() {
        scaffold_test_result(YEAR, DAY, "input.txt", "input-spec.1.txt", part1);
    }

    #[test]
    fn test_part2_input() {
        scaffold_test_result(YEAR, DAY, "input.txt", "input-spec.2.txt", part2);
    }

    #[test]
    fn invalid_numbers_return_errors() {
        for input in ["2 nope 8", "9223372036854775808"] {
            assert!(matches!(part1(input), Err(SolveError::InvalidNumber(_))));
            assert!(matches!(part2(input), Err(SolveError::InvalidNumber(_))));
        }
    }

    #[test]
    fn empty_documents_and_blank_rows_are_rejected() {
        for input in ["", " ", "\n", "\r\n \t"] {
            assert!(matches!(part1(input), Err(SolveError::EmptyInput)));
            assert!(matches!(part2(input), Err(SolveError::EmptyInput)));
        }
        for input in ["\n2 5 8", "2 5 8\n\n3 6 9", "2 5 8\n \t", "2 5 8\n\n"] {
            assert!(matches!(part1(input), Err(SolveError::InvalidInput(_))));
            assert!(matches!(part2(input), Err(SolveError::InvalidInput(_))));
        }
    }

    #[test]
    fn normal_whitespace_and_trailing_newlines_are_accepted() {
        for input in ["2 5 8", "2 5 8\n", " 2\t5  8 \r\n"] {
            assert_eq!(part1(input).unwrap(), "11");
            assert_eq!(part2(input).unwrap(), "-1");
        }
    }

    #[test]
    fn arithmetic_overflow_returns_errors() {
        let inputs = [
            format!("{} 0", i64::MIN),
            format!("{}\n1", i64::MAX),
            format!("{}\n-1", i64::MIN),
        ];
        for input in inputs {
            assert!(matches!(part1(&input), Err(SolveError::ArithmeticOverflow)));
            assert!(matches!(part2(&input), Err(SolveError::ArithmeticOverflow)));
        }
    }

    #[test]
    fn each_part_only_checks_its_own_prediction() {
        let following_overflows = format!("{} {}", i64::MAX - 1, i64::MAX);
        assert!(matches!(
            part1(&following_overflows),
            Err(SolveError::ArithmeticOverflow)
        ));
        assert_eq!(
            part2(&following_overflows).unwrap(),
            (i64::MAX - 2).to_string()
        );

        let preceding_overflows = format!("{} {}", i64::MIN, i64::MIN + 1);
        assert_eq!(
            part1(&preceding_overflows).unwrap(),
            (i64::MIN + 2).to_string()
        );
        assert!(matches!(
            part2(&preceding_overflows),
            Err(SolveError::ArithmeticOverflow)
        ));
    }

    #[test]
    fn predicts_zero_constant_short_and_curved_histories() {
        for (input, following, preceding) in [
            ("0 0 0", "0", "0"),
            ("7", "7", "7"),
            ("-4 -4 -4", "-4", "-4"),
            ("5 2", "-1", "8"),
            ("1 4 9 16", "25", "0"),
            ("0 0\n2 5 8\n1 4 9 16", "36", "-1"),
        ] {
            assert_eq!(part1(input).unwrap(), following);
            assert_eq!(part2(input).unwrap(), preceding);
        }
    }
}
