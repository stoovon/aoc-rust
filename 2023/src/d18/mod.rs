extern crate core;

use regex::Regex;

fn small_directions(text: &str) -> Vec<(char, i64)> {
    // Discard the hex colours
    let regex = Regex::new(r"(?m)^([RLDU]) ([[:digit:]]+)").unwrap();

    regex
        .captures_iter(text)
        .map(|cap| {
            let (_, [digit, number]) = cap.extract();
            (digit.chars().next().unwrap(), number.parse().unwrap())
        })
        .collect()
}

fn large_directions(text: &str) -> Vec<(char, i64)> {
    let regex = Regex::new(r"(?m)\(\#([[:xdigit:]]{5})([0-3])\)$").unwrap();
    regex
        .captures_iter(text)
        .map(|cap| {
            let (_, [hexstr, d]) = cap.extract();
            let d_int = usize::from_str_radix(d, 16).unwrap();
            let dir = ['R', 'D', 'L', 'U'][d_int];
            let hex = i64::from_str_radix(hexstr, 16).unwrap();
            (dir, hex)
        })
        .collect()
}

fn get_area(dirs: &[(char, i64)]) -> i64 {
    // Shoelace
    let (perimeter, area, _) = dirs
        .iter()
        .fold((0, 0, (0, 0)), |(p, a, (y, x)), (d, l)| match d {
            'U' => (p + l, a - x * l, (y - l, x)),
            'R' => (p + l, a, (y, x + l)),
            'D' => (p + l, a + x * l, (y + l, x)),
            'L' => (p + l, a, (y, x - l)),
            _ => panic!("Unknown direction {d}"),
        });
    area + perimeter / 2 + 1
}

pub fn fn1(input: &str) -> i64 {
    let dirs = small_directions(input);
    get_area(&dirs)
}

pub fn fn2(input: &str) -> i64 {
    let dirs = large_directions(input);
    get_area(&dirs)
}

#[cfg(test)]
mod tests {
    use super::*;
    use svutils::scaffold_test;

    const YEAR: i16 = 2023;
    const DAY: i16 = 18;

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
