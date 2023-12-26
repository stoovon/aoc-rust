extern crate core;

pub fn fn1(input: &str) -> i64 {
    let mut data = input.lines();
    let times = data
        .next()
        .unwrap()
        .split(":")
        .nth(1)
        .unwrap()
        .trim()
        .split_ascii_whitespace()
        .map(|x| x.parse::<i64>().unwrap());

    let distances = data
        .next()
        .unwrap()
        .split(":")
        .nth(1)
        .unwrap()
        .trim()
        .split_ascii_whitespace()
        .map(|x| x.parse::<i64>().unwrap());

    times
        .zip(distances)
        .map(|(max_time, win_distance)| {
            let mut combinations = 0;
            for charge_duration in 1..max_time {
                let remaining_time = max_time - charge_duration;
                let speed = charge_duration;
                let distance_covered = speed * remaining_time;
                if distance_covered > win_distance {
                    combinations += 1;
                }
            }

            combinations
        })
        .product()
}

pub fn fn2(input: &str) -> i64 {
    let mut data = input.lines();
    let time = data
        .next()
        .unwrap()
        .split(":")
        .nth(1)
        .unwrap()
        .trim()
        .split_ascii_whitespace()
        .collect::<String>()
        .parse::<i64>()
        .unwrap();

    let distance = data
        .next()
        .unwrap()
        .split(":")
        .nth(1)
        .unwrap()
        .trim()
        .split_ascii_whitespace()
        .collect::<String>()
        .parse::<i64>()
        .unwrap();

    std::iter::once(time)
        .zip(std::iter::once(distance))
        .map(|(max_time, win_distance)| {
            let mut combinations = 0;
            for charge_duration in 1..max_time {
                let remaining_time = max_time - charge_duration;
                let speed = charge_duration;
                let distance_covered = speed * remaining_time;
                if distance_covered > win_distance {
                    combinations += 1;
                }
            }

            combinations
        })
        .product()
}

#[cfg(test)]
mod tests {
    use super::*;
    use svutils::scaffold_test;

    const YEAR: i16 = 2023;
    const DAY: i16 = 6;

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
