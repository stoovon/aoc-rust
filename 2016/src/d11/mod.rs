use std::collections::{HashMap, HashSet, VecDeque};
use svutils::SolveError;

const TOP: u8 = 3;

#[derive(Clone, PartialEq, Eq, Hash)]
struct State {
    elevator: u8,
    pairs: Vec<[u8; 2]>,
}

impl State {
    fn safe(&self) -> bool {
        let mut generators = [false; 4];
        for &[generator, _] in &self.pairs {
            generators[usize::from(generator)] = true;
        }
        self.pairs
            .iter()
            .all(|&[generator, chip]| generator == chip || !generators[usize::from(chip)])
    }
}

fn parse(input: &str) -> Result<State, SolveError> {
    if input.trim().is_empty() {
        return Err(SolveError::EmptyInput);
    }
    let mut lines = input.trim().lines();
    let mut elements = HashMap::<&str, [Option<u8>; 2]>::new();
    for (floor, ordinal) in ["first", "second", "third", "fourth"]
        .into_iter()
        .enumerate()
    {
        let invalid =
            || SolveError::InvalidInput("invalid floor description".into()).at_line(floor + 1);
        let prefix = format!("The {ordinal} floor contains ");
        let contents = lines
            .next()
            .and_then(|line| line.strip_prefix(&prefix))
            .and_then(|line| line.strip_suffix('.'))
            .ok_or_else(invalid)?;
        if contents == "nothing relevant" {
            continue;
        }
        for item in contents.split(", ") {
            for item in item.strip_prefix("and ").unwrap_or(item).split(" and ") {
                let item = item
                    .strip_prefix("a ")
                    .or_else(|| item.strip_prefix("an "))
                    .ok_or_else(invalid)?;
                let (element, kind) = if let Some(name) = item.strip_suffix(" generator") {
                    (name, 0)
                } else if let Some(name) = item.strip_suffix("-compatible microchip") {
                    (name, 1)
                } else {
                    return Err(invalid());
                };
                if element.is_empty() || !element.bytes().all(|byte| byte.is_ascii_lowercase()) {
                    return Err(invalid());
                }
                if elements.entry(element).or_default()[kind]
                    .replace(floor as u8)
                    .is_some()
                {
                    return Err(
                        SolveError::InvalidInput("duplicate component".into()).at_line(floor + 1)
                    );
                }
            }
        }
    }
    if lines.next().is_some() {
        return Err(SolveError::InvalidInput("expected four floors".into()).at_line(5));
    }
    let mut pairs = Vec::with_capacity(elements.len());
    for (element, pair) in elements {
        let [Some(generator), Some(chip)] = pair else {
            return Err(SolveError::InvalidInput(format!(
                "missing partner for {element}"
            )));
        };
        pairs.push([generator, chip]);
    }
    Ok(State { elevator: 0, pairs })
}

fn shortest_route(mut start: State) -> Result<usize, SolveError> {
    if !start.safe() {
        return Err(SolveError::InvalidInput(
            "unsafe initial arrangement".into(),
        ));
    }
    start.pairs.sort_unstable();
    let mut seen = HashSet::from([start.clone()]);
    let mut queue = VecDeque::from([(start, 0)]);
    while let Some((state, distance)) = queue.pop_front() {
        if state.pairs.iter().all(|pair| *pair == [TOP; 2]) {
            return Ok(distance);
        }
        let cargo: Vec<_> = state
            .pairs
            .iter()
            .enumerate()
            .flat_map(|(pair, floors)| {
                floors
                    .iter()
                    .enumerate()
                    .filter(|&(_, &floor)| floor == state.elevator)
                    .map(move |(kind, _)| (pair, kind))
            })
            .collect();
        for next_floor in [
            state.elevator.checked_sub(1),
            (state.elevator < TOP).then_some(state.elevator + 1),
        ]
        .into_iter()
        .flatten()
        {
            for (index, &(pair, kind)) in cargo.iter().enumerate() {
                for &(other_pair, other_kind) in &cargo[index..] {
                    let mut next = state.clone();
                    next.elevator = next_floor;
                    next.pairs[pair][kind] = next_floor;
                    next.pairs[other_pair][other_kind] = next_floor;
                    if next.safe() {
                        next.pairs.sort_unstable();
                        if seen.insert(next.clone()) {
                            queue.push_back((next, distance + 1));
                        }
                    }
                }
            }
        }
    }
    Err(SolveError::NoSolution("no safe elevator route"))
}

pub fn part1(input: &str) -> Result<String, SolveError> {
    Ok(shortest_route(parse(input)?)?.to_string())
}

pub fn part2(input: &str) -> Result<String, SolveError> {
    let mut start = parse(input)?;
    start.pairs.extend([[0, 0]; 2]);
    Ok(shortest_route(start)?.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn safety_preserves_pair_identity() {
        assert!(State {
            elevator: 0,
            pairs: vec![[0, 2], [0, 3]]
        }
        .safe());
        assert!(State {
            elevator: 0,
            pairs: vec![[0, 0], [2, 2]]
        }
        .safe());
        assert!(!State {
            elevator: 0,
            pairs: vec![[0, 2], [2, 0]]
        }
        .safe());
    }

    #[test]
    fn finds_shortest_route_and_reports_stranded_elevator() {
        assert_eq!(
            shortest_route(State {
                elevator: 0,
                pairs: vec![[0, 1]]
            })
            .unwrap(),
            3
        );
        assert!(matches!(
            shortest_route(State {
                elevator: 0,
                pairs: vec![[1, 2]]
            }),
            Err(SolveError::NoSolution(_))
        ));
    }

    #[test]
    fn parses_a_pair_split_across_floors() {
        let input = concat!(
            "The first floor contains an argon generator.\r\n",
            "The second floor contains an argon-compatible microchip.\r\n",
            "The third floor contains nothing relevant.\r\n",
            "The fourth floor contains nothing relevant.",
        );
        assert_eq!(part1(input).unwrap(), "3");
        assert!(parse(&input.replace("argon-compatible", "xenon-compatible")).is_err());
    }
}
