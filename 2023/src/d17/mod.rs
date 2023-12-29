extern crate core;

use itertools::Itertools;
use std::collections::BinaryHeap;

fn parse_grid(input: &str) -> (Vec<usize>, usize, usize) {
    let input = input.trim();
    let cols = input.find('\n').unwrap();
    let vals = input
        .lines()
        .map(|l| l.chars().map(|c| c.to_digit(10).unwrap() as usize))
        .flatten()
        .collect_vec();
    // assert_eq!(vals.len() % cols, 0);
    let rows = vals.len() / cols;
    (vals, rows, cols)
}

#[derive(Hash, Eq, PartialEq, Clone, Copy, Debug)]
enum Direction {
    North,
    East,
    South,
    West,
}

use Direction::*;

impl Direction {
    fn reflect(&self) -> Self {
        match self {
            North => South,
            East => West,
            South => North,
            West => East,
        }
    }

    fn index(&self) -> usize {
        match self {
            North => 0,
            East => 1,
            South => 2,
            West => 3,
        }
    }
}

#[derive(PartialEq, Eq)]
struct Node {
    pos: usize,
    dir: Option<Direction>,
    distance: usize,
    cost: usize,
}

impl Ord for Node {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other.cost.cmp(&self.cost)
    }
}

impl PartialOrd for Node {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        other.cost.partial_cmp(&self.cost)
    }
}

fn solve(input: &str, min: usize, max: usize) -> i64 {
    let (tiles, rows, cols) = parse_grid(input);

    // Could use a Priority Queue for A* in Python, and probably in Rust
    let mut path = BinaryHeap::<Node>::new();

    let mut history = vec![(false, usize::MAX); tiles.len() * 4 * max];
    path.push(Node {
        pos: 0,
        dir: None,
        distance: 0,
        cost: 0,
    });
    while let Some(Node {
        pos,
        dir,
        distance,
        cost,
    }) = path.pop()
    {
        match dir {
            Some(d) => history[pos * 4 * max + d.index() * max + distance].0 = true,
            None => {
                for d in 0..4 {
                    history[pos * 4 * max + d * max + distance].0 = true;
                }
            }
        };
        path.extend([North, East, South, West].iter().filter_map(|&extend_dir| {
            let (same_dir, opp_dir) = match dir {
                Some(pdir) => (pdir == extend_dir, pdir.reflect() == extend_dir),
                None => (true, false),
            };
            if (distance < min && !same_dir)
                || (distance > max - 1 && same_dir) // too long in same direction
                || opp_dir // no backtracking.
                || match extend_dir { // Slick way of doing grid constraints
                    North => pos < cols,
                    East => pos % cols == cols - 1,
                    South => pos / cols == rows - 1,
                    West => pos % cols == 0,
                }
            {
                return None;
            }
            let next = match extend_dir {
                North => pos - cols,
                East => pos + 1,
                South => pos + cols,
                West => pos - 1,
            };
            let ndist = 1 + if same_dir { distance } else { 0 };
            let nkey = next * (4 * max) + extend_dir.index() * max + ndist;
            let ncost = cost + tiles[next];
            let (visited, prevcost) = history[nkey];
            if visited || prevcost <= ncost {
                return None;
            }
            history[nkey].1 = ncost;
            Some(Node {
                pos: next,
                dir: Some(extend_dir),
                distance: ndist,
                cost: ncost,
            })
        }));
    }
    // EDGE CASE: min cost of last tile.
    history[(tiles.len() - 1) * 4 * max..]
        .iter()
        .map(|(_visited, cost)| *cost)
        .min()
        .unwrap() as i64
}

pub fn fn1(input: &str) -> i64 {
    solve(input, 0, 3)
}

pub fn fn2(input: &str) -> i64 {
    solve(input, 4, 10)
}

#[cfg(test)]
mod tests {
    use super::*;
    use svutils::scaffold_test;

    const YEAR: i16 = 2023;
    const DAY: i16 = 17;

    #[test]
    fn test_fn1_example() {
        scaffold_test(YEAR, DAY, "example.txt", "example-spec.1.txt", fn1);
    }

    #[test]
    fn test_fn1_input() {
        scaffold_test(YEAR, DAY, "input.txt", "input-spec.1.txt", fn1);
    }

    #[test]
    fn test_fn2_example() {
        scaffold_test(YEAR, DAY, "example.txt", "example-spec.2.txt", fn2);
    }

    #[test]
    fn test_fn2_input() {
        scaffold_test(YEAR, DAY, "input.txt", "input-spec.2.txt", fn2);
    }
}
