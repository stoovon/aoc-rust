use std::cmp::Reverse;
use std::collections::{BinaryHeap, HashMap, HashSet, VecDeque};
use svutils::SolveError;

type Point = (u32, u32);
const START: Point = (1, 1);
const GOAL: Point = (31, 39);
const SEARCH_LIMIT: usize = 100_000;

fn favorite(input: &str) -> Result<u64, SolveError> {
    let input = input.trim();
    if input.is_empty() {
        return Err(SolveError::EmptyInput);
    }
    Ok(input.parse()?)
}

fn is_open((x, y): Point, favorite: u64) -> bool {
    let (x, y) = (u128::from(x), u128::from(y));
    let value = (x + y).pow(2) + 3 * x + y + u128::from(favorite);
    value.count_ones() % 2 == 0
}

fn neighbors((x, y): Point, favorite: u64) -> impl Iterator<Item = Point> {
    [
        x.checked_add(1).map(|x| (x, y)),
        x.checked_sub(1).map(|x| (x, y)),
        y.checked_add(1).map(|y| (x, y)),
        y.checked_sub(1).map(|y| (x, y)),
    ]
    .into_iter()
    .flatten()
    .filter(move |&point| is_open(point, favorite))
}

fn shortest_route(favorite: u64, goal: Point) -> Result<u64, SolveError> {
    let distance = |(x, y): Point| u64::from(x.abs_diff(goal.0)) + u64::from(y.abs_diff(goal.1));
    if !is_open(START, favorite) || !is_open(goal, favorite) {
        return Err(SolveError::NoSolution("start or target is a wall"));
    }
    let mut best = HashMap::from([(START, 0)]);
    let mut frontier = BinaryHeap::from([Reverse((distance(START), 0, START))]);
    let mut expanded = 0;
    while let Some(Reverse((_, moves, point))) = frontier.pop() {
        if best[&point] != moves {
            continue;
        }
        if point == goal {
            return Ok(moves);
        }
        if expanded == SEARCH_LIMIT {
            return Err(SolveError::ResourceLimit(
                "maze search exceeded 100000 expansions",
            ));
        }
        expanded += 1;
        let next_moves = moves + 1;
        for next in neighbors(point, favorite) {
            if best.get(&next).is_none_or(|&old| next_moves < old) {
                best.insert(next, next_moves);
                frontier.push(Reverse((next_moves + distance(next), next_moves, next)));
            }
        }
    }
    Err(SolveError::NoSolution("target is unreachable"))
}

fn reachable(favorite: u64, limit: u32) -> Result<usize, SolveError> {
    if !is_open(START, favorite) {
        return Err(SolveError::NoSolution("start is a wall"));
    }
    let mut seen = HashSet::from([START]);
    let mut queue = VecDeque::from([(START, 0)]);
    while let Some((point, moves)) = queue.pop_front() {
        if moves < limit {
            for next in neighbors(point, favorite) {
                if seen.insert(next) {
                    queue.push_back((next, moves + 1));
                }
            }
        }
    }
    Ok(seen.len())
}

pub fn part1(input: &str) -> Result<String, SolveError> {
    Ok(shortest_route(favorite(input)?, GOAL)?.to_string())
}

pub fn part2(input: &str) -> Result<String, SolveError> {
    Ok(reachable(favorite(input)?, 50)?.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn searches_a_synthetic_maze() {
        assert_eq!(shortest_route(2, (2, 0)).unwrap(), 2);
        assert_eq!(reachable(2, 0).unwrap(), 1);
        assert_eq!(reachable(2, 1).unwrap(), 3);
    }

    #[test]
    fn reports_walls_and_invalid_input() {
        assert!(matches!(
            shortest_route(2, (0, 1)),
            Err(SolveError::NoSolution(_))
        ));
        assert!(matches!(part2(""), Err(SolveError::EmptyInput)));
        assert!(matches!(part1("-1"), Err(SolveError::InvalidNumber(_))));
    }
}
