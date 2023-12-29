extern crate core;

use std::collections::HashSet;

pub fn fn1(input: &str) -> i64 {
    input
        .lines()
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
            break;
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
    use svutils::scaffold_test;

    const YEAR: i16 = 2018;
    const DAY: i16 = 1;

    #[test]
    #[ignore]
    fn test_fn1_example() {
        scaffold_test(YEAR, DAY, "example.1.txt", "example-spec.1.txt", fn1);
    }

    #[test]
    #[ignore]
    fn test_fn1_input() {
        scaffold_test(YEAR, DAY, "input.txt", "input-spec.1.txt", fn1);
    }

    #[test]
    #[ignore]
    fn test_fn2_example() {
        scaffold_test(YEAR, DAY, "example.2.txt", "example-spec.2.txt", fn2);
    }

    #[test]
    #[ignore]
    fn test_fn2_input() {
        scaffold_test(YEAR, DAY, "input.txt", "input-spec.2.txt", fn2);
    }
}
