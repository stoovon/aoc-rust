extern crate core;

use itertools::Itertools;
use regex::Regex;
use std::collections::HashMap;

pub fn solve(input: &str) -> (i64, i64) {
    let input: Vec<&str> = input.lines().collect_vec();

    let mut part1_sum: i64 = 0;
    let re = Regex::new(r"(.*)-(\d+)\[(\w{5})\]").unwrap();
    let mut northpole_object_storage = 0;

    for line in &input {
        let caps = re.captures(line).unwrap();

        let mut freqs = HashMap::new();
        for ch in caps[1].chars().filter(|ch| ch.is_alphabetic()) {
            *freqs.entry(ch).or_insert(0) += 1;
        }

        let mut letters = freqs.iter().collect::<Vec<_>>();
        letters.sort_by(|a, b| {
            if a.1 != b.1 {
                a.1.cmp(b.1).reverse()
            } else {
                a.0.cmp(b.0)
            }
        });

        let actual_checksum = letters
            .iter()
            .take(5)
            .map(|freq| freq.0)
            .collect::<String>();

        if caps[3] == actual_checksum {
            let sector_id = caps[2].parse::<i64>().unwrap();
            part1_sum += sector_id;

            let decrypted = caps[1]
                .chars()
                .map(|ch| {
                    if ch.is_ascii_lowercase() {
                        let cur_shift = ch as u8 - 'a' as u8;
                        let new_shift = ((cur_shift as i64 + sector_id) % 26) as u8;
                        ('a' as u8 + new_shift) as char
                    } else {
                        ch
                    }
                })
                .collect::<String>();

            if decrypted == "northpole-object-storage" {
                northpole_object_storage = sector_id;
            }
        }
    }

    (part1_sum, northpole_object_storage)
}

pub fn fn1(input: &str) -> i64 {
    solve(input).0
}

pub fn fn2(input: &str) -> i64 {
    solve(input).1
}

#[cfg(test)]
mod tests {
    use super::*;
    use svutils::scaffold_test;

    const YEAR: i16 = 2016;
    const DAY: i16 = 4;

    #[test]
    fn test_fn1_example() {
        scaffold_test(YEAR, DAY, "example.txt", "example-spec.1.txt", fn1);
    }

    #[test]
    fn test_fn1_input() {
        scaffold_test(YEAR, DAY, "input.txt", "input-spec.1.txt", fn1);
    }

    #[test]
    fn test_fn2_input() {
        scaffold_test(YEAR, DAY, "input.txt", "input-spec.2.txt", fn2);
    }
}
