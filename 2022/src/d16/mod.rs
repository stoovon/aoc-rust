extern crate core;

pub fn fn1(_input: &str) -> i64 {
    todo!()
}

pub fn fn2(_input: &str) -> i64 {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;
    use svutils::scaffold_test;

    const YEAR: i16 = 2022;
    const DAY: i16 = 16;

    #[test]
    #[ignore]
    fn test_fn1_example() {
        scaffold_test(YEAR, DAY, "example.txt", "example-spec.1.txt", fn1);
    }

    #[test]
    #[ignore]
    fn test_fn1_input() {
        scaffold_test(YEAR, DAY, "input.txt", "input-spec.1.txt", fn1);
    }

    #[test]
    #[ignore]
    fn test_fn2_example() {
        scaffold_test(YEAR, DAY, "example.txt", "example-spec.2.txt", fn2);
    }

    #[test]
    #[ignore]
    fn test_fn2_input() {
        scaffold_test(YEAR, DAY, "input.txt", "input-spec.2.txt", fn2);
    }
}
