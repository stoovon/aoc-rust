use std::collections::HashSet;
use svutils::SolveError;

fn parse_changes(input: &str) -> Result<Vec<i64>, SolveError> {
    if input.trim().is_empty() {
        return Err(SolveError::EmptyInput);
    }
    input
        .lines()
        .enumerate()
        .map(|(index, line)| {
            let value = line.trim();
            let digits = value
                .strip_prefix('-')
                .or_else(|| value.strip_prefix('+'))
                .unwrap_or(value);
            let parsed = if digits.is_empty() || !digits.bytes().all(|c| c.is_ascii_digit()) {
                Err(SolveError::InvalidInput(
                    "expected one signed decimal frequency change".into(),
                ))
            } else {
                value.parse::<i64>().map_err(SolveError::from)
            };
            parsed.map_err(|error| error.at_line(index + 1))
        })
        .collect()
}

pub fn part1(input: &str) -> Result<String, SolveError> {
    parse_changes(input)?
        .into_iter()
        .try_fold(0i128, |total, change| {
            total
                .checked_add(i128::from(change))
                .ok_or(SolveError::ArithmeticOverflow)
        })
        .map(|total| total.to_string())
}

struct Prefix {
    normalized: i128,
    remainder: i128,
    offset: usize,
}

fn first_repeat(changes: &[i64]) -> Result<i128, SolveError> {
    if changes.is_empty() {
        return Err(SolveError::EmptyInput);
    }
    let mut seen = HashSet::from([0i128]);
    let mut prefixes = Vec::with_capacity(changes.len());
    let mut total = 0i128;
    for (offset, &change) in changes.iter().enumerate() {
        prefixes.push(Prefix {
            normalized: total,
            remainder: 0,
            offset,
        });
        total = total
            .checked_add(i128::from(change))
            .ok_or(SolveError::ArithmeticOverflow)?;
        if !seen.insert(total) {
            return Ok(total);
        }
    }
    // Zero drift necessarily repeats zero during the first-pass check above.
    let shift = total.checked_abs().ok_or(SolveError::ArithmeticOverflow)?;
    debug_assert!(shift > 0);
    for prefix in &mut prefixes {
        if total < 0 {
            prefix.normalized = prefix
                .normalized
                .checked_neg()
                .ok_or(SolveError::ArithmeticOverflow)?;
        }
        prefix.remainder = prefix.normalized.rem_euclid(shift);
    }
    prefixes.sort_unstable_by_key(|prefix| (prefix.remainder, prefix.normalized));
    let mut best: Option<((i128, usize), i128)> = None;
    for pair in prefixes.windows(2) {
        let (low, high) = (&pair[0], &pair[1]);
        if low.remainder != high.remainder {
            continue;
        }
        // The next higher prefix is the earliest target for this phase.
        let cycles = high
            .normalized
            .checked_sub(low.normalized)
            .ok_or(SolveError::ArithmeticOverflow)?
            / shift;
        let when = (cycles, low.offset);
        if best.as_ref().is_none_or(|(earliest, _)| when < *earliest) {
            let frequency = if total < 0 {
                high.normalized
                    .checked_neg()
                    .ok_or(SolveError::ArithmeticOverflow)?
            } else {
                high.normalized
            };
            best = Some((when, frequency));
        }
    }
    best.map(|(_, frequency)| frequency)
        .ok_or_else(|| SolveError::InvalidInput("frequency never repeats".into()))
}

pub fn part2(input: &str) -> Result<String, SolveError> {
    first_repeat(&parse_changes(input)?).map(|frequency| frequency.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use svutils::scaffold_test_result;

    #[test]
    fn repeats_respect_initial_zero_and_chronological_order() {
        for (input, want) in [
            ("+6\n-6", "0"),
            ("+6\n+4\n-4\n-6", "6"),
            ("+6\n-4", "6"),
            ("-6\n+4", "-6"),
            ("+8\n-3\n-2", "8"),
            ("0", "0"),
        ] {
            assert_eq!(part2(input).unwrap(), want);
        }
    }

    #[test]
    fn reports_no_repeat_and_jumps_over_enormous_cycle_counts() {
        for input in ["+2", "-2", "+5\n-2", "-5\n+2", "+2\n+4"] {
            assert!(part2(input)
                .unwrap_err()
                .to_string()
                .contains("never repeats"));
        }
        assert_eq!(
            part2("1000000000000\n-999999999999").unwrap(),
            "1000000000000"
        );
        assert_eq!(
            part2("-1000000000000\n999999999999").unwrap(),
            "-1000000000000"
        );
    }

    #[test]
    fn validates_the_entire_input_and_uses_wider_frequency_totals() {
        for input in [
            "",
            " \n\t",
            "+6\n-6\njunk",
            "2 4",
            "2\n\n4",
            "--2",
            "1.5",
            "9223372036854775808",
        ] {
            for solve in [part1, part2] {
                assert!(solve(input).is_err());
            }
        }
        for solve in [part1, part2] {
            assert!(solve("+6\n-6\njunk")
                .unwrap_err()
                .to_string()
                .starts_with("line 3:"));
        }
        assert_eq!(part1(" +6\r\n -4\r\n").unwrap(), "2");
        let max = i64::MAX;
        assert_eq!(
            part1(&format!("{max}\n{max}")).unwrap(),
            (i128::from(max) * 2).to_string()
        );
        let input = format!("{max}\n{}\n{}\n{}", max - 2, -max + 4, i64::MIN);
        assert_eq!(part1(&input).unwrap(), "1");
        assert_eq!(part2(&input).unwrap(), (i128::from(max) + 2).to_string());
        assert!(first_repeat(&[]).is_err());
    }

    fn simulate(changes: &[i64]) -> Option<i128> {
        let mut seen = HashSet::from([0i128]);
        let mut frequency = 0i128;
        // For at most six changes of magnitude at most two, 200 steps
        // exceed the possible first-repeat time implied by the prefix span.
        for &change in changes.iter().cycle().take(200) {
            frequency += i128::from(change);
            if !seen.insert(frequency) {
                return Some(frequency);
            }
        }
        None
    }

    #[test]
    fn finite_analysis_matches_exhaustive_small_simulations() {
        for length in 1..=6 {
            for mut code in 0..5usize.pow(length) {
                let changes: Vec<_> = (0..length)
                    .map(|_| {
                        let value = (code % 5) as i64 - 2;
                        code /= 5;
                        value
                    })
                    .collect();
                let actual = first_repeat(&changes);
                match simulate(&changes) {
                    Some(want) => assert_eq!(actual.unwrap(), want, "{changes:?}"),
                    None => assert!(
                        actual.unwrap_err().to_string().contains("never repeats"),
                        "{changes:?}"
                    ),
                }
            }
        }
    }

    #[test]
    fn test_part1_input() {
        scaffold_test_result(2018, 1, "input.txt", "input-spec.1.txt", part1);
    }
    #[test]
    fn test_part2_input() {
        scaffold_test_result(2018, 1, "input.txt", "input-spec.2.txt", part2);
    }
}
