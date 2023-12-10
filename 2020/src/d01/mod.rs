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
    use svutils::load_spec;

    #[test]
    fn test_fn1_example() {
        assert_eq!(fn1(include_str!("example.txt")), load_spec(include_str!("example-spec.1.txt")));
    }

    #[test]
    fn test_fn1_input() {
        assert_eq!(fn1(include_str!("input.txt")), load_spec(include_str!("input-spec.1.txt")));
    }

    #[test]
    fn test_fn2_example() {
        assert_eq!(fn2(include_str!("example.txt")), load_spec(include_str!("example-spec.2.txt")));
    }

    #[test]
    fn test_fn2_input() {
        assert_eq!(fn2(include_str!("input.txt")), load_spec(include_str!("input-spec.2.txt")));
    }
}