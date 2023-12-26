extern crate core;

use itertools::Itertools;

fn permutations(spring: &str, counts: Vec<i64>) -> i64 {
    let spring = format!(".{}", spring.trim_end_matches('.'));
    let spring = spring.chars().collect_vec();

    let mut possible = vec![0; spring.len() + 1];

    // We're going to place the first broken spring at the start, padded by 0 or more working springs.
    possible[0] = 1;

    for (i, _) in spring.iter().take_while(|&&c| c != '#').enumerate() {
        possible[i + 1] = 1;
    }

    for count in counts {
        let mut new_possible = vec![0; spring.len() + 1];
        let mut chunk = 0;

        for (i, &c) in spring.iter().enumerate() {
            if c != '.' {
                chunk += 1;
            } else {
                chunk = 0;
            }

            if c != '#' {
                new_possible[i + 1] += new_possible[i];
            }

            if chunk >= count && spring[i - count as usize] != '#' {
                new_possible[i + 1] += possible[i - count as usize];
            }
        }

        possible = new_possible;
    }

    *possible.last().unwrap()
}

pub fn fn1(input: &str) -> i64 {
    input
        .lines()
        .map(|line| {
            let (spring, counts) = line.split_once(' ').unwrap();
            let counts = counts
                .split(',')
                .map(|number| number.parse::<i64>().unwrap()).collect_vec();
            permutations(spring, counts)
        })
        .sum()
}

pub fn fn2(input: &str) -> i64 {
    input
        .lines()
        .map(|line| {
            let (spring, counts) = line.split_once(' ').unwrap();

            let spring = std::iter::once(spring).cycle().take(5).join("?");

            let counts = counts
                .split(',')
                .map(|number| number.parse::<i64>().unwrap())
                .collect_vec();
            let n = counts.len();

            permutations(&spring, counts.into_iter().cycle().take(5 * n).collect_vec())
        })
        .sum()
}

#[cfg(test)]
mod tests {
    use super::*;
    use svutils::scaffold_test;

    const YEAR: i16 = 2023;
    const DAY: i16 = 12;

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
