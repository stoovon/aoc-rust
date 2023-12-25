use itertools::Itertools;

extern crate core;

const DIGITS: [(&str, i64); 20] = [
    ("0", 0),
    ("1", 1),
    ("2", 2),
    ("3", 3),
    ("4", 4),
    ("5", 5),
    ("6", 6),
    ("7", 7),
    ("8", 8),
    ("9", 9),
    ("zero", 0),
    ("one", 1),
    ("two", 2),
    ("three", 3),
    ("four", 4),
    ("five", 5),
    ("six", 6),
    ("seven", 7),
    ("eight", 8),
    ("nine", 9),
];

pub fn fn1(input: &str) -> i64 {

    let numbers: Vec<i64> = input.lines()
        .map(|l| {
            let numbers = l.chars().filter(|c| c.is_numeric()).collect_vec();
            let first = numbers[0];
            let last = numbers[numbers.len() - 1];
            let joined = first.to_string() + &last.to_string();

            joined.parse::<i64>().ok().unwrap_or_default()
        })
        .collect();


    numbers.iter().sum()
}

pub fn fn2(input: &[&str]) -> i64 {
    input
        .iter()
        .map(|line| {
            let mut line = *line;

            let first = 'outer: loop {
                for (digit, value) in DIGITS {
                    if line.starts_with(digit) {
                        break 'outer value;
                    }
                }
                line = &line[1..];
            };

            let second = 'outer: loop {
                for (digit, value) in DIGITS {
                    if line.ends_with(digit) {
                        break 'outer value;
                    }
                }
                line = &line[..line.len() - 1];
            };

            10 * first + second
        })
        .sum()
}

#[cfg(test)]
mod tests {
    use super::*;
    use svutils::load_spec;

    #[test]
    fn test_example() {
        assert_eq!(fn1(include_str!("../../../input/2023/d01/example.txt")), load_spec(include_str!("../../../input/2023/d01/example-spec.txt")));
    }

    #[test]
    fn test_fn1_test_case_name() {
        assert_eq!(fn1(include_str!("../../../input/2023/d01/input.txt")), load_spec(include_str!("../../../input/2023/d01/input-spec.1.txt")));
    }

    #[test]
    fn test_fn2_test_case_name() {
        assert_eq!(fn2(&include_str!("../../../input/2023/d01/input.txt").lines().collect_vec()), load_spec(include_str!("../../../input/2023/d01/input-spec.2.txt")));
    }
}