use std::collections::HashMap;

use svutils::SolveError;

const MAX_CITIES: usize = 18;

fn city_index<'a>(
    cities: &mut HashMap<&'a str, usize>,
    name: &'a str,
) -> Result<usize, SolveError> {
    if let Some(&index) = cities.get(name) {
        return Ok(index);
    }
    let index = cities.len();
    if index == MAX_CITIES {
        return Err(SolveError::ResourceLimit(
            "route search supports at most 18 cities",
        ));
    }
    cities.insert(name, index);
    Ok(index)
}

#[derive(Clone, Copy)]
enum Objective {
    Shortest,
    Longest,
}

impl Objective {
    fn improves(self, candidate: u64, current: u64) -> bool {
        match self {
            Self::Shortest => candidate < current,
            Self::Longest => candidate > current,
        }
    }
}

struct Graph {
    distances: Vec<Vec<u64>>,
}

impl Graph {
    fn parse(input: &str) -> Result<Self, SolveError> {
        if input.trim().is_empty() {
            return Err(SolveError::EmptyInput);
        }
        let mut cities = HashMap::new();
        let mut routes = [[None; MAX_CITIES]; MAX_CITIES];
        for (line_number, line) in input.lines().enumerate() {
            let mut record = || -> Result<(), SolveError> {
                let mut tokens = line.split_ascii_whitespace();
                let (Some(from), Some("to"), Some(to), Some("="), Some(distance), None) = (
                    tokens.next(),
                    tokens.next(),
                    tokens.next(),
                    tokens.next(),
                    tokens.next(),
                    tokens.next(),
                ) else {
                    return Err(SolveError::InvalidInput(
                        "expected 'city to city = distance'".into(),
                    ));
                };
                if !from.bytes().all(|byte| byte.is_ascii_alphabetic())
                    || !to.bytes().all(|byte| byte.is_ascii_alphabetic())
                    || from == to
                {
                    return Err(SolveError::InvalidInput(
                        "expected distinct ASCII alphabetic city names".into(),
                    ));
                }
                if !distance.bytes().all(|byte| byte.is_ascii_digit()) {
                    return Err(SolveError::InvalidInput(
                        "expected a nonnegative decimal distance".into(),
                    ));
                }
                let distance = distance.parse::<u64>()?;
                let from = city_index(&mut cities, from)?;
                let to = city_index(&mut cities, to)?;
                if routes[from][to].is_some() {
                    return Err(SolveError::InvalidInput(
                        "duplicate undirected route".into(),
                    ));
                }
                routes[from][to] = Some(distance);
                routes[to][from] = Some(distance);
                Ok(())
            };
            record().map_err(|error| error.at_line(line_number + 1))?;
        }
        let count = cities.len();
        let mut distances = vec![vec![0; count]; count];
        for from in 0..count {
            for to in 0..count {
                if from != to {
                    distances[from][to] = routes[from][to].ok_or_else(|| {
                        SolveError::InvalidInput("missing route between cities".into())
                    })?;
                }
            }
        }
        Ok(Self { distances })
    }

    fn best_route(&self, objective: Objective) -> Result<u64, SolveError> {
        let count = self.distances.len();
        if count == 0 {
            return Err(SolveError::EmptyInput);
        }
        if count > MAX_CITIES {
            return Err(SolveError::ResourceLimit(
                "route search supports at most 18 cities",
            ));
        }
        let states = 1usize << count;
        let mut costs = vec![None; states * count];
        for city in 0..count {
            costs[(1 << city) * count + city] = Some(0u64);
        }
        for mask in 1..states {
            for last in 0..count {
                let Some(cost) = costs[mask * count + last] else {
                    continue;
                };
                for next in 0..count {
                    let bit = 1 << next;
                    if mask & bit != 0 {
                        continue;
                    }
                    let candidate = cost
                        .checked_add(self.distances[last][next])
                        .ok_or(SolveError::ArithmeticOverflow)?;
                    let slot = &mut costs[(mask | bit) * count + next];
                    if slot.map_or(true, |current| objective.improves(candidate, current)) {
                        *slot = Some(candidate);
                    }
                }
            }
        }
        costs[(states - 1) * count..]
            .iter()
            .flatten()
            .copied()
            .reduce(|best, candidate| {
                if objective.improves(candidate, best) {
                    candidate
                } else {
                    best
                }
            })
            .ok_or_else(|| SolveError::InvalidInput("no route visits every city".into()))
    }
}

fn solve(input: &str, objective: Objective) -> Result<String, SolveError> {
    Graph::parse(input)?
        .best_route(objective)
        .map(|cost| cost.to_string())
}

pub fn part1(input: &str) -> Result<String, SolveError> {
    solve(input, Objective::Shortest)
}

pub fn part2(input: &str) -> Result<String, SolveError> {
    solve(input, Objective::Longest)
}

#[cfg(test)]
mod tests {
    use super::*;
    use itertools::Itertools;
    use svutils::scaffold_test_result;

    const YEAR: i16 = 2015;
    const DAY: i16 = 9;

    #[test]
    fn route_search_matches_permutation_oracle() {
        for count in 2..=7 {
            for seed in 0..8 {
                let mut distances = vec![vec![0u64; count]; count];
                for a in 0..count {
                    for b in a + 1..count {
                        let weight = ((a * 17 + b * 11 + seed * 7) % 23) as u64;
                        distances[a][b] = weight;
                        distances[b][a] = weight;
                    }
                }
                let graph = Graph { distances };
                let totals: Vec<u64> = (0..count)
                    .permutations(count)
                    .map(|route| {
                        route
                            .windows(2)
                            .map(|pair| graph.distances[pair[0]][pair[1]])
                            .sum()
                    })
                    .collect();
                assert_eq!(
                    graph.best_route(Objective::Shortest).unwrap(),
                    *totals.iter().min().unwrap()
                );
                assert_eq!(
                    graph.best_route(Objective::Longest).unwrap(),
                    *totals.iter().max().unwrap()
                );
            }
        }
        let zero = Graph {
            distances: vec![vec![0; 3]; 3],
        };
        assert_eq!(zero.best_route(Objective::Longest).unwrap(), 0);
        let overflow = Graph {
            distances: vec![vec![u64::MAX; 3]; 3],
        };
        assert!(matches!(
            overflow.best_route(Objective::Shortest),
            Err(SolveError::ArithmeticOverflow)
        ));
        let oversized = Graph {
            distances: vec![vec![]; 19],
        };
        assert!(matches!(
            oversized.best_route(Objective::Shortest),
            Err(SolveError::ResourceLimit(_))
        ));
    }

    #[test]
    fn parser_checks_records_and_complete_graph() {
        let input = "Birch to Alder = 4\r\nCedar to Birch = 9\r\nAlder to Cedar = 2\r\n";
        assert_eq!(part1(input).unwrap(), "6");
        assert_eq!(part2(input).unwrap(), "13");
        assert_eq!(part1(" Alder\tto Birch = 0 ").unwrap(), "0");
        assert_eq!(
            part2("Alder to Birch = 18446744073709551615").unwrap(),
            u64::MAX.to_string()
        );
        for solver in [part1, part2] {
            for empty in ["", " \r\n"] {
                assert!(matches!(solver(empty), Err(SolveError::EmptyInput)));
            }
            for bad in [
                "",
                "Alder Birch 7",
                "Alder by Birch = 7",
                "Alder to Birch : 7",
                "Alder to Birch = 7 extra",
                "Alder to Birch = -1",
                "Alder to Birch = +1",
                "Alder to Birch = 18446744073709551616",
                "Alder to Alder = 7",
                "Ålder to Birch = 7",
                "A1 to Birch = 7",
                "Alder to Birch = 4",
                "Birch to Alder = 4",
                "Birch to Alder = 5",
            ] {
                let input = format!("Alder to Birch = 4\n{bad}\n");
                assert!(
                    solver(&input)
                        .unwrap_err()
                        .to_string()
                        .starts_with("line 2:"),
                    "{bad:?}"
                );
            }
            assert!(solver("Alder to Birch = 4\nBirch to Cedar = 3").is_err());
            let mut input = String::new();
            for letter in b'B'..=b'S' {
                input.push_str(&format!("A to {} = 1\n", char::from(letter)));
            }
            assert!(solver(&input)
                .unwrap_err()
                .to_string()
                .contains("resource limit"));
        }
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
