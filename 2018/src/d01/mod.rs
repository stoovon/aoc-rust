extern crate core;

use std::collections::HashSet;

pub fn fn1(input: &str) -> i64 {
    input.lines()
        .map(|v| v.parse::<i64>().ok().unwrap_or_default())
        .sum()
}

pub fn fn2(input: &str) -> i64 {
    // Cycle detection using a hash. We could probably do better.

    let mut total = 0;
    let mut cache = HashSet::new();

	let lines: Vec<i64> = input
        .lines()
        .map(|v| v.parse::<i64>().ok().unwrap_or_default())
        .collect();

    let mut offset = 0;

    loop {
        // Note that the lists repeat UNTIL a number is seen twice.
        total += lines[offset];

        if !cache.insert(total) {
            break
        }

        offset += 1;

        if offset > lines.len() - 1 {
            offset = 0
        }
    }

    total
}

#[cfg(test)]
mod tests {
    use super::*;
    use svutils::load_spec;

    #[test]
    fn test_fn1_example() {
        assert_eq!(fn1(include_str!("example.1.txt")), load_spec(include_str!("example-spec.1.txt")));
    }

    #[test]
    fn test_fn1_input() {
        assert_eq!(fn1(include_str!("input.txt")), load_spec(include_str!("input-spec.1.txt")));
    }

    #[test]
    fn test_fn2_example() {
        assert_eq!(fn2(include_str!("example.2.txt")), load_spec(include_str!("example-spec.2.txt")));
    }

    #[test]
    fn test_fn2_input() {
        assert_eq!(fn2(&include_str!("input.txt")), load_spec(include_str!("input-spec.2.txt")));
    }
}