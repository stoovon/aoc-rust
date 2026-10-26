use svutils::SolveError;

fn winning_hold_count(time: i64, record: i64) -> i64 {
    if time < 2 {
        return 0;
    }
    let wins = |hold: i64| i128::from(hold) * i128::from(time - hold) > i128::from(record);
    let peak = time / 2;
    if !wins(peak) {
        return 0;
    }

    // On the rising half, find the first hold that strictly beats the record.
    let mut first = 1;
    let mut last = peak;
    while first < last {
        let middle = first + (last - first) / 2;
        if wins(middle) {
            last = middle;
        } else {
            first = middle + 1;
        }
    }
    // Symmetry puts the final winning hold at time - first.
    time - first - first + 1
}

struct RaceInput<'a> {
    times: Vec<&'a str>,
    distances: Vec<&'a str>,
}

fn parse_row<'a>(line: &'a str, label: &str) -> Result<Vec<&'a str>, SolveError> {
    let numbers = line
        .strip_prefix(label)
        .ok_or_else(|| SolveError::InvalidInput(format!("expected '{label}' header")))?;
    let tokens: Vec<_> = numbers.split_ascii_whitespace().collect();
    if tokens.is_empty() {
        return Err(SolveError::InvalidInput(format!("{label} row is empty")));
    }
    if tokens
        .iter()
        .any(|token| !token.bytes().all(|byte| byte.is_ascii_digit()))
    {
        return Err(SolveError::InvalidInput(
            "race values must contain only decimal digits".into(),
        ));
    }
    Ok(tokens)
}

fn parse_input(input: &str) -> Result<RaceInput<'_>, SolveError> {
    if input.trim().is_empty() {
        return Err(SolveError::EmptyInput);
    }
    let mut lines = input.lines();
    let times = parse_row(lines.next().ok_or(SolveError::EmptyInput)?, "Time:")?;
    let distances = parse_row(
        lines
            .next()
            .ok_or_else(|| SolveError::InvalidInput("missing Distance row".into()))?,
        "Distance:",
    )?;
    if lines.any(|line| !line.trim().is_empty()) {
        return Err(SolveError::InvalidInput(
            "unexpected extra race input row".into(),
        ));
    }
    if times.len() != distances.len() {
        return Err(SolveError::InvalidInput(
            "time and distance counts differ".into(),
        ));
    }
    Ok(RaceInput { times, distances })
}

pub fn part1(input: &str) -> Result<String, SolveError> {
    let RaceInput { times, distances } = parse_input(input)?;
    let mut total = 1_i64;
    for (time, distance) in times.into_iter().zip(distances) {
        let count = winning_hold_count(time.parse()?, distance.parse()?);
        total = total
            .checked_mul(count)
            .ok_or(SolveError::ArithmeticOverflow)?;
    }
    Ok(total.to_string())
}

pub fn part2(input: &str) -> Result<String, SolveError> {
    let RaceInput { times, distances } = parse_input(input)?;
    let time = times.into_iter().collect::<String>().parse()?;
    let distance = distances.into_iter().collect::<String>().parse()?;
    Ok(winning_hold_count(time, distance).to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use svutils::scaffold_test_result as scaffold_test;

    const YEAR: i16 = 2023;
    const DAY: i16 = 6;

    #[test]
    fn winning_intervals_exclude_ties_and_handle_odd_and_even_peaks() {
        for (time, record, expected) in [
            (10, 20, 5),
            (10, 24, 1),
            (10, 25, 0),
            (10, 26, 0),
            (9, 19, 2),
            (9, 20, 0),
            (6, 8, 1),
            (10, 0, 9),
            (0, 0, 0),
            (1, 0, 0),
            (2, 0, 1),
        ] {
            assert_eq!(
                winning_hold_count(time, record),
                expected,
                "time={time}, record={record}"
            );
        }
    }

    #[test]
    fn binary_search_matches_exhaustive_counting_for_small_races() {
        for time in 0..32 {
            for record in 0..=time * time / 4 + 1 {
                let expected = (1..time)
                    .filter(|&hold| hold * (time - hold) > record)
                    .count() as i64;
                assert_eq!(winning_hold_count(time, record), expected);
            }
        }
    }

    #[test]
    fn large_races_do_not_overflow_distance_arithmetic() {
        assert_eq!(winning_hold_count(i64::MAX, 0), i64::MAX - 1);
        assert_eq!(winning_hold_count(i64::MAX, i64::MAX), i64::MAX - 3);
    }

    #[test]
    fn race_count_product_reports_overflow() {
        let input = "Time: 4000000000 4000000000\nDistance: 0 0";
        assert!(matches!(part1(input), Err(SolveError::ArithmeticOverflow)));
    }

    #[test]
    fn malformed_race_input_returns_errors_from_both_parts() {
        for input in [
            "",
            " ",
            "Time: 8",
            "Duration: 8\nDistance: 12",
            "Time: 8\nRecord: 12",
            "Time 8\nDistance: 12",
            "Time: 8 10\nDistance: 12",
            "Time: 8\nDistance: 12 20",
            "Time:\nDistance:",
            "Time: 8 oops\nDistance: 12 20",
            "Time: 8\nDistance: -1",
            "Time: +8\nDistance: 12",
            "Time: 8\nDistance: 1.2",
            "Time: 8\nDistance: 12\nextra",
            "Time: 9999999999999999999999999999\nDistance: 1",
        ] {
            for solver in [part1, part2] {
                assert!(solver(input).is_err(), "{input}");
            }
        }
    }

    #[test]
    fn valid_spacing_and_leading_zeroes_preserve_part_two_concatenation() {
        let input = "Time:  1\t02\r\nDistance: 0  00\r\n\r\n";
        assert_eq!(part1(input).unwrap(), "0");
        assert_eq!(part2(input).unwrap(), "101");
    }

    #[test]
    fn test_part1_input() {
        scaffold_test(YEAR, DAY, "input.txt", "input-spec.1.txt", part1);
    }

    #[test]
    fn test_part2_input() {
        scaffold_test(YEAR, DAY, "input.txt", "input-spec.2.txt", part2);
    }
}
