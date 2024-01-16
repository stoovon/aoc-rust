use itertools::Itertools;
use std::collections::HashSet;

extern crate core;

pub fn fn1(input: &str) -> i64 {
    let input: Vec<Vec<char>> = input
        .lines()
        .map(|line| line.chars().collect::<Vec<_>>())
        .collect_vec();

    let filter_ip7 = |state: &mut bool, &ch| {
        if ch == '[' || ch == ']' {
            *state = !*state;
        }

        if *state && (ch == ']' || ch == '[') {
            Some(Ok(' '))
        } else if *state {
            Some(Ok(ch))
        } else {
            Some(Err(0))
        }
    };

    let abba_detect = |quad: &[char]| {
        let arr = quad.iter().collect::<Vec<_>>();
        arr[0] != arr[1] && arr[0] == arr[3] && arr[1] == arr[2]
    };

    let abbas = input
        .iter()
        .filter(|line| {
            let trimmed = line
                .iter()
                .scan(true, &filter_ip7)
                .filter_map(|ch| ch.ok())
                .collect::<String>();
            let hypernet = line
                .iter()
                .scan(false, &filter_ip7)
                .filter_map(|ch| ch.ok())
                .collect::<String>();

            trimmed.split_whitespace().any(|chunk| {
                chunk
                    .chars()
                    .collect::<Vec<_>>()
                    .windows(4)
                    .any(&abba_detect)
            }) && !hypernet.split_whitespace().any(|chunk| {
                chunk
                    .chars()
                    .collect::<Vec<_>>()
                    .windows(4)
                    .any(&abba_detect)
            })
        })
        .count();

    abbas as i64
}

pub fn fn2(input: &str) -> i64 {
    let input: Vec<Vec<char>> = input
        .lines()
        .map(|line| line.chars().collect::<Vec<_>>())
        .collect_vec();

    let filter_ip7 = |state: &mut bool, &ch| {
        if ch == '[' || ch == ']' {
            *state = !*state;
        }

        if *state && (ch == ']' || ch == '[') {
            Some(Ok(' '))
        } else if *state {
            Some(Ok(ch))
        } else {
            Some(Err(0))
        }
    };

    input
        .iter()
        .filter(|line| {
            let trimmed = line
                .iter()
                .scan(true, &filter_ip7)
                .filter_map(|ch| ch.ok())
                .collect::<String>();
            let hypernet = line
                .iter()
                .scan(false, &filter_ip7)
                .filter_map(|ch| ch.ok())
                .collect::<String>();

            let mut abas = HashSet::new();
            trimmed
                .split_whitespace()
                .map(|chunk| chunk.chars().collect::<Vec<_>>())
                .for_each(|chunk| {
                    chunk.windows(3).for_each(|aba| {
                        let arr = aba.iter().cloned().collect::<Vec<_>>();
                        if arr[0] != arr[1] && arr[0] == arr[2] {
                            abas.insert(arr.clone());
                        }
                    })
                });

            hypernet.split_whitespace().any(|chunk| {
                chunk.chars().collect::<Vec<_>>().windows(3).any(|bab| {
                    let arr = bab.iter().collect::<Vec<_>>();
                    let aba = vec![arr[1].clone(), arr[0].clone(), arr[1].clone()];
                    arr[0] != arr[1] && arr[0] == arr[2] && abas.contains(&aba)
                })
            })
        })
        .count() as i64
}

#[cfg(test)]
mod tests {
    use super::*;
    use svutils::scaffold_test;

    const YEAR: i16 = 2016;
    const DAY: i16 = 7;

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
