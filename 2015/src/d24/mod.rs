extern crate core;

use itertools::Itertools;

fn entanglement(packages: Vec<i64>, divisor: i64) -> Option<i64> {
    let target = packages.iter().sum::<i64>() / divisor;

    let mut done = false;
    let mut qe = i64::MAX;
    for n in 1..packages.len() {
        for combo in packages.iter().combinations(n) {
            let sum: i64 = combo.iter().map(|x| *x).sum();
            if sum == target {
                qe = qe.min(combo.iter().map(|x| *x).product());
                done = true;
            }
        }
        if done {
            return Some(qe);
        }
    }

    None
}

pub fn fn1(input: &str) -> i64 {
    let packages: Vec<i64> = input.lines().map(|line| line.parse().unwrap()).collect();
    entanglement(packages, 3).unwrap()
}

pub fn fn2(input: &str) -> i64 {
    let packages: Vec<i64> = input.lines().map(|line| line.parse().unwrap()).collect();
    entanglement(packages, 4).unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;
    use svutils::scaffold_test;

    const YEAR: i16 = 2015;
    const DAY: i16 = 24;

    #[test]
    fn test_fn1_input() {
        scaffold_test(YEAR, DAY, "input.txt", "input-spec.1.txt", fn1);
    }

    #[test]
    fn test_fn2_input() {
        scaffold_test(YEAR, DAY, "input.txt", "input-spec.2.txt", fn2);
    }
}
