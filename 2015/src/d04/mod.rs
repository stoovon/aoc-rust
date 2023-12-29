extern crate core;
use md5;

fn solve(input: &str, prefix: &str) -> i64 {
    let mut i = 0;
    loop {
        let digest = md5::compute(format!("{}{}", input, i));
        let hex = format!("{:x}", digest);
        if hex.starts_with(prefix) {
            return i;
        }
        i += 1;
    }
}

pub fn fn1(input: &str) -> i64 {
    solve(input, "00000")
}

pub fn fn2(input: &str) -> i64 {
    solve(input, "000000")
}

#[cfg(test)]
mod tests {
    use super::*;
    use svutils::scaffold_test;

    const YEAR: i16 = 2015;
    const DAY: i16 = 4;

    #[test]
    fn test_fn1_example_1() {
        scaffold_test(YEAR, DAY, "example.1.1.txt", "example-spec.1.1.txt", fn1);
    }

    #[test]
    fn test_fn1_example_2() {
        scaffold_test(YEAR, DAY, "example.1.2.txt", "example-spec.1.2.txt", fn1);
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
