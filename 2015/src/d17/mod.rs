extern crate core;

use itertools::Itertools;

pub fn solve(input: &str, seeking_capacity: i64) -> (i64, i64) {
    let containers: Vec<i64> = input.lines().map(|l| l.parse().unwrap()).collect();

    let mut count = 0;
    let mut part2 = 0;

    for i in 1..containers.len() {
        let c = containers.iter().combinations(i).filter(|c| c.iter().map(|&x| *x).sum::<i64>() == seeking_capacity).count();
        count += c as i64;
        if part2 == 0 && c > 0 {
            part2 = c as i64;
        }
    }

    (count, part2)
}

pub fn fn1(input: &str, seeking_capacity: i64) -> i64 {
    solve(input, seeking_capacity).0
}

pub fn fn2(input: &str, seeking_capacity: i64) -> i64 {
    solve(input, seeking_capacity).1
}

#[cfg(test)]
mod tests {
    use super::*;
    use svutils::scaffold_test;

    const YEAR: i16 = 2015;
    const DAY: i16 = 17;

    #[test]
    fn test_fn1_example() {
        scaffold_test(YEAR, DAY, "example.txt", "example-spec.1.txt", |input| { fn1(input, 25) });
    }

    #[test]
    fn test_fn1_input() {
        scaffold_test(YEAR, DAY, "input.txt", "input-spec.1.txt", |input| { fn1(input, 150) });
    }

    #[test]
    fn test_fn2_example() {
        scaffold_test(YEAR, DAY, "example.txt", "example-spec.2.txt", |input| { fn2(input, 25) });
    }

    #[test]
    fn test_fn2_input() {
        scaffold_test(YEAR, DAY, "input.txt", "input-spec.2.txt", |input| { fn2(input, 150) });
    }
}
