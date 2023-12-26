extern crate core;

use itertools::Itertools;

pub fn fn1(input: &str) -> i64 {
    input
        .lines()
        .map(|v| v.parse::<i64>().ok())
        .into_iter()
        .tuple_combinations()
        .find(|(a, b)| a.unwrap_or_default() + b.unwrap_or_default() == 2020)
        .map_or_else(|| 0, |(a, b)| a.unwrap_or_default() * b.unwrap_or_default())
}

pub fn fn2(input: &str) -> i64 {
    input
        .lines()
        .map(|v| v.parse::<i64>().ok())
        .into_iter()
        .tuple_combinations()
        .find(|(a, b, c)| a.unwrap_or_default() + b.unwrap_or_default() + c.unwrap_or_default() == 2020)
        .map_or_else(|| 0, |(a, b, c)| a.unwrap_or_default() * b.unwrap_or_default() * c.unwrap_or_default())
}

#[cfg(test)]
mod tests {
    use super::*;
    use svutils::scaffold_test;

    const YEAR: i16 = 2020;
    const DAY: i16 = 1;

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
