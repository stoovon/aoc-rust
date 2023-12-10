extern crate core;

pub fn fn1(input: &str) -> i64 {
    let data: Vec<i64> = input
        .lines()
        .map(|v| v.parse::<i64>().ok().unwrap_or_default())
        .collect();

    data.windows(2).filter(|d| d[0] < d[1]).count().try_into().unwrap()
}

pub fn fn2(input: &str) -> i64 {
    let data: Vec<i64> = input
        .lines()
        .map(|v| v.parse::<i64>().ok().unwrap_or_default())
        .collect();

    data.windows(3)
        .zip(data.windows(3).skip(1))
        .filter(|(a, b)| a.iter().sum::<i64>() < b.iter().sum())
        .count().try_into().unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use svutils::load_spec;

    #[test]
    fn test_fn1_example() {
        assert_eq!(fn1(include_str!("example.txt")), load_spec(include_str!("example-spec.1.txt")));
    }

    #[test]
    fn test_fn1_input() {
        assert_eq!(fn1(include_str!("input.txt")), load_spec(include_str!("input-spec.1.txt")));
    }

    #[test]
    fn test_fn2_example() {
        assert_eq!(fn2(include_str!("example.txt")), load_spec(include_str!("example-spec.2.txt")));
    }

    #[test]
    fn test_fn2_input() {
        assert_eq!(fn2(include_str!("input.txt")), load_spec(include_str!("input-spec.2.txt")));
    }
}