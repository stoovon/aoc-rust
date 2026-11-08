use itertools::Itertools;
use svutils::SolveError;

const MAX_PACKAGES: usize = 63;
const MAX_SEARCH_WORK: usize = 10_000_000;

struct Search {
    remaining: usize,
}

impl Search {
    fn spend(&mut self) -> Result<(), SolveError> {
        self.remaining = self
            .remaining
            .checked_sub(1)
            .ok_or(SolveError::ResourceLimit(
                "package partition search budget exhausted",
            ))?;
        Ok(())
    }

    fn partition(
        &mut self,
        packages: &[u64],
        remaining: u64,
        groups: usize,
        target: u64,
    ) -> Result<bool, SolveError> {
        self.spend()?;
        if groups == 1 {
            return Ok(true);
        }
        let first = remaining.trailing_zeros() as usize;
        let bit = 1u64 << first;
        // Every partition has a group containing this package; fix it to avoid group permutations.
        self.complete_group(
            packages,
            remaining ^ bit,
            groups,
            target,
            first + 1,
            target - packages[first],
        )
    }

    fn complete_group(
        &mut self,
        packages: &[u64],
        remaining: u64,
        groups: usize,
        target: u64,
        start: usize,
        needed: u64,
    ) -> Result<bool, SolveError> {
        self.spend()?;
        if needed == 0 {
            return self.partition(packages, remaining, groups - 1, target);
        }
        for index in start..packages.len() {
            let bit = 1u64 << index;
            if remaining & bit != 0
                && packages[index] <= needed
                && self.complete_group(
                    packages,
                    remaining ^ bit,
                    groups,
                    target,
                    index + 1,
                    needed - packages[index],
                )?
            {
                return Ok(true);
            }
        }
        Ok(false)
    }
}

fn entanglement(packages: &[u64], groups: usize) -> Result<u64, SolveError> {
    if packages.is_empty() {
        return Err(SolveError::EmptyInput);
    }
    if packages.len() > MAX_PACKAGES {
        return Err(SolveError::ResourceLimit(
            "partition search supports at most 63 packages",
        ));
    }
    if packages.contains(&0) || groups < 2 {
        return Err(SolveError::InvalidInput(
            "expected positive weights and at least two groups".into(),
        ));
    }
    let total = packages.iter().try_fold(0u64, |sum, &weight| {
        sum.checked_add(weight)
            .ok_or(SolveError::ArithmeticOverflow)
    })?;
    if total % groups as u64 != 0 {
        return Err(SolveError::NoSolution(
            "total weight is not divisible by the group count",
        ));
    }
    let target = total / groups as u64;
    if packages.iter().any(|&weight| weight > target) {
        return Err(SolveError::NoSolution("a package exceeds the group weight"));
    }
    let all = (1u64 << packages.len()) - 1;
    let mut search = Search {
        remaining: MAX_SEARCH_WORK,
    };
    for size in 1..=packages.len() {
        let mut best = None;
        let mut overflowed = false;
        for indices in (0..packages.len()).combinations(size) {
            search.spend()?;
            let sum: u64 = indices.iter().map(|&index| packages[index]).sum();
            if sum != target {
                continue;
            }
            let mask = indices
                .iter()
                .fold(0, |mask, &index| mask | (1u64 << index));
            if !search.partition(packages, all ^ mask, groups - 1, target)? {
                continue;
            }
            let product = indices
                .iter()
                .try_fold(1u64, |product, &index| product.checked_mul(packages[index]));
            let Some(product) = product else {
                overflowed = true;
                continue;
            };
            best = Some(best.map_or(product, |current: u64| current.min(product)));
        }
        if let Some(best) = best {
            return Ok(best);
        }
        if overflowed {
            return Err(SolveError::ArithmeticOverflow);
        }
    }
    Err(SolveError::NoSolution(
        "packages cannot be partitioned into equal-weight groups",
    ))
}

fn parse(input: &str) -> Result<Vec<u64>, SolveError> {
    if input.trim().is_empty() {
        return Err(SolveError::EmptyInput);
    }
    let mut packages = Vec::new();
    for (index, line) in input.lines().enumerate() {
        let record = || -> Result<u64, SolveError> {
            let text = line.trim();
            if text.is_empty() || !text.bytes().all(|byte| byte.is_ascii_digit()) {
                return Err(SolveError::InvalidInput(
                    "expected a positive decimal package weight".into(),
                ));
            }
            let weight: u64 = text.parse()?;
            if weight == 0 {
                return Err(SolveError::InvalidInput(
                    "package weights must be positive".into(),
                ));
            }
            Ok(weight)
        };
        if packages.len() == MAX_PACKAGES {
            return Err(SolveError::ResourceLimit(
                "partition search supports at most 63 packages",
            ));
        }
        packages.push(record().map_err(|error| error.at_line(index + 1))?);
    }
    Ok(packages)
}

pub fn part1(input: &str) -> Result<String, SolveError> {
    let packages = parse(input)?;
    entanglement(&packages, 3).map(|product| product.to_string())
}

pub fn part2(input: &str) -> Result<String, SolveError> {
    let packages = parse(input)?;
    entanglement(&packages, 4).map(|product| product.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use svutils::scaffold_test_result;

    const YEAR: i16 = 2015;
    const DAY: i16 = 24;

    #[test]
    fn first_group_must_leave_a_partitionable_remainder() {
        let weights = [1, 2, 2, 2, 3, 6, 6, 8, 9];
        assert_eq!(entanglement(&weights, 3).unwrap(), 36);
        let mut search = Search {
            remaining: MAX_SEARCH_WORK,
        };
        let all = (1u64 << weights.len()) - 1;
        let bad = (1 << 0) | (1 << 4) | (1 << 8);
        assert!(!search.partition(&weights, all ^ bad, 2, 13).unwrap());
        let good = (1 << 0) | (1 << 5) | (1 << 6);
        assert!(search.partition(&weights, all ^ good, 2, 13).unwrap());
    }

    #[test]
    fn partition_search_budget_is_explicit() {
        let mut search = Search { remaining: 0 };
        assert!(matches!(
            search.partition(&[2, 2], 3, 2, 2),
            Err(SolveError::ResourceLimit(_))
        ));
    }

    fn exhaustive_entanglement(packages: &[u64], groups: usize) -> Option<u64> {
        let total: u64 = packages.iter().sum();
        if total % groups as u64 != 0 {
            return None;
        }
        let target = total / groups as u64;
        let mut best: Option<(usize, u64)> = None;
        for mut code in 0..groups.pow(packages.len() as u32) {
            let mut totals = vec![0; groups];
            let mut sizes = vec![0; groups];
            let mut products = vec![1; groups];
            for &weight in packages {
                let group = code % groups;
                code /= groups;
                totals[group] += weight;
                sizes[group] += 1;
                products[group] *= weight;
            }
            if totals.iter().all(|&weight| weight == target) {
                let candidate = (sizes[0], products[0]);
                best = Some(best.map_or(candidate, |current| current.min(candidate)));
            }
        }
        best.map(|(_, product)| product)
    }

    #[test]
    fn partition_search_matches_all_assignments_to_three_and_four_groups() {
        for length in 3..=7 {
            for seed in 0..24usize {
                let weights: Vec<_> = (0..length)
                    .map(|index| ((seed * 7 + index * 11 + index * seed) % 6 + 1) as u64)
                    .collect();
                for groups in [3, 4] {
                    let expected = exhaustive_entanglement(&weights, groups);
                    match (entanglement(&weights, groups), expected) {
                        (Ok(actual), Some(expected)) => assert_eq!(actual, expected,
                            "weights={weights:?}, groups={groups}"),
                        (Err(SolveError::NoSolution(_)), None) => (),
                        (actual, expected) => panic!(
                            "weights={weights:?}, groups={groups}, actual={actual:?}, expected={expected:?}"),
                    }
                }
            }
        }
    }

    #[test]
    fn overflowing_candidate_does_not_hide_a_representable_minimum() {
        let mut weights = [
            5_000_000_000,
            5_000_000_000,
            1,
            9_999_999_999,
            1,
            9_999_999_999,
        ];
        assert_eq!(entanglement(&weights, 3).unwrap(), 9_999_999_999);
        weights.reverse();
        assert_eq!(entanglement(&weights, 3).unwrap(), 9_999_999_999);
    }

    #[test]
    fn all_products_overflowing_at_the_minimum_size_return_an_error() {
        assert!(matches!(
            entanglement(&[5_000_000_000; 6], 3),
            Err(SolveError::ArithmeticOverflow)
        ));
    }

    #[test]
    fn parser_rejects_bad_weights_and_preserves_duplicates() {
        assert_eq!(parse(" 2 \r\n\t2\r\n7\r\n").unwrap(), [2, 2, 7]);
        for solve in [part1, part2] {
            assert!(matches!(solve(" \n"), Err(SolveError::EmptyInput)));
            for bad in [
                "",
                "0",
                "-2",
                "+2",
                "2.5",
                "2 3",
                "２",
                "18446744073709551616",
            ] {
                assert!(
                    matches!(
                        solve(&format!("2\n{bad}\n3")),
                        Err(SolveError::AtLine { line: 2, .. })
                    ),
                    "{bad:?}"
                );
            }
            assert!(matches!(
                solve(&"1\n".repeat(MAX_PACKAGES + 1)),
                Err(SolveError::ResourceLimit(_))
            ));
        }
    }

    #[test]
    fn totals_and_impossible_partitions_return_errors() {
        assert!(matches!(
            entanglement(&[u64::MAX, 1], 3),
            Err(SolveError::ArithmeticOverflow)
        ));
        for weights in [&[1, 1, 3][..], &[1, 1, 1, 9], &[2, 2, 2, 3]] {
            assert!(matches!(
                entanglement(weights, 3),
                Err(SolveError::NoSolution(_))
            ));
        }
        assert_eq!(entanglement(&[1; MAX_PACKAGES], MAX_PACKAGES).unwrap(), 1);
    }

    #[test]
    fn test_fn1_input() {
        scaffold_test_result(YEAR, DAY, "input.txt", "input-spec.1.txt", part1);
    }

    #[test]
    fn test_fn2_input() {
        scaffold_test_result(YEAR, DAY, "input.txt", "input-spec.2.txt", part2);
    }
}
