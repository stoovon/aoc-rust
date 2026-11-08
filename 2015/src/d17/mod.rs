use svutils::SolveError;

const TARGET: usize = 150;
const MAX_CONTAINERS: usize = 256;
const MAX_TABLE_CELLS: usize = 1_000_000;
const MAX_UPDATES: usize = 20_000_000;

fn parse(input: &str) -> Result<Vec<u64>, SolveError> {
    if input.trim().is_empty() {
        return Err(SolveError::EmptyInput);
    }
    let mut containers = Vec::new();
    for (index, line) in input.lines().enumerate() {
        let capacity = || -> Result<u64, SolveError> {
            if containers.len() == MAX_CONTAINERS {
                return Err(SolveError::ResourceLimit("too many containers"));
            }
            let digits = line.trim();
            if digits.is_empty() || !digits.bytes().all(|byte| byte.is_ascii_digit()) {
                return Err(SolveError::InvalidInput(
                    "expected a nonnegative decimal capacity".into(),
                ));
            }
            Ok(digits.parse()?)
        };
        containers.push(capacity().map_err(|error| error.at_line(index + 1))?);
    }
    Ok(containers)
}

struct Combinations {
    ways: Vec<u64>,
    width: usize,
}

impl Combinations {
    fn count(containers: &[u64], target: usize) -> Result<Self, SolveError> {
        if containers.len() > MAX_CONTAINERS {
            return Err(SolveError::ResourceLimit("too many containers"));
        }
        let width = containers.len() + 1;
        let cells = target
            .checked_add(1)
            .and_then(|rows| rows.checked_mul(width))
            .filter(|&cells| cells <= MAX_TABLE_CELLS)
            .ok_or(SolveError::ResourceLimit(
                "container combination table budget exceeded",
            ))?;
        let updates = cells
            .checked_mul(containers.len())
            .ok_or(SolveError::ResourceLimit(
                "container combination work budget exceeded",
            ))?;
        if updates > MAX_UPDATES {
            return Err(SolveError::ResourceLimit(
                "container combination work budget exceeded",
            ));
        }
        let mut ways = Vec::new();
        ways.try_reserve_exact(cells).map_err(|_| {
            SolveError::ResourceLimit("could not allocate container combination table")
        })?;
        ways.resize(cells, 0u64);
        ways[0] = 1;
        for (index, &capacity) in containers.iter().enumerate() {
            let Ok(capacity) = usize::try_from(capacity) else {
                continue;
            };
            if capacity > target {
                continue;
            }
            for volume in (0..=target - capacity).rev() {
                // Descending size also prevents reuse of a zero-capacity container.
                for used in (0..=index).rev() {
                    let source = ways[volume * width + used];
                    let destination = &mut ways[(volume + capacity) * width + used + 1];
                    *destination = destination
                        .checked_add(source)
                        .ok_or(SolveError::ArithmeticOverflow)?;
                }
            }
        }
        Ok(Self { ways, width })
    }

    fn target_counts(&self) -> &[u64] {
        &self.ways[self.ways.len() - self.width..]
    }

    fn total(&self) -> Result<u64, SolveError> {
        self.target_counts().iter().try_fold(0u64, |total, &count| {
            total
                .checked_add(count)
                .ok_or(SolveError::ArithmeticOverflow)
        })
    }

    fn fewest_container_ways(&self) -> u64 {
        self.target_counts()
            .iter()
            .copied()
            .find(|&count| count != 0)
            .unwrap_or(0)
    }
}

pub fn part1(input: &str) -> Result<String, SolveError> {
    Combinations::count(&parse(input)?, TARGET)?
        .total()
        .map(|total| total.to_string())
}

pub fn part2(input: &str) -> Result<String, SolveError> {
    Ok(Combinations::count(&parse(input)?, TARGET)?
        .fewest_container_ways()
        .to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use svutils::scaffold_test_result;

    fn exhaustive_counts(containers: &[u64], target: usize) -> Vec<u64> {
        let mut counts = vec![0; containers.len() + 1];
        for mask in 0..1usize << containers.len() {
            let mut total = 0u128;
            let mut used = 0;
            for (index, &capacity) in containers.iter().enumerate() {
                if mask & (1 << index) != 0 {
                    total += u128::from(capacity);
                    used += 1;
                }
            }
            if total == target as u128 {
                counts[used] += 1;
            }
        }
        counts
    }

    #[test]
    fn generated_counts_match_all_physical_subsets() {
        for length in 0..=6 {
            for mut code in 0..3usize.pow(length) {
                let mut containers = Vec::new();
                for _ in 0..length {
                    containers.push([0, 2, 5][code % 3]);
                    code /= 3;
                }
                for target in 0..=12 {
                    let expected = exhaustive_counts(&containers, target);
                    let actual = Combinations::count(&containers, target).unwrap();
                    assert_eq!(actual.target_counts(), expected);
                    assert_eq!(actual.total().unwrap(), expected.iter().sum::<u64>());
                    assert_eq!(
                        actual.fewest_container_ways(),
                        expected.into_iter().find(|&count| count != 0).unwrap_or(0)
                    );
                }
            }
        }
    }

    #[test]
    fn full_set_duplicates_empty_selection_and_no_reuse() {
        for (containers, target, total, minimum) in [
            (vec![4, 7, 9], 20, 1, 1),
            (vec![17], 17, 1, 1),
            (vec![4, 4], 4, 2, 2),
            (vec![4, 4], 8, 1, 1),
            (vec![4], 8, 0, 0),
            (vec![3, 6, 9], 9, 2, 1),
            (vec![0, 0, 4], 4, 4, 1),
            (vec![0, 0, 4], 0, 4, 1),
            (vec![u64::MAX, 4], 4, 1, 1),
        ] {
            let combinations = Combinations::count(&containers, target).unwrap();
            assert_eq!(combinations.total().unwrap(), total);
            assert_eq!(combinations.fewest_container_ways(), minimum);
        }
        assert_eq!(part1("150").unwrap(), "1");
        assert_eq!(part2("150").unwrap(), "1");
        assert_eq!(part1("151").unwrap(), "0");
        assert_eq!(part2("151").unwrap(), "0");
    }

    #[test]
    fn parser_preserves_separate_containers_and_reports_bad_lines() {
        assert_eq!(parse(" 4 \r\n\t4\r\n0\r\n").unwrap(), vec![4, 4, 0]);
        for solver in [part1, part2] {
            for empty in ["", " \r\n"] {
                assert!(matches!(solver(empty), Err(SolveError::EmptyInput)));
            }
            for bad in ["", "-4", "+4", "4.0", "4 7", "18446744073709551616", "４"] {
                let input = format!("4\n{bad}\n");
                assert!(
                    solver(&input)
                        .unwrap_err()
                        .to_string()
                        .starts_with("line 2:"),
                    "{bad:?}"
                );
            }
            let input = "1\n".repeat(MAX_CONTAINERS + 1);
            assert!(solver(&input)
                .unwrap_err()
                .to_string()
                .contains("too many containers"));
        }
    }

    #[test]
    fn count_overflow_and_resource_budgets_are_explicit() {
        assert!(matches!(
            Combinations::count(&[1; 68], 34),
            Err(SolveError::ArithmeticOverflow)
        ));
        let combinations = Combinations::count(&[0; 64], 0).unwrap();
        assert!(matches!(
            combinations.total(),
            Err(SolveError::ArithmeticOverflow)
        ));
        assert_eq!(combinations.fewest_container_ways(), 1);
        assert!(matches!(
            Combinations::count(&[1], usize::MAX),
            Err(SolveError::ResourceLimit(_))
        ));
        assert!(matches!(
            Combinations::count(&[1], MAX_TABLE_CELLS),
            Err(SolveError::ResourceLimit(_))
        ));
        assert!(matches!(
            Combinations::count(&[1; MAX_CONTAINERS], 1000),
            Err(SolveError::ResourceLimit(_))
        ));
    }

    #[test]
    fn test_part1_input() {
        scaffold_test_result(2015, 17, "input.txt", "input-spec.1.txt", part1);
    }
    #[test]
    fn test_part2_input() {
        scaffold_test_result(2015, 17, "input.txt", "input-spec.2.txt", part2);
    }
}
