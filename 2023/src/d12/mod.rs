use svutils::SolveError;

use itertools::Itertools;

fn count_arrangements(spring: &str, counts: &[usize]) -> Result<i64, SolveError> {
    let spring = format!(".{}", spring.trim_end_matches('.'));
    let spring = spring.chars().collect_vec();

    let state_count = spring
        .len()
        .checked_add(1)
        .ok_or(SolveError::ArithmeticOverflow)?;
    let mut previous_groups = vec![0_i64; state_count];

    // With no groups placed, a prefix has one arrangement until a forced '#'.
    previous_groups[0] = 1;

    for (i, _) in spring.iter().take_while(|&&c| c != '#').enumerate() {
        previous_groups[i + 1] = 1;
    }

    // Each buffer entry counts arrangements for a prefix of the padded pattern.
    for &count in counts {
        let mut current_groups = vec![0_i64; state_count];
        let mut damageable_run = 0;

        for (i, &c) in spring.iter().enumerate() {
            if c != '.' {
                damageable_run += 1;
            } else {
                damageable_run = 0;
            }

            // Treat this cell as working, retaining the groups already placed.
            if c != '#' {
                current_groups[i + 1] = current_groups[i + 1]
                    .checked_add(current_groups[i])
                    .ok_or(SolveError::ArithmeticOverflow)?;
            }

            // Finish this group here, with a working separator before it.
            if damageable_run >= count && spring[i - count] != '#' {
                current_groups[i + 1] = current_groups[i + 1]
                    .checked_add(previous_groups[i - count])
                    .ok_or(SolveError::ArithmeticOverflow)?;
            }
        }

        previous_groups = current_groups;
    }

    Ok(previous_groups[spring.len()])
}

struct Record {
    spring: String,
    groups: Vec<usize>,
}

fn parse_record(line: &str) -> Result<Record, SolveError> {
    let mut fields = line.split_whitespace();
    let spring = fields
        .next()
        .ok_or_else(|| SolveError::InvalidInput("missing spring pattern".into()))?;
    if !spring
        .bytes()
        .all(|tile| matches!(tile, b'.' | b'#' | b'?'))
    {
        return Err(SolveError::InvalidInput(
            "spring pattern accepts only '.', '#' and '?'".into(),
        ));
    }
    let groups = fields
        .next()
        .ok_or_else(|| SolveError::InvalidInput("missing damaged-group lengths".into()))?;
    if fields.next().is_some() {
        return Err(SolveError::InvalidInput(
            "unexpected field after group lengths".into(),
        ));
    }
    let groups = groups
        .split(',')
        .map(|token| {
            if token.is_empty() || !token.bytes().all(|byte| byte.is_ascii_digit()) {
                return Err(SolveError::InvalidInput(
                    "group lengths must be positive decimal integers".into(),
                ));
            }
            let length: usize = token.parse()?;
            if length == 0 {
                return Err(SolveError::InvalidInput(
                    "group lengths must be positive".into(),
                ));
            }
            Ok(length)
        })
        .collect::<Result<Vec<_>, SolveError>>()?;
    Ok(Record {
        spring: spring.to_owned(),
        groups,
    })
}

fn unfold(record: Record) -> Result<Record, SolveError> {
    let spring_length = record
        .spring
        .len()
        .checked_mul(5)
        .and_then(|length| length.checked_add(4))
        .ok_or(SolveError::ArithmeticOverflow)?;
    let group_count = record
        .groups
        .len()
        .checked_mul(5)
        .ok_or(SolveError::ArithmeticOverflow)?;
    let mut spring = String::with_capacity(spring_length);
    let mut groups = Vec::with_capacity(group_count);
    for copy in 0..5 {
        if copy != 0 {
            spring.push('?');
        }
        spring.push_str(&record.spring);
        groups.extend_from_slice(&record.groups);
    }
    Ok(Record { spring, groups })
}

fn count_record(record: &Record) -> Result<i64, SolveError> {
    let count = count_arrangements(&record.spring, &record.groups)?;
    if count == 0 {
        return Err(SolveError::InvalidInput(
            "spring record has no valid arrangements".into(),
        ));
    }
    Ok(count)
}

fn sum_records(
    input: &str,
    count: impl Fn(Record) -> Result<i64, SolveError>,
) -> Result<String, SolveError> {
    if input.trim().is_empty() {
        return Err(SolveError::EmptyInput);
    }
    let mut total = 0_i64;
    for (index, line) in input.lines().enumerate() {
        let record = parse_record(line).map_err(|error| error.at_line(index + 1))?;
        let arrangements = count(record).map_err(|error| error.at_line(index + 1))?;
        total = total
            .checked_add(arrangements)
            .ok_or_else(|| SolveError::ArithmeticOverflow.at_line(index + 1))?;
    }
    Ok(total.to_string())
}

pub fn part1(input: &str) -> Result<String, SolveError> {
    sum_records(input, |record| count_record(&record))
}

pub fn part2(input: &str) -> Result<String, SolveError> {
    sum_records(input, |record| count_record(&unfold(record)?))
}

#[cfg(test)]
mod tests {
    use super::*;
    use svutils::scaffold_test_result;

    const YEAR: i16 = 2023;
    const DAY: i16 = 12;

    #[test]
    fn test_part1_input() {
        scaffold_test_result(YEAR, DAY, "input.txt", "input-spec.1.txt", part1);
    }

    #[test]
    fn test_part2_input() {
        scaffold_test_result(YEAR, DAY, "input.txt", "input-spec.2.txt", part2);
    }

    #[test]
    fn malformed_records_are_rejected() {
        for input in [
            "?",
            "? 0",
            "? nope",
            "? -1",
            "? +1",
            "? 1,",
            "? ,1",
            "? 1,,2",
            "x 1",
            "é 1",
            "? 1 extra",
            "? 1\n\n? 1",
        ] {
            assert!(part1(input).is_err(), "{input:?}");
            assert!(part2(input).is_err(), "{input:?}");
        }
        for input in ["", " \t\r\n"] {
            assert!(matches!(part1(input), Err(SolveError::EmptyInput)));
            assert!(matches!(part2(input), Err(SolveError::EmptyInput)));
        }
    }

    #[test]
    fn impossible_records_are_rejected_with_line_context() {
        for input in ["# 2", ". 1", "## 1", "#.# 2", "? 1\n# 2"] {
            for solver in [part1, part2] {
                let error = solver(input).unwrap_err();
                assert!(matches!(error, SolveError::AtLine { .. }));
                assert!(error.to_string().contains("no valid arrangements"));
            }
        }
    }

    #[test]
    fn whitespace_and_trailing_newlines_are_accepted() {
        for input in ["?? 1", "  ??\t1 \n", "?? 1\r\n"] {
            assert_eq!(part1(input).unwrap(), "2");
            assert_eq!(part2(input).unwrap(), "252");
        }
    }

    #[test]
    fn arrangement_counts_and_document_totals_report_overflow() {
        let spring = "?".repeat(100);
        assert!(matches!(
            count_arrangements(&spring, &[1; 25]),
            Err(SolveError::ArithmeticOverflow)
        ));
        // Singleton groups have C(n-k+1,k) placements; this fits once but not twice.
        let input = format!("{} {}", "?".repeat(95), vec!["1"; 23].join(","));
        assert_eq!(part1(&input).unwrap(), "5685248339583664176");
        assert!(matches!(part1(&format!("{input}\n{input}")),
            Err(SolveError::AtLine { line: 2, error }) if matches!(*error, SolveError::ArithmeticOverflow)));
    }

    #[test]
    fn dynamic_counts_match_exhaustive_small_patterns() {
        use std::collections::HashMap;
        for length in 1..=6_u32 {
            for mut encoding in 0..3_usize.pow(length) {
                let pattern: String = (0..length)
                    .map(|_| {
                        let tile = ['.', '#', '?'][encoding % 3];
                        encoding /= 3;
                        tile
                    })
                    .collect();
                let unknowns = pattern.bytes().filter(|&tile| tile == b'?').count();
                let mut expected: HashMap<Vec<usize>, i64> = HashMap::new();
                for assignment in 0..(1_usize << unknowns) {
                    let mut unknown = 0;
                    let resolved: String = pattern
                        .chars()
                        .map(|tile| {
                            if tile != '?' {
                                return tile;
                            }
                            let damaged = assignment & (1 << unknown) != 0;
                            unknown += 1;
                            if damaged {
                                '#'
                            } else {
                                '.'
                            }
                        })
                        .collect();
                    let groups = resolved
                        .split('.')
                        .filter(|run| !run.is_empty())
                        .map(str::len)
                        .collect::<Vec<_>>();
                    *expected.entry(groups).or_default() += 1;
                }
                for (groups, count) in expected {
                    assert_eq!(
                        count_arrangements(&pattern, &groups).unwrap(),
                        count,
                        "pattern={pattern}, groups={groups:?}"
                    );
                }
                assert_eq!(
                    count_arrangements(&pattern, &[length as usize + 1]).unwrap(),
                    0
                );
            }
        }
    }
}
