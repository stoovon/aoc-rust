extern crate core;

pub fn fn1(_input: &str) -> i64 {
    // Build a list of connections
    // Initial solve in Python/networkx. Will try to implement the solution in Rust soon.

    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;
    use svutils::scaffold_test;

    const YEAR: i16 = 2023;
    const DAY: i16 = 25;

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
}
