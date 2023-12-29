extern crate core;

pub fn fn1(input: &str) -> i64 {
    let data: Vec<i64> = input
        .lines()
        .map(|v| v.parse::<i64>().ok().unwrap_or_default())
        .collect();

    data.windows(2)
        .filter(|d| d[0] < d[1])
        .count()
        .try_into()
        .unwrap()
}

pub fn fn2(input: &str) -> i64 {
    let data: Vec<i64> = input
        .lines()
        .map(|v| v.parse::<i64>().ok().unwrap_or_default())
        .collect();

    data.windows(3)
        .zip(data.windows(3).skip(1))
        .filter(|(a, b)| a.iter().sum::<i64>() < b.iter().sum())
        .count()
        .try_into()
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use svutils::scaffold_test;

    const YEAR: i16 = 2021;
    const DAY: i16 = 1;

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
