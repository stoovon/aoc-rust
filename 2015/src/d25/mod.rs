extern crate core;

use regex::Regex;

fn parse(input: &str) -> Vec<i64> {
    let re = Regex::new(r"row (\d+), column (\d+)").unwrap();
    let caps = re.captures(input).unwrap();
    vec![caps[1].parse().unwrap(), caps[2].parse().unwrap()]
}

fn find_num_for_coords(row: i64, col: i64) -> i64 {
    1 + (col + row - 1) * (col + row) / 2 - row
}

fn num_to_code(num: i64) -> i64 {
    let mut code = 20151125;
    for _ in 1..num {
        code = (code * 252533) % 33554393;
    }
    code
}

pub fn fn1(input: &str) -> i64 {
    let row = parse(input)[0];
    let col = parse(input)[1];

    num_to_code(find_num_for_coords(row, col))
}

#[cfg(test)]
mod tests {
    use super::*;
    use svutils::scaffold_test;

    const YEAR: i16 = 2015;
    const DAY: i16 = 25;

    #[test]
    fn test_fn1_input() {
        scaffold_test(YEAR, DAY, "input.txt", "input-spec.1.txt", fn1);
    }
}
