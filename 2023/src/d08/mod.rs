use std::collections::HashMap;
use svutils::SolveError;

#[derive(Clone, Copy)]
enum Direction {
    Left,
    Right,
}

struct Network<'a> {
    instructions: Vec<Direction>,
    names: Vec<&'a str>,
    ids: HashMap<&'a str, usize>,
    nodes: Vec<[usize; 2]>,
}

fn valid_name(name: &str) -> bool {
    name.len() == 3
        && name
            .bytes()
            .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit())
}

fn parse_network(input: &str) -> Result<Network<'_>, SolveError> {
    if input.trim().is_empty() {
        return Err(SolveError::EmptyInput);
    }
    let mut lines = input.lines();
    let instructions = lines
        .next()
        .ok_or(SolveError::EmptyInput)?
        .trim()
        .chars()
        .map(|c| match c {
            'L' => Ok(Direction::Left),
            'R' => Ok(Direction::Right),
            _ => Err(SolveError::InvalidInput(
                "instructions must contain only L and R".into(),
            )),
        })
        .collect::<Result<Vec<_>, _>>()?;
    if instructions.is_empty() {
        return Err(SolveError::InvalidInput(
            "instruction sequence is empty".into(),
        ));
    }
    let mut ids = HashMap::new();
    let mut definitions = Vec::new();
    for (idx, line) in lines.enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        let invalid =
            || SolveError::InvalidInput(format!("line {}: expected NAME = (LEFT, RIGHT)", idx + 2));
        let (name, edges) = line.split_once('=').ok_or_else(invalid)?;
        let name = name.trim();
        let edges = edges
            .trim()
            .strip_prefix('(')
            .and_then(|s| s.strip_suffix(')'))
            .ok_or_else(invalid)?;
        let (left, right) = edges.split_once(',').ok_or_else(invalid)?;
        let (left, right) = (left.trim(), right.trim());
        if ![name, left, right].into_iter().all(valid_name) {
            return Err(invalid());
        }
        if ids.insert(name, definitions.len()).is_some() {
            return Err(SolveError::InvalidInput(format!("duplicate node {name}")));
        }
        definitions.push((name, [left, right]));
    }
    if definitions.is_empty() {
        return Err(SolveError::InvalidInput("network contains no nodes".into()));
    }
    let mut names = Vec::with_capacity(definitions.len());
    let mut nodes = Vec::with_capacity(definitions.len());
    for (name, [left, right]) in definitions {
        let resolve = |target| {
            ids.get(target)
                .copied()
                .ok_or_else(|| SolveError::InvalidInput(format!("undefined node {target}")))
        };
        names.push(name);
        nodes.push([resolve(left)?, resolve(right)?]);
    }
    Ok(Network {
        instructions,
        names,
        ids,
        nodes,
    })
}

struct WalkCycle {
    cycle_start: i64,
    period: i64,
    // Target visits in the finite prefix and one complete cycle.
    target_steps: Vec<i64>,
}

impl<'a> Network<'a> {
    fn advance(&self, node: usize, instruction: usize) -> (usize, usize) {
        let edge = match self.instructions[instruction] {
            Direction::Left => 0,
            Direction::Right => 1,
        };
        (
            self.nodes[node][edge],
            (instruction + 1) % self.instructions.len(),
        )
    }

    fn analyze_cycle(
        &self,
        start: &'a str,
        is_target: impl Fn(&str) -> bool,
    ) -> Result<WalkCycle, SolveError> {
        let mut node = *self
            .ids
            .get(start)
            .ok_or_else(|| SolveError::InvalidInput(format!("missing start node {start}")))?;
        let state_count = self
            .nodes
            .len()
            .checked_mul(self.instructions.len())
            .ok_or(SolveError::ArithmeticOverflow)?;
        let mut instruction = 0;
        let mut steps = 0_i64;
        let mut first_seen = vec![-1_i64; state_count];
        let mut target_steps = Vec::new();
        loop {
            let state = node * self.instructions.len() + instruction;
            let cycle_start = first_seen[state];
            if cycle_start >= 0 {
                return Ok(WalkCycle {
                    cycle_start,
                    period: steps - cycle_start,
                    target_steps,
                });
            }
            first_seen[state] = steps;
            if is_target(self.names[node]) {
                target_steps.push(steps);
            }
            (node, instruction) = self.advance(node, instruction);
            steps = steps.checked_add(1).ok_or(SolveError::ArithmeticOverflow)?;
        }
    }

    fn first_hit(
        &self,
        start: &'a str,
        is_target: impl Fn(&str) -> bool,
    ) -> Result<i64, SolveError> {
        let mut node = *self
            .ids
            .get(start)
            .ok_or_else(|| SolveError::InvalidInput(format!("missing start node {start}")))?;
        let state_count = self
            .nodes
            .len()
            .checked_mul(self.instructions.len())
            .ok_or(SolveError::ArithmeticOverflow)?;
        let mut instruction = 0;
        let mut steps = 0_i64;
        let mut seen = vec![false; state_count];
        loop {
            if is_target(self.names[node]) {
                return Ok(steps);
            }
            let state = node * self.instructions.len() + instruction;
            if seen[state] {
                return Err(SolveError::InvalidInput(format!(
                    "target is unreachable from {start}"
                )));
            }
            seen[state] = true;
            (node, instruction) = self.advance(node, instruction);
            steps = steps.checked_add(1).ok_or(SolveError::ArithmeticOverflow)?;
        }
    }
}

pub fn part1(input: &str) -> Result<String, SolveError> {
    let network = parse_network(input)?;
    Ok(network.first_hit("AAA", |node| node == "ZZZ")?.to_string())
}

impl WalkCycle {
    fn hits_at(&self, step: i64) -> bool {
        let representative = if step < self.cycle_start {
            step
        } else {
            self.cycle_start + (step - self.cycle_start) % self.period
        };
        self.target_steps.binary_search(&representative).is_ok()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
// All solutions are residue + k * period; 0 <= residue < period.
struct Congruence {
    residue: i128,
    period: i128,
}

// Returns gcd(a, b) and the coefficient x in a*x + b*y = gcd(a, b).
fn extended_gcd(mut a: i128, mut b: i128) -> Result<(i128, i128), SolveError> {
    let (mut x, mut next_x) = (1_i128, 0_i128);
    while b != 0 {
        let quotient = a / b;
        (a, b) = (b, a % b);
        let updated = quotient
            .checked_mul(next_x)
            .and_then(|product| x.checked_sub(product))
            .ok_or(SolveError::ArithmeticOverflow)?;
        (x, next_x) = (next_x, updated);
    }
    Ok((a, x))
}

impl Congruence {
    fn combine(self, other: Self) -> Result<Option<Self>, SolveError> {
        let (gcd, inverse) = extended_gcd(self.period, other.period)?;
        let difference = other.residue - self.residue;
        if difference % gcd != 0 {
            return Ok(None);
        }
        // Divide out shared factors, then solve for the number of our periods to add.
        let reduced = other.period / gcd;
        let multiplier = (difference / gcd)
            .rem_euclid(reduced)
            .checked_mul(inverse.rem_euclid(reduced))
            .ok_or(SolveError::ArithmeticOverflow)?
            % reduced;
        let period = self
            .period
            .checked_mul(reduced)
            .ok_or(SolveError::ArithmeticOverflow)?;
        let residue = self
            .period
            .checked_mul(multiplier)
            .and_then(|offset| self.residue.checked_add(offset))
            .ok_or(SolveError::ArithmeticOverflow)?
            % period;
        Ok(Some(Self { residue, period }))
    }

    fn first_at_or_after(self, minimum: i128) -> Result<i128, SolveError> {
        if self.residue >= minimum {
            return Ok(self.residue);
        }
        let difference = minimum - self.residue;
        let periods = difference / self.period + i128::from(difference % self.period != 0);
        self.period
            .checked_mul(periods)
            .and_then(|offset| self.residue.checked_add(offset))
            .ok_or(SolveError::ArithmeticOverflow)
    }
}

fn synchronize(profiles: &[WalkCycle]) -> Result<i64, SolveError> {
    let no_solution =
        || SolveError::InvalidInput("walkers never reach targets simultaneously".into());
    let latest_cycle = profiles
        .iter()
        .max_by_key(|profile| profile.cycle_start)
        .ok_or_else(|| {
            SolveError::InvalidInput("network has no starting nodes ending in A".into())
        })?;
    let periodic_start = latest_cycle.cycle_start;

    // Any earlier solution must be one of this walker's transient target hits.
    for &step in latest_cycle
        .target_steps
        .iter()
        .take_while(|&&step| step < periodic_start)
    {
        if profiles.iter().all(|profile| profile.hits_at(step)) {
            return Ok(step);
        }
    }

    let mut candidates = vec![Congruence {
        residue: 0,
        period: 1,
    }];
    for profile in profiles {
        let mut combined = Vec::new();
        for &candidate in &candidates {
            for &step in profile
                .target_steps
                .iter()
                .filter(|&&step| step >= profile.cycle_start)
            {
                let target = Congruence {
                    residue: i128::from(step % profile.period),
                    period: i128::from(profile.period),
                };
                if let Some(intersection) = candidate.combine(target)? {
                    combined.push(intersection);
                }
            }
        }
        if combined.is_empty() {
            return Err(no_solution());
        }
        combined.sort_unstable();
        combined.dedup();
        candidates = combined;
    }
    let mut earliest = i128::MAX;
    for candidate in candidates {
        earliest = earliest.min(candidate.first_at_or_after(i128::from(periodic_start))?);
    }
    i64::try_from(earliest).map_err(|_| SolveError::ArithmeticOverflow)
}

pub fn part2(input: &str) -> Result<String, SolveError> {
    let network = parse_network(input)?;
    let mut starts: Vec<_> = network
        .names
        .iter()
        .copied()
        .filter(|name| name.ends_with('A'))
        .collect();
    starts.sort_unstable();
    let profiles = starts
        .into_iter()
        .map(|start| network.analyze_cycle(start, |node| node.ends_with('Z')))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(synchronize(&profiles)?.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use svutils::scaffold_test_result;

    const YEAR: i16 = 2023;
    const DAY: i16 = 8;

    #[test]
    fn generalized_crt_handles_offsets_shared_factors_and_incompatibility() {
        let a = Congruence {
            residue: 2,
            period: 3,
        };
        let b = Congruence {
            residue: 3,
            period: 4,
        };
        assert_eq!(
            a.combine(b).unwrap(),
            Some(Congruence {
                residue: 11,
                period: 12
            })
        );
        let a = Congruence {
            residue: 2,
            period: 6,
        };
        assert_eq!(
            a.combine(Congruence {
                residue: 8,
                period: 9
            })
            .unwrap(),
            Some(Congruence {
                residue: 8,
                period: 18
            })
        );
        assert_eq!(
            a.combine(Congruence {
                residue: 3,
                period: 9
            })
            .unwrap(),
            None
        );
    }

    #[test]
    fn synchronization_handles_transients_multiple_hits_and_cycle_entry_bounds() {
        let a = WalkCycle {
            cycle_start: 3,
            period: 3,
            target_steps: vec![1, 5],
        };
        let b = WalkCycle {
            cycle_start: 4,
            period: 4,
            target_steps: vec![1, 7],
        };
        assert_eq!(synchronize(&[a, b]).unwrap(), 1);
        let a = WalkCycle {
            cycle_start: 0,
            period: 3,
            target_steps: vec![2],
        };
        let b = WalkCycle {
            cycle_start: 0,
            period: 4,
            target_steps: vec![0, 3],
        };
        assert_eq!(synchronize(&[a, b]).unwrap(), 8);
        let a = WalkCycle {
            cycle_start: 10,
            period: 3,
            target_steps: vec![11],
        };
        let b = WalkCycle {
            cycle_start: 0,
            period: 4,
            target_steps: vec![0],
        };
        assert_eq!(synchronize(&[a, b]).unwrap(), 20);
    }

    #[test]
    fn impossible_synchronization_and_missing_starts_return_errors() {
        let a = WalkCycle {
            cycle_start: 0,
            period: 2,
            target_steps: vec![0],
        };
        let b = WalkCycle {
            cycle_start: 0,
            period: 2,
            target_steps: vec![1],
        };
        assert!(synchronize(&[a, b]).is_err());
        assert!(part2("L\n\nBBB = (BBB, BBB)").is_err());
        assert!(part2("L\n\nAAA = (AAA, AAA)").is_err());
    }

    #[test]
    fn synchronization_honours_offsets_in_actual_networks() {
        let input = "L\n\nAAA = (A01, A01)\nA01 = (AAZ, AAZ)\nAAZ = (AAA, AAA)\nBBA = (B01, B01)\nB01 = (B02, B02)\nB02 = (BBZ, BBZ)\nBBZ = (BBA, BBA)";
        assert_eq!(part2(input).unwrap(), "11");
    }

    #[test]
    fn crt_arithmetic_reports_unrepresentable_periods() {
        let a = Congruence {
            residue: 0,
            period: i128::MAX,
        };
        let b = Congruence {
            residue: 0,
            period: 2,
        };
        assert!(matches!(a.combine(b), Err(SolveError::ArithmeticOverflow)));
    }

    #[test]
    fn cycle_analysis_distinguishes_transient_hits_from_repeating_hits() {
        let input = "L\n\nAAA = (ABZ, ABZ)\nABZ = (BBB, BBB)\nBBB = (CCZ, CCZ)\nCCZ = (DDD, DDD)\nDDD = (CCZ, CCZ)";
        let network = parse_network(input).unwrap();
        let cycle = network
            .analyze_cycle("AAA", |node| node.ends_with('Z'))
            .unwrap();
        assert_eq!(cycle.cycle_start, 3);
        assert_eq!(cycle.period, 2);
        assert_eq!(cycle.target_steps, vec![1, 3]);
    }

    #[test]
    fn a_cycle_can_have_multiple_target_positions_or_none() {
        let input = "L\n\nAAA = (ABZ, ABZ)\nABZ = (ACZ, ACZ)\nACZ = (AAA, AAA)";
        let network = parse_network(input).unwrap();
        let cycle = network
            .analyze_cycle("AAA", |node| node.ends_with('Z'))
            .unwrap();
        assert_eq!(cycle.cycle_start, 0);
        assert_eq!(cycle.period, 3);
        assert_eq!(cycle.target_steps, vec![1, 2]);
        let cycle = network.analyze_cycle("AAA", |node| node == "ZZZ").unwrap();
        assert!(cycle.target_steps.is_empty());
    }

    #[test]
    fn cycle_analysis_includes_the_instruction_phase() {
        let network = parse_network("LR\n\nAAA = (AAA, AAA)").unwrap();
        let cycle = network.analyze_cycle("AAA", |_| false).unwrap();
        assert_eq!(cycle.cycle_start, 0);
        assert_eq!(cycle.period, 2);
    }

    #[test]
    fn part_one_follows_instructions_instead_of_shortest_graph_path() {
        let input = "R\n\nAAA = (ZZZ, BBB)\nBBB = (ZZZ, ZZZ)\nZZZ = (ZZZ, ZZZ)";
        assert_eq!(part1(input).unwrap(), "2");
    }

    #[test]
    fn revisiting_a_node_at_a_different_instruction_is_not_a_cycle() {
        let input = "LR\n\nAAA = (AAA, ZZZ)\nZZZ = (ZZZ, ZZZ)";
        assert_eq!(part1(input).unwrap(), "2");
    }

    #[test]
    fn unreachable_targets_return_errors_and_part_one_requires_exact_zzz() {
        let input = "L\n\nAAA = (AAZ, ZZZ)\nAAZ = (AAZ, AAZ)\nZZZ = (ZZZ, ZZZ)";
        assert!(part1(input)
            .unwrap_err()
            .to_string()
            .contains("unreachable"));
        assert!(part1("L\n\nBBB = (BBB, BBB)").is_err());
    }

    #[test]
    fn malformed_networks_return_errors_from_both_parts() {
        for input in [
            "",
            " ",
            "X\n\nAAA = (AAA, AAA)",
            "L",
            "L\n\nAAA = AAA, AAA",
            "L\n\nAAAA = (AAA, AAA)",
            "L\n\nAAA = (BBB, AAA)",
            "L\n\nAAA = (AAA, AAA)\nAAA = (AAA, AAA)",
            "L\n\nAAA = (AAA, AAA, AAA)",
        ] {
            for solver in [part1, part2] {
                assert!(solver(input).is_err(), "{input}");
            }
        }
    }

    #[test]
    fn network_spacing_and_windows_line_endings_are_supported() {
        let input = "R\r\n\r\n AAA=(AAA, ZZZ) \r\nZZZ = (ZZZ,ZZZ)\r\n";
        assert_eq!(part1(input).unwrap(), "1");
    }

    #[test]
    fn test_part1_input() {
        scaffold_test_result(YEAR, DAY, "input.txt", "input-spec.1.txt", part1);
    }

    #[test]
    fn test_part2_input() {
        scaffold_test_result(YEAR, DAY, "input.txt", "input-spec.2.txt", part2);
    }
}
