extern crate core;

use std::collections::HashMap;

pub fn fn1(input: &str) -> i64 {
    let mut pos = (0, 0);
    let mut houses: HashMap<(i32, i32), i32> = HashMap::new();
    houses.insert(pos, 1);

    for c in input.chars() {
        match c {
            '^' => pos.1 += 1,
            '>' => pos.0 += 1,
            'v' => pos.1 -= 1,
            '<' => pos.0 -= 1,
            _ => panic!("Unknown character: {}", c),
        }

        houses
            .entry(pos)
            .and_modify(|visits| *visits += 1)
            .or_insert(1);
    }

    houses.len() as i64
}

pub fn fn2(input: &str) -> i64 {
    let mut santa = (0, 0);
    let mut robo_santa = (0, 0);
    let mut houses: HashMap<(i32, i32), i32> = HashMap::new();

    // Lucky first house!
    houses.insert(santa, 1);
    houses.insert(robo_santa, 1);

    // First mut says I can have active point to different things, and the second mut says active's values can be changed.
    let mut active: &mut (i32, i32);

    for (i, c) in input.chars().enumerate() {
        active = if i % 2 == 0 {
            &mut santa
        } else {
            &mut robo_santa
        };

        match c {
            '^' => active.1 += 1,
            '>' => active.0 += 1,
            'v' => active.1 -= 1,
            '<' => active.0 -= 1,
            _ => panic!("Unknown character: {}", c),
        }

        houses
            .entry(*active)
            .and_modify(|visits| *visits += 1)
            .or_insert(1);
    }

    houses.len() as i64
}

#[cfg(test)]
mod tests {
    use super::*;
    use svutils::scaffold_test;

    const YEAR: i16 = 2015;
    const DAY: i16 = 3;

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
