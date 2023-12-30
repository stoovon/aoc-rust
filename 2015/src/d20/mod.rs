extern crate core;

pub fn solve(input: &str, presents_per_elf: usize, limit: usize) -> i64 {
    let threshold = input.trim().parse::<usize>().unwrap();
    let mut houses = vec![0; threshold / 10 + 1];

    for elf in 1..houses.len(){
        let mut loc = elf;
        let mut delivered = 0;
        while loc < houses.len()  && delivered < limit {
            houses[loc] += elf * presents_per_elf;
            loc += elf;
            delivered += 1;
        }
    }

    for (i, house) in houses.iter().enumerate() {
        if *house >= threshold {
            return i as i64;
        }
    }

    panic!("No house found");
}

pub fn fn1(input: &str) -> i64 {
    solve(input, 10, usize::MAX)
}

pub fn fn2(input: &str) -> i64 {
    solve(input, 11, 50)
}

#[cfg(test)]
mod tests {
    use super::*;
    use svutils::scaffold_test;

    const YEAR: i16 = 2015;
    const DAY: i16 = 20;

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
