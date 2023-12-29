extern crate core;

use fancy_regex::Regex;

pub fn fn1(input: &str) -> i64 {
    input
        .lines()
        .filter(|line| {
            let mut vowels = 0;
            let mut double = false;
            let mut bad = false;
            let mut last = ' ';
            for c in line.chars() {
                if c == 'a' || c == 'e' || c == 'i' || c == 'o' || c == 'u' {
                    vowels += 1;
                }
                if c == last {
                    double = true;
                }
                if (last == 'a' && c == 'b')
                    || (last == 'c' && c == 'd')
                    || (last == 'p' && c == 'q')
                    || (last == 'x' && c == 'y')
                {
                    bad = true;
                }
                last = c;
            }
            vowels >= 3 && double && !bad
        })
        .count() as i64
}

pub fn fn2(input: &str) -> i64 {
    let re1 = Regex::new(r"(..).*\1").unwrap();
    let re2 = Regex::new(r"(.).\1").unwrap();

    input
        .lines()
        .filter(|line| re1.is_match(line).unwrap() && re2.is_match(line).unwrap())
        .count() as i64
}

#[cfg(test)]
mod tests {
    use super::*;
    use svutils::scaffold_test;

    const YEAR: i16 = 2015;
    const DAY: i16 = 5;

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
