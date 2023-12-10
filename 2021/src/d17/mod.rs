extern crate core;

pub fn fn1(_input: &str) -> i64 {
    todo!()
}

pub fn fn2(_input: &[&str]) -> i64 {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;
    use svutils::load_spec;
    use itertools::Itertools;

    #[test]
    fn test_example() {
        assert_eq!(fn1(include_str!("example.txt")), load_spec(include_str!("example-spec.txt")));
    }

    #[test]
    fn test_fn1_test_case_name() {
        assert_eq!(fn1(include_str!("input.txt")), load_spec(include_str!("input-spec.1.txt")));
    }

    #[test]
    fn test_fn2_test_case_name() {
        assert_eq!(fn2(&include_str!("input.txt").lines().collect_vec()), load_spec(include_str!("input-spec.2.txt")));
    }
}