extern crate core;

use std::collections::HashMap;

fn solve(input: &str) -> (String, String) {
    let input: Vec<Vec<char>> = input
        .lines()
        .map(|line| line.chars().collect::<Vec<_>>())
        .collect();

    let columns = input[0].len();
    let mut part1 = Vec::new();
    let mut part2 = Vec::new();

    for i in 0..columns {
        let mut hist = HashMap::new();
        for line in &input {
            *hist.entry(line[i]).or_insert(0) += 1;
        }

        part1.push(
            hist.iter()
                .max_by_key(|&(_ch, freq)| freq)
                .unwrap()
                .0
                .clone(),
        );
        part2.push(
            hist.iter()
                .min_by_key(|&(_ch, freq)| freq)
                .unwrap()
                .0
                .clone(),
        );
    }

    (
        part1.into_iter().collect::<String>(),
        part2.into_iter().collect::<String>(),
    )
}

pub fn fn1(input: &str) -> String {
    solve(input).0
}

pub fn fn2(input: &str) -> String {
    solve(input).1
}

#[cfg(test)]
mod tests {
    use super::*;
    use svutils::scaffold_test_string;

    const YEAR: i16 = 2016;
    const DAY: i16 = 6;

    #[test]
    fn test_fn1_example() {
        scaffold_test_string(YEAR, DAY, "example.txt", "example-spec.1.txt", fn1);
    }

    #[test]
    fn test_fn1_input() {
        scaffold_test_string(YEAR, DAY, "input.txt", "input-spec.1.txt", fn1);
    }

    #[test]
    fn test_fn2_example() {
        scaffold_test_string(YEAR, DAY, "example.txt", "example-spec.2.txt", fn2);
    }

    #[test]
    fn test_fn2_input() {
        scaffold_test_string(YEAR, DAY, "input.txt", "input-spec.2.txt", fn2);
    }
}
