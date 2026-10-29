use std::collections::{HashSet, VecDeque};
use std::rc::Rc;
use svutils::SolveError;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct Rule<'a> {
    from: &'a str,
    to: &'a str,
}

fn valid_molecule(text: &str) -> bool {
    !text.is_empty() && text.bytes().all(|byte| byte.is_ascii_alphanumeric())
}

struct Problem<'a> {
    rules: Vec<Rule<'a>>,
    molecule: &'a str,
}

impl<'a> Problem<'a> {
    fn parse(input: &'a str) -> Result<Self, SolveError> {
        if input.trim().is_empty() {
            return Err(SolveError::EmptyInput);
        }
        let mut rules = Vec::new();
        let mut unique = HashSet::new();
        let mut separated = false;
        let mut molecule = None;
        for (index, line) in input.lines().enumerate() {
            let invalid =
                |message: &str| SolveError::InvalidInput(message.into()).at_line(index + 1);
            if !separated {
                if line.is_empty() {
                    if rules.is_empty() {
                        return Err(invalid("expected at least one replacement rule"));
                    }
                    separated = true;
                    continue;
                }
                let (from, to) = line
                    .split_once(" => ")
                    .ok_or_else(|| invalid("expected source => replacement"))?;
                if !valid_molecule(from) || !valid_molecule(to) {
                    return Err(invalid(
                        "rule sides must be nonempty ASCII alphanumeric strings",
                    ));
                }
                let rule = Rule { from, to };
                if !unique.insert(rule) {
                    return Err(invalid("duplicate replacement rule"));
                }
                rules.push(rule);
            } else {
                if molecule.is_some() {
                    return Err(invalid("unexpected data after the target molecule"));
                }
                if !valid_molecule(line) {
                    return Err(invalid(
                        "expected one nonempty ASCII alphanumeric target molecule",
                    ));
                }
                molecule = Some(line);
            }
        }
        let molecule = molecule.ok_or_else(|| {
            SolveError::InvalidInput("expected a blank line followed by one target molecule".into())
        })?;
        Ok(Self { rules, molecule })
    }

    fn one_step_molecules(&self) -> Result<HashSet<String>, SolveError> {
        let mut molecules = HashSet::new();
        for rule in &self.rules {
            for offset in occurrences(self.molecule, rule.from) {
                molecules.insert(replace_at(self.molecule, offset, rule.from, rule.to)?);
            }
        }
        Ok(molecules)
    }

    fn minimum_steps(&self, limits: SearchLimits) -> Result<i64, SolveError> {
        if self.molecule == "e" {
            return Ok(0);
        }
        if !self.rules.iter().any(|rule| rule.from == "e") {
            return Err(SolveError::InvalidInput(
                "target cannot be derived from e".into(),
            ));
        }
        let mut reverse: Vec<_> = self.rules.iter().collect();
        reverse.sort_unstable_by(|a, b| {
            b.to.len()
                .cmp(&a.to.len())
                .then_with(|| a.to.cmp(b.to))
                .then_with(|| a.from.cmp(b.from))
        });
        let prediction = CharacterCounter::derive(&self.rules)
            .and_then(|counter| counter.predict(self.molecule));
        let required = match prediction {
            Some(CounterPrediction::Impossible) => {
                return Err(SolveError::InvalidInput(
                    "target cannot be derived from e".into(),
                ))
            }
            Some(CounterPrediction::Steps(steps)) => Some(steps),
            None => None,
        };
        // A counter fixes the length, but a real path must still prove reachability.
        if let Some(required) = required {
            if let Some(steps) = greedy_witness(self.molecule, &reverse, required, limits) {
                if steps == required {
                    return i64::try_from(steps).map_err(|_| SolveError::ArithmeticOverflow);
                }
            }
        }
        exact_reverse_search(self.molecule, &reverse, required, limits)
    }
}

fn occurrences<'a>(molecule: &'a str, pattern: &'a str) -> impl Iterator<Item = usize> + 'a {
    let mut start = 0usize;
    std::iter::from_fn(move || {
        let offset = start.checked_add(molecule.get(start..)?.find(pattern)?)?;
        // Advancing one byte retains overlaps; validated molecules are ASCII.
        start = offset.checked_add(1)?;
        Some(offset)
    })
}

fn replacement_len(molecule: &str, from: &str, to: &str) -> Result<usize, SolveError> {
    molecule
        .len()
        .checked_sub(from.len())
        .and_then(|length| length.checked_add(to.len()))
        .ok_or(SolveError::ArithmeticOverflow)
}

fn replace_at(molecule: &str, offset: usize, from: &str, to: &str) -> Result<String, SolveError> {
    let length = replacement_len(molecule, from, to)?;
    let end = offset
        .checked_add(from.len())
        .ok_or(SolveError::ArithmeticOverflow)?;
    let mut result = String::new();
    result
        .try_reserve_exact(length)
        .map_err(|_| SolveError::InvalidInput("replacement molecule is too large".into()))?;
    result.push_str(&molecule[..offset]);
    result.push_str(to);
    result.push_str(&molecule[end..]);
    Ok(result)
}

fn gcd(a: i128, b: i128) -> Option<i128> {
    let (mut a, mut b) = (a.checked_abs()?, b.checked_abs()?);
    while b != 0 {
        (a, b) = (b, a % b);
    }
    Some(a)
}

fn normalize_row(row: &mut [i128]) -> Option<()> {
    let divisor = row
        .iter()
        .try_fold(0i128, |divisor, &value| gcd(divisor, value))?;
    if divisor > 1 {
        for value in row.iter_mut() {
            *value /= divisor;
        }
    }
    if row
        .iter()
        .find(|&&value| value != 0)
        .is_some_and(|&value| value < 0)
    {
        for value in row {
            *value = value.checked_neg()?;
        }
    }
    Some(())
}

// Additive weights give a rule the same increment in every surrounding context.
// One fixed positive increment forces the length of every successful derivation.
struct CharacterCounter {
    weights: [i128; 128],
    increment: i128,
}

enum CounterPrediction {
    Steps(usize),
    Impossible,
}

impl CharacterCounter {
    fn derive(rules: &[Rule<'_>]) -> Option<Self> {
        if rules.len() > 256 {
            return None;
        }
        let mut present = [false; 128];
        present[b'e' as usize] = true;
        for rule in rules {
            for byte in rule.from.bytes().chain(rule.to.bytes()) {
                *present.get_mut(byte as usize)? = true;
            }
        }
        let alphabet: Vec<_> = present
            .iter()
            .enumerate()
            .filter_map(|(byte, &used)| used.then_some(byte))
            .collect();
        if alphabet.len() > 64 {
            return None;
        }
        let mut columns = [0; 128];
        for (column, &byte) in alphabet.iter().enumerate() {
            columns[byte] = column;
        }
        let variables = alphabet.len();
        let mut rows = Vec::new();
        for rule in rules {
            let mut row = vec![0i128; variables + 1];
            for byte in rule.from.bytes() {
                let value = &mut row[columns[byte as usize]];
                *value = value.checked_sub(1)?;
            }
            for byte in rule.to.bytes() {
                let value = &mut row[columns[byte as usize]];
                *value = value.checked_add(1)?;
            }
            row[variables] = 1;
            rows.push(row);
        }
        let mut pivots = Vec::new();
        // Exact elimination keeps integer rows; overflow declines this optional proof.
        for column in 0..variables {
            let rank = pivots.len();
            let Some(pivot) = (rank..rows.len()).find(|&row| rows[row][column] != 0) else {
                continue;
            };
            rows.swap(rank, pivot);
            normalize_row(&mut rows[rank])?;
            let pivot_row = rows[rank].clone();
            for (index, row) in rows.iter_mut().enumerate() {
                if index == rank || row[column] == 0 {
                    continue;
                }
                let factor = row[column];
                for (value, &pivot_value) in row.iter_mut().zip(&pivot_row) {
                    *value = value
                        .checked_mul(pivot_row[column])?
                        .checked_sub(factor.checked_mul(pivot_value)?)?;
                }
                normalize_row(row)?;
            }
            pivots.push(column);
        }
        if rows
            .iter()
            .any(|row| row[..variables].iter().all(|&value| value == 0) && row[variables] != 0)
        {
            return None;
        }
        let mut increment = 1i128;
        for (row, &column) in pivots.iter().enumerate() {
            let divisor = rows[row][column];
            increment = (increment / gcd(increment, divisor)?).checked_mul(divisor)?;
        }
        let mut weights = [0i128; 128];
        // Free weights are zero; a common denominator makes all weights integral.
        for (row, &column) in pivots.iter().enumerate() {
            weights[alphabet[column]] =
                rows[row][variables].checked_mul(increment / rows[row][column])?;
        }
        let counter = Self { weights, increment };
        counter.verifies(rules).then_some(counter)
    }

    fn total(&self, molecule: &str) -> Option<i128> {
        molecule.bytes().try_fold(0i128, |total, byte| {
            total.checked_add(*self.weights.get(byte as usize)?)
        })
    }

    fn verifies(&self, rules: &[Rule<'_>]) -> bool {
        self.increment > 0
            && rules.iter().all(|rule| {
                self.total(rule.to)
                    .zip(self.total(rule.from))
                    .and_then(|(to, from)| to.checked_sub(from))
                    == Some(self.increment)
            })
    }

    fn predict(&self, molecule: &str) -> Option<CounterPrediction> {
        let difference = self.total(molecule)?.checked_sub(self.total("e")?)?;
        if difference < 0 || difference % self.increment != 0 {
            return Some(CounterPrediction::Impossible);
        }
        usize::try_from(difference / self.increment)
            .ok()
            .map(CounterPrediction::Steps)
    }
}

#[derive(Clone, Copy)]
struct SearchLimits {
    states: usize,
    bytes: usize,
    checks: usize,
}

impl Default for SearchLimits {
    fn default() -> Self {
        Self {
            states: 65_536,
            bytes: 32 * 1024 * 1024,
            checks: 10_000_000,
        }
    }
}

fn offset_checks(molecule: &str, pattern: &str) -> usize {
    molecule
        .len()
        .checked_sub(pattern.len())
        .map_or(0, |last| last + 1)
}

fn greedy_witness(
    target: &str,
    reverse: &[&Rule<'_>],
    required: usize,
    limits: SearchLimits,
) -> Option<usize> {
    if target.len() > limits.bytes {
        return None;
    }
    let mut molecule = target.to_owned();
    let mut checks = 0usize;
    let mut steps = 0usize;
    loop {
        if molecule == "e" {
            return Some(steps);
        }
        if steps == required {
            return None;
        }
        let mut changed = false;
        for rule in reverse {
            checks = checks.checked_add(offset_checks(&molecule, rule.to))?;
            if checks > limits.checks {
                return None;
            }
            let offset = occurrences(&molecule, rule.to).next();
            if let Some(offset) = offset {
                if replacement_len(&molecule, rule.to, rule.from).ok()? > limits.bytes {
                    return None;
                }
                molecule = replace_at(&molecule, offset, rule.to, rule.from).ok()?;
                steps = steps.checked_add(1)?;
                changed = true;
                if molecule == "e" {
                    return Some(steps);
                }
                if steps == required {
                    return None;
                }
            }
        }
        if !changed {
            return None;
        }
    }
}

fn exact_reverse_search(
    target: &str,
    reverse: &[&Rule<'_>],
    required: Option<usize>,
    limits: SearchLimits,
) -> Result<i64, SolveError> {
    let unsupported =
        || SolveError::InvalidInput("molecule search exceeds its resource budget".into());
    if limits.states == 0 || target.len() > limits.bytes {
        return Err(unsupported());
    }
    let start: Rc<str> = Rc::from(target);
    let mut seen = HashSet::from([Rc::clone(&start)]);
    let mut queue = VecDeque::from([(start, 0usize)]);
    let mut bytes = target.len();
    let mut checks = 0usize;
    while let Some((molecule, steps)) = queue.pop_front() {
        if required.is_some_and(|required| steps >= required) {
            continue;
        }
        for rule in reverse {
            checks = checks
                .checked_add(offset_checks(&molecule, rule.to))
                .ok_or_else(unsupported)?;
            if checks > limits.checks {
                return Err(unsupported());
            }
            for offset in occurrences(&molecule, rule.to) {
                if replacement_len(&molecule, rule.to, rule.from)? > limits.bytes {
                    return Err(unsupported());
                }
                let next = replace_at(&molecule, offset, rule.to, rule.from)?;
                let next_steps = steps.checked_add(1).ok_or(SolveError::ArithmeticOverflow)?;
                if next == "e" {
                    return i64::try_from(next_steps).map_err(|_| SolveError::ArithmeticOverflow);
                }
                if seen.contains(next.as_str()) {
                    continue;
                }
                let retained = bytes.checked_add(next.len()).ok_or_else(unsupported)?;
                if seen.len() >= limits.states || retained > limits.bytes {
                    return Err(unsupported());
                }
                bytes = retained;
                let next: Rc<str> = next.into();
                seen.insert(Rc::clone(&next));
                queue.push_back((next, next_steps));
            }
        }
    }
    Err(SolveError::InvalidInput(
        "target cannot be derived from e".into(),
    ))
}

pub fn part1(input: &str) -> Result<String, SolveError> {
    let count = Problem::parse(input)?.one_step_molecules()?.len();
    i64::try_from(count)
        .map(|count| count.to_string())
        .map_err(|_| SolveError::ArithmeticOverflow)
}

pub fn part2(input: &str) -> Result<String, SolveError> {
    Problem::parse(input)?
        .minimum_steps(SearchLimits::default())
        .map(|steps| steps.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use svutils::scaffold_test_result;

    #[test]
    fn synthetic_rewrites_and_zero_steps() {
        let rules = "e => Q\nQ => QR\nR => SS";
        assert_eq!(part1(&format!("{rules}\n\nQR")).unwrap(), "2");
        assert_eq!(part1(&format!("{rules}\n\nQRR")).unwrap(), "3");
        assert_eq!(part2(&format!("{rules}\n\nQR")).unwrap(), "2");
        assert_eq!(part2(&format!("{rules}\n\nQRR")).unwrap(), "3");
        assert_eq!(part2("A => A\n\ne").unwrap(), "0");
        let problem = Problem::parse("e => A\n\ne").unwrap();
        assert_eq!(
            problem
                .minimum_steps(SearchLimits {
                    states: 0,
                    bytes: 0,
                    checks: 0
                })
                .unwrap(),
            0
        );
    }

    #[test]
    fn all_rules_and_overlapping_positions_are_preserved() {
        let problem = Problem::parse("Q => QR\nR => QR\n\nQR").unwrap();
        assert_eq!(
            problem.one_step_molecules().unwrap(),
            HashSet::from(["QRR".to_owned(), "QQR".to_owned()])
        );
        let problem = Problem::parse("QQ => R\n\nQQQ").unwrap();
        assert_eq!(
            problem.one_step_molecules().unwrap(),
            HashSet::from(["RQ".to_owned(), "QR".to_owned()])
        );
        assert_eq!(part1("A => AA\n\nAA").unwrap(), "1");
        assert_eq!(part1("A => A\n\nA").unwrap(), "1");
    }

    #[test]
    fn replacements_use_literal_character_contexts() {
        let problem = Problem::parse("Q => XYZ\n\nQ7R").unwrap();
        assert_eq!(
            problem.one_step_molecules().unwrap(),
            HashSet::from(["XYZ7R".to_owned()])
        );
        let problem = Problem::parse("Q => X\n\nQe").unwrap();
        assert_eq!(
            problem.one_step_molecules().unwrap(),
            HashSet::from(["Xe".to_owned()])
        );
        let input = "e => Q\ne => R\nQ => Qe\n\nQe";
        assert_eq!(part1(input).unwrap(), "3");
        assert_eq!(part2(&input.replace("\n\nQe", "\n\nQQ")).unwrap(), "3");
    }

    #[test]
    fn stuck_greedy_candidates_fall_back_without_returning_counts() {
        let input = "e => C\nC => AB\nA => AB\n\nAB";
        let problem = Problem::parse(input).unwrap();
        let reverse = [&problem.rules[2], &problem.rules[1], &problem.rules[0]];
        assert_eq!(
            greedy_witness(problem.molecule, &reverse, 2, SearchLimits::default()),
            None
        );
        assert_eq!(part2(input).unwrap(), "2");
        for _ in 0..10 {
            assert_eq!(part2("e => A\nA => AB\n\nAB").unwrap(), "2");
            assert_eq!(part2("A => AB\ne => A\n\nAB").unwrap(), "2");
        }
    }

    #[test]
    fn variable_length_paths_use_exact_search() {
        let problem = Problem::parse("e => A\ne => B\nB => A\n\nA").unwrap();
        assert!(CharacterCounter::derive(&problem.rules).is_none());
        assert_eq!(problem.minimum_steps(SearchLimits::default()).unwrap(), 1);
        assert_eq!(part2("e => A\nA => A\n\nA").unwrap(), "1");
    }

    #[test]
    fn character_counter_is_exact_and_does_not_prove_reachability() {
        let problem = Problem::parse("e => AA\n\nAA").unwrap();
        let counter = CharacterCounter::derive(&problem.rules).unwrap();
        assert!(counter.verifies(&problem.rules));
        assert!(matches!(
            counter.predict("AA"),
            Some(CounterPrediction::Steps(1))
        ));
        assert!(matches!(
            counter.predict("A"),
            Some(CounterPrediction::Impossible)
        ));
        assert_eq!(part2("e => AA\n\nAA").unwrap(), "1");
        assert!(part2("e => AA\n\nA")
            .unwrap_err()
            .to_string()
            .contains("cannot be derived"));
        let fractional = Problem::parse("e => AA\ne => BBB\n\nBBB").unwrap();
        let counter = CharacterCounter::derive(&fractional.rules).unwrap();
        assert!(counter.verifies(&fractional.rules));
        assert!(matches!(
            counter.predict("AA"),
            Some(CounterPrediction::Steps(1))
        ));
        assert!(matches!(
            counter.predict("BBB"),
            Some(CounterPrediction::Steps(1))
        ));
        assert!(matches!(
            counter.predict("BB"),
            Some(CounterPrediction::Impossible)
        ));
        let unreachable = Problem::parse("A => AA\n\nAA").unwrap();
        let counter = CharacterCounter::derive(&unreachable.rules).unwrap();
        assert!(matches!(
            counter.predict("AA"),
            Some(CounterPrediction::Steps(_))
        ));
        assert!(unreachable
            .minimum_steps(SearchLimits::default())
            .unwrap_err()
            .to_string()
            .contains("cannot be derived"));
    }

    #[test]
    fn invalid_counter_and_checked_elimination_cannot_supply_a_proof() {
        let problem = Problem::parse("e => A\nA => AB\n\nAB").unwrap();
        let mut counter = CharacterCounter::derive(&problem.rules).unwrap();
        counter.weights[b'A' as usize] += 1;
        assert!(!counter.verifies(&problem.rules));
        counter.weights[b'A' as usize] = i128::MAX;
        assert_eq!(counter.total("AA"), None);
        assert!(!counter.verifies(&problem.rules));
        counter.increment = 0;
        assert!(!counter.verifies(&problem.rules));
        let mut row = [-6, 4, -2];
        normalize_row(&mut row).unwrap();
        assert_eq!(row, [3, -2, 1]);
        assert!(normalize_row(&mut [i128::MIN, 1]).is_none());
        let mut input = String::new();
        for index in 0..257 {
            input.push_str(&format!("e => A{index:03}\n"));
        }
        input.push_str("\nA000");
        let problem = Problem::parse(&input).unwrap();
        assert!(CharacterCounter::derive(&problem.rules).is_none());
        assert_eq!(problem.minimum_steps(SearchLimits::default()).unwrap(), 1);
    }

    fn forward_reference(problem: &Problem<'_>) -> Option<i64> {
        let mut seen = HashSet::from(["e".to_owned()]);
        let mut queue = VecDeque::from([("e".to_owned(), 0i64)]);
        while let Some((molecule, steps)) = queue.pop_front() {
            if molecule == problem.molecule {
                return Some(steps);
            }
            for rule in &problem.rules {
                for offset in 0..molecule.len() {
                    let end = offset + rule.from.len();
                    if molecule.get(offset..end) != Some(rule.from) {
                        continue;
                    }
                    let next = format!("{}{}{}", &molecule[..offset], rule.to, &molecule[end..]);
                    // These generated forward rules never shrink, so this bound is complete.
                    if next.len() <= problem.molecule.len() && seen.insert(next.clone()) {
                        queue.push_back((next, steps + 1));
                    }
                }
            }
        }
        None
    }

    #[test]
    fn minimum_steps_match_complete_forward_graphs() {
        let pool = [
            Rule { from: "e", to: "A" },
            Rule { from: "e", to: "B" },
            Rule { from: "A", to: "B" },
            Rule { from: "B", to: "A" },
            Rule {
                from: "A",
                to: "AA",
            },
            Rule {
                from: "A",
                to: "AB",
            },
            Rule {
                from: "B",
                to: "BA",
            },
            Rule {
                from: "B",
                to: "BB",
            },
        ];
        let targets = [
            "A", "B", "AA", "AB", "BA", "BB", "AAA", "AAB", "ABA", "ABB", "BAA", "BAB", "BBA",
            "BBB",
        ];
        let limits = SearchLimits {
            states: 128,
            bytes: 4096,
            checks: 10_000,
        };
        let mut cases = 0;
        for mask in 1..1 << pool.len() {
            let rules: Vec<_> = pool
                .iter()
                .enumerate()
                .filter_map(|(index, rule)| (mask & (1 << index) != 0).then_some(*rule))
                .collect();
            for target in targets {
                let problem = Problem {
                    rules: rules.clone(),
                    molecule: target,
                };
                let expected = forward_reference(&problem);
                let actual = problem.minimum_steps(limits);
                match expected {
                    Some(steps) => {
                        assert_eq!(actual.unwrap(), steps, "rules={mask}, target={target}")
                    }
                    None => assert!(
                        actual
                            .unwrap_err()
                            .to_string()
                            .contains("cannot be derived"),
                        "rules={mask}, target={target}"
                    ),
                }
                cases += 1;
            }
        }
        assert_eq!(cases, 3570);
    }

    #[test]
    fn cycles_and_expanding_reverse_rules_respect_budgets() {
        let problem = Problem::parse("e => A\nBB => B\nB => B\n\nB").unwrap();
        let limits = SearchLimits {
            states: 8,
            bytes: 128,
            checks: 1000,
        };
        assert!(problem
            .minimum_steps(limits)
            .unwrap_err()
            .to_string()
            .contains("budget"));
        let problem = Problem::parse("e => A\nA => A\n\nA").unwrap();
        for limits in [
            SearchLimits {
                states: 0,
                bytes: 128,
                checks: 1000,
            },
            SearchLimits {
                states: 8,
                bytes: 0,
                checks: 1000,
            },
            SearchLimits {
                states: 8,
                bytes: 128,
                checks: 0,
            },
        ] {
            assert!(problem
                .minimum_steps(limits)
                .unwrap_err()
                .to_string()
                .contains("budget"));
        }
    }

    #[test]
    fn parser_validates_sections_rules_and_molecules() {
        for input in ["", " \n\t"] {
            assert!(matches!(Problem::parse(input), Err(SolveError::EmptyInput)));
        }
        for input in [
            "\nA",
            "e => A",
            "e => A\n\n",
            "e => A\n\nA\nB",
            "e => A\n\nA\n\n",
            "e => A\n\n A",
            "e => A\n\nÅ",
            " => A\n\nA",
            "e => \n\nA",
            "e->A\n\nA",
            "e => A => B\n\nA",
            "e => A\ne => A\n\nA",
            "e => A\n\n\nA",
        ] {
            assert!(Problem::parse(input).is_err(), "{input:?}");
        }
        let input = "e => A\ninvalid\n\nA";
        for solver in [part1, part2] {
            assert!(solver(input)
                .unwrap_err()
                .to_string()
                .starts_with("line 2:"));
        }
        assert_eq!(part2("e => A\r\nA => AB\r\n\r\nAB\r\n").unwrap(), "2");
    }

    #[test]
    fn test_part1_input() {
        scaffold_test_result(2015, 19, "input.txt", "input-spec.1.txt", part1);
    }

    #[test]
    fn test_part2_input() {
        scaffold_test_result(2015, 19, "input.txt", "input-spec.2.txt", part2);
    }
}
