extern crate core;

pub fn parse(input: &str) -> (i64, i64) {
    let mut current = &mut Vec::new();
    let mut next = &mut Vec::new();
    let mut starts = Vec::new();
    let mut ends = Vec::new();

    let mut part_one = 0;
    let mut part_two = 0;

    for line in input.lines() {
        current.extend(
            line.split_whitespace()
                .map(|w| w.parse::<i64>().unwrap())
                .collect::<Vec<_>>(),
        );

        while current.iter().any(|&n| n != 0) {
            next.extend(current.windows(2).map(|w| w[1] - w[0]));
            starts.push(current[0]);
            ends.push(current[current.len() - 1]);

            (current, next) = (next, current);
            next.clear();
        }

        part_one += ends.iter().sum::<i64>();
        part_two += starts.iter().rev().fold(0, |acc, s| s - acc);

        current.clear();
        starts.clear();
        ends.clear();
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
    const DAY: i16 = 9;

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
