extern crate core;

use std::collections::HashMap;
use std::collections::HashSet;
use std::collections::VecDeque;

fn gcd(first: i64, second: i64) -> i64 {
    let mut a = first;
    let mut b = second;

    while b != 0 {
        (a, b) = (b, a % b);
    }

    a
}

fn lcm(first: i64, second: i64) -> i64 {
    let a = first;
    let b = second;

    a * (b / gcd(a, b))
}

fn parse(input: &str) -> (i64, i64) {
    let lines: Vec<_> = input.lines().collect();
    let mut nodes = HashMap::with_capacity(lines.len());

    for line in &lines[2..] {
        nodes.insert(&line[0..3], [&line[7..10], &line[12..15]]);
    }

    let mut part_one = lines[0].len() as i64;
    let mut part_two = lines[0].len() as i64;
    let mut todo = VecDeque::new();
    let mut seen = HashSet::new();

    for &start in nodes.keys().filter(|k| k.ends_with('A')) {
        // Find the length of the cycle using a BFS from each start node.
        todo.push_back((start, 0));
        seen.insert(start);

        while let Some((node, cost)) = todo.pop_front() {
            if node.ends_with('Z') {
                if start == "AAA" {
                    part_one = lcm(part_one, cost);
                }
                part_two = lcm(part_two, cost);
                break;
            }

            for next in nodes[node] {
                if seen.insert(next) {
                    todo.push_back((next, cost + 1));
                }
            }
        }

        todo.clear();
        seen.clear();
    }

    (part_one, part_two)
}

pub fn fn1(input: &str) -> i64 {
    parse(input).0
}

pub fn fn2(input: &str) -> i64 {
    parse(input).1
}

#[cfg(test)]
mod tests {
    use super::*;
    use svutils::scaffold_test;

    const YEAR: i16 = 2023;
    const DAY: i16 = 8;

    #[test]
    fn test_fn1_example() {
        scaffold_test(YEAR, DAY, "example.1.txt", "example-spec.1.1.txt", fn1);
        scaffold_test(YEAR, DAY, "example.2.txt", "example-spec.1.2.txt", fn1);
    }

    #[test]
    fn test_fn1_input() {
        scaffold_test(YEAR, DAY, "input.txt", "input-spec.1.txt", fn1);
    }

    #[test]
    fn test_fn2_example() {
        scaffold_test(YEAR, DAY, "example.3.txt", "example-spec.2.txt", fn2);
    }

    #[test]
    fn test_fn2_input() {
        scaffold_test(YEAR, DAY, "input.txt", "input-spec.2.txt", fn2);
    }
}
