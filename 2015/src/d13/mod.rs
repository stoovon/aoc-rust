use std::collections::{HashMap, HashSet};
use svutils::SolveError;

struct Record<'a> {
    person: &'a str,
    neighbour: &'a str,
    happiness: i64,
}

fn parse_record(line: &str) -> Result<Record<'_>, SolveError> {
    let invalid =
        || SolveError::InvalidInput("expected a directed happiness statement ending in '.'".into());
    let (person, rest) = line.split_once(" would ").ok_or_else(invalid)?;
    let (direction, rest) = rest.split_once(' ').ok_or_else(invalid)?;
    let (amount, neighbour) = rest
        .split_once(" happiness units by sitting next to ")
        .ok_or_else(invalid)?;
    let neighbour = neighbour.strip_suffix('.').ok_or_else(invalid)?;
    let valid_name = |name: &str| {
        !name.is_empty()
            && name
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
    };
    if !valid_name(person) || !valid_name(neighbour) || person == neighbour {
        return Err(SolveError::InvalidInput(
            "expected two distinct guest names".into(),
        ));
    }
    if amount.is_empty() || !amount.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err(SolveError::InvalidInput(
            "happiness magnitude must be a nonnegative decimal integer".into(),
        ));
    }
    let amount: u64 = amount.parse()?;
    let signed = match direction {
        "gain" => i128::from(amount),
        "lose" => -i128::from(amount),
        _ => return Err(SolveError::InvalidInput("expected 'gain' or 'lose'".into())),
    };
    let happiness = i64::try_from(signed).map_err(|_| SolveError::ArithmeticOverflow)?;
    Ok(Record {
        person,
        neighbour,
        happiness,
    })
}

fn score_table_layout(guests: usize) -> Result<(usize, usize), SolveError> {
    let unsupported = || SolveError::InvalidInput("seating exceeds the subset table budget".into());
    let others = guests
        .checked_sub(1)
        .filter(|&count| count > 0)
        .ok_or_else(|| SolveError::InvalidInput("expected at least two guests".into()))?;
    let shift = u32::try_from(others).map_err(|_| unsupported())?;
    let subsets = 1usize.checked_shl(shift).ok_or_else(unsupported)?;
    let entries = subsets
        .checked_mul(others)
        .filter(|&count| count <= 1 << 20)
        .ok_or_else(unsupported)?;
    Ok((subsets, entries))
}

struct Party {
    pair_happiness: Vec<Vec<i64>>,
}

impl Party {
    fn parse(input: &str) -> Result<Self, SolveError> {
        if input.trim().is_empty() {
            return Err(SolveError::EmptyInput);
        }
        let mut records = HashMap::new();
        let mut people = HashSet::new();
        for (index, line) in input.lines().enumerate() {
            let record = parse_record(line).map_err(|error| error.at_line(index + 1))?;
            if records
                .insert((record.person, record.neighbour), record.happiness)
                .is_some()
            {
                return Err(
                    SolveError::InvalidInput("duplicate directed happiness entry".into())
                        .at_line(index + 1),
                );
            }
            people.extend([record.person, record.neighbour]);
            score_table_layout(people.len()).map_err(|error| error.at_line(index + 1))?;
        }
        let mut people: Vec<_> = people.into_iter().collect();
        people.sort_unstable();
        let mut pair_happiness = vec![vec![0; people.len()]; people.len()];
        for a in 0..people.len() {
            for b in a + 1..people.len() {
                let directed = |from, to| {
                    records
                        .get(&(people[from], people[to]))
                        .copied()
                        .ok_or_else(|| {
                            SolveError::InvalidInput(format!(
                                "missing happiness entry from '{}' to '{}'",
                                people[from], people[to]
                            ))
                        })
                };
                let happiness = directed(a, b)?
                    .checked_add(directed(b, a)?)
                    .ok_or(SolveError::ArithmeticOverflow)?;
                pair_happiness[a][b] = happiness;
                pair_happiness[b][a] = happiness;
            }
        }
        Ok(Self { pair_happiness })
    }

    fn with_neutral_guest(mut self) -> Result<Self, SolveError> {
        let guests = self
            .pair_happiness
            .len()
            .checked_add(1)
            .ok_or(SolveError::ArithmeticOverflow)?;
        score_table_layout(guests)?;
        for row in &mut self.pair_happiness {
            row.push(0);
        }
        self.pair_happiness.push(vec![0; guests]);
        Ok(self)
    }

    fn maximum_happiness(&self) -> Result<i64, SolveError> {
        let (subsets, entries) = score_table_layout(self.pair_happiness.len())?;
        let others = self.pair_happiness.len() - 1;
        let mut scores: Vec<Option<i64>> = vec![None; entries];
        // Fix guest 0 to remove rotations; subset bits describe the other guests.
        for last in 0..others {
            scores[(1 << last) * others + last] = Some(self.pair_happiness[0][last + 1]);
        }
        for seated in 1..subsets {
            for last in 0..others {
                let Some(current) = scores[seated * others + last] else {
                    continue;
                };
                for next in 0..others {
                    if seated & (1 << next) != 0 {
                        continue;
                    }
                    let candidate = current
                        .checked_add(self.pair_happiness[last + 1][next + 1])
                        .ok_or(SolveError::ArithmeticOverflow)?;
                    let entry = &mut scores[(seated | 1 << next) * others + next];
                    *entry = Some(entry.map_or(candidate, |best| best.max(candidate)));
                }
            }
        }
        let mut best = None;
        for (last, &score) in scores[(subsets - 1) * others..].iter().enumerate() {
            let Some(score) = score else { continue };
            let closed = score
                .checked_add(self.pair_happiness[last + 1][0])
                .ok_or(SolveError::ArithmeticOverflow)?;
            best = Some(best.map_or(closed, |best: i64| best.max(closed)));
        }
        best.ok_or_else(|| SolveError::InvalidInput("no complete seating arrangement".into()))
    }
}

pub fn part1(input: &str) -> Result<String, SolveError> {
    Party::parse(input)?
        .maximum_happiness()
        .map(|score| score.to_string())
}

pub fn part2(input: &str) -> Result<String, SolveError> {
    Party::parse(input)?
        .with_neutral_guest()?
        .maximum_happiness()
        .map(|score| score.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use svutils::scaffold_test_result;

    fn statements(directed: &[Vec<i64>]) -> String {
        let mut input = String::new();
        for (a, row) in directed.iter().enumerate() {
            for (b, &value) in row.iter().enumerate() {
                if a == b {
                    continue;
                }
                let direction = if value < 0 { "lose" } else { "gain" };
                input.push_str(&format!(
                    "Guest{a} would {direction} {} happiness units by sitting next to Guest{b}.\n",
                    value.unsigned_abs()
                ));
            }
        }
        input
    }

    fn exhaustive_score(directed: &[Vec<i64>], neutral: bool) -> i64 {
        let n = directed.len() + usize::from(neutral);
        let mut order: Vec<_> = (0..n).collect();
        let mut best = i128::MIN;
        // Enumerate all rotations, using the original directed values as the oracle.
        fn visit(order: &mut [usize], at: usize, directed: &[Vec<i64>], best: &mut i128) {
            if at == order.len() {
                let score = order
                    .iter()
                    .zip(order.iter().cycle().skip(1))
                    .map(|(&a, &b)| {
                        if a < directed.len() && b < directed.len() {
                            i128::from(directed[a][b]) + i128::from(directed[b][a])
                        } else {
                            0
                        }
                    })
                    .sum();
                *best = (*best).max(score);
                return;
            }
            for next in at..order.len() {
                order.swap(at, next);
                visit(order, at + 1, directed, best);
                order.swap(at, next);
            }
        }
        visit(&mut order, 0, directed, &mut best);
        i64::try_from(best).unwrap()
    }

    #[test]
    fn synthetic_seating_answers() {
        let input = statements(&[vec![0, 2, 4], vec![6, 0, 8], vec![10, 12, 0]]);
        assert_eq!(part1(&input).unwrap(), "42");
        assert_eq!(part2(&input).unwrap(), "34");
    }

    #[test]
    fn subset_scores_match_exhaustive_seatings() {
        let mut seed = 13u64;
        let mut cases = 0;
        for n in 2..=7 {
            for _ in 0..10 {
                let mut directed = vec![vec![0; n]; n];
                for (a, row) in directed.iter_mut().enumerate() {
                    for (b, value) in row.iter_mut().enumerate() {
                        if a != b {
                            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
                            *value = ((seed >> 32) % 41) as i64 - 20;
                        }
                    }
                }
                let input = statements(&directed);
                assert_eq!(
                    part1(&input).unwrap(),
                    exhaustive_score(&directed, false).to_string()
                );
                assert_eq!(
                    part2(&input).unwrap(),
                    exhaustive_score(&directed, true).to_string()
                );
                cases += 2;
            }
        }
        assert_eq!(cases, 120);
    }

    #[test]
    fn negative_and_zero_scores_are_reachable() {
        let input = statements(&[vec![0, -5], vec![-7, 0]]);
        assert_eq!(part1(&input).unwrap(), "-24");
        assert_eq!(part2(&input).unwrap(), "-12");
        assert_eq!(part1(&statements(&[vec![0, 0], vec![0, 0]])).unwrap(), "0");
        let minimum = Party {
            pair_happiness: vec![vec![0, i64::MIN, 0], vec![i64::MIN, 0, 0], vec![0, 0, 0]],
        };
        assert_eq!(minimum.maximum_happiness().unwrap(), i64::MIN);
    }

    #[test]
    fn neutral_guest_does_not_collide_with_a_real_name() {
        let input = "Me would gain 4 happiness units by sitting next to Bob.\nBob would gain 3 happiness units by sitting next to Me.";
        assert_eq!(part1(input).unwrap(), "14");
        assert_eq!(part2(input).unwrap(), "7");
    }

    #[test]
    fn parser_checks_complete_unique_directed_data() {
        for input in ["", " \n\t"] {
            assert!(matches!(Party::parse(input), Err(SolveError::EmptyInput)));
        }
        for input in [
            "Alice would gain 1 happiness units by sitting next to Bob.",
            "Alice would gain 1 happiness units by sitting next to Alice.",
            "Alice would gain 1 happiness units by sitting next to Bob.\nAlice would lose 1 happiness units by sitting next to Bob.",
        ] {
            assert!(Party::parse(input).is_err());
        }
        let duplicate = "Alice would gain 1 happiness units by sitting next to Bob.\nAlice would gain 1 happiness units by sitting next to Bob.";
        assert!(Party::parse(duplicate)
            .err()
            .unwrap()
            .to_string()
            .starts_with("line 2:"));
        let input = statements(&[vec![0, 2, 4], vec![6, 0, 8], vec![10, 12, 0]]);
        assert_eq!(part1(&input.replace('\n', "\r\n")).unwrap(), "42");
        let reordered = input.lines().rev().collect::<Vec<_>>().join("\n");
        assert_eq!(part1(&reordered).unwrap(), "42");
    }

    #[test]
    fn malformed_records_have_line_context() {
        for line in [
            "invalid",
            "Alice would take 1 happiness units by sitting next to Bob.",
            "Alice would gain -1 happiness units by sitting next to Bob.",
            "Alice would gain one happiness units by sitting next to Bob.",
            "Alice would gain 1 happiness units by sitting next to Bob",
            "",
        ] {
            let input =
                format!("Alice would gain 1 happiness units by sitting next to Bob.\n{line}\n");
            for solver in [part1, part2] {
                assert!(
                    solver(&input)
                        .unwrap_err()
                        .to_string()
                        .starts_with("line 2:"),
                    "{input:?}"
                );
            }
        }
    }

    #[test]
    fn numeric_errors_preserve_sources_and_signed_limits() {
        use std::error::Error;
        let line = "Alice would gain 99999999999999999999999999999999 happiness units by sitting next to Bob.";
        let error = part1(line).unwrap_err();
        assert!(error.to_string().starts_with("line 1:"));
        assert!(error.source().unwrap().source().is_some());
        assert_eq!(
            parse_record(
                "Alice would lose 9223372036854775808 happiness units by sitting next to Bob."
            )
            .unwrap()
            .happiness,
            i64::MIN
        );
        assert!(matches!(
            parse_record(
                "Alice would gain 9223372036854775808 happiness units by sitting next to Bob."
            ),
            Err(SolveError::ArithmeticOverflow)
        ));
    }

    #[test]
    fn pair_path_and_closing_arithmetic_are_checked() {
        let pair_overflow = statements(&[vec![0, i64::MAX], vec![1, 0]]);
        assert!(matches!(
            part1(&pair_overflow),
            Err(SolveError::ArithmeticOverflow)
        ));
        let closing_overflow = statements(&[vec![0, i64::MAX], vec![0, 0]]);
        assert!(matches!(
            part1(&closing_overflow),
            Err(SolveError::ArithmeticOverflow)
        ));
        assert_eq!(part2(&closing_overflow).unwrap(), i64::MAX.to_string());
        let path_overflow = Party {
            pair_happiness: vec![vec![0, i64::MAX, 1], vec![i64::MAX, 0, 1], vec![1, 1, 0]],
        };
        assert!(matches!(
            path_overflow.maximum_happiness(),
            Err(SolveError::ArithmeticOverflow)
        ));
    }

    #[test]
    fn oversized_tables_are_rejected_without_unbounded_fallback() {
        assert!(score_table_layout(17).is_ok());
        for guests in [0, 1, 18, usize::MAX] {
            assert!(score_table_layout(guests).is_err());
        }
        let party = Party {
            pair_happiness: vec![vec![0; 17]; 17],
        };
        assert!(party.with_neutral_guest().is_err());
    }

    #[test]
    fn test_part1_input() {
        scaffold_test_result(2015, 13, "input.txt", "input-spec.1.txt", part1);
    }

    #[test]
    fn test_part2_input() {
        scaffold_test_result(2015, 13, "input.txt", "input-spec.2.txt", part2);
    }
}
