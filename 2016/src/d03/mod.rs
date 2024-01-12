extern crate core;

fn valid_triangle(sides: &[isize; 3]) -> bool {
    let mut sides = sides.to_vec();
    sides.sort();
    sides[0] + sides[1] > sides[2]
}

pub fn fn1(input: &str) -> i64 {
    let mut count = 0;
    for line in input.lines() {
        let mut sides = line.split_whitespace().map(|s| s.parse::<isize>().unwrap());
        let a = sides.next().unwrap();
        let b = sides.next().unwrap();
        let c = sides.next().unwrap();

        if valid_triangle(&[a, b, c]) {
            count += 1;
        }
    }

    count
}

pub fn fn2(input: &str) -> i64 {
    let mut count = 0;
    let mut index = 0;
    let mut buffer = [[0isize; 3]; 3];

    for strs in input.lines() {
        let line_strs = strs.split_whitespace().collect::<Vec<_>>();
        if line_strs.len() != 3 {
            panic!("line does not have three lengths");
        };
        for i in 0..3 {
            buffer[i][index] = line_strs[i].parse().unwrap();
        }

        index += 1;
        if index == 3 {
            for b in &mut buffer {
                if valid_triangle(b) {
                    count += 1;
                }
            }
            index = 0;
        }
    }

    if index != 0 {
        panic!("uneven number of lines in input");
    };
    count
}

#[cfg(test)]
mod tests {
    use super::*;
    use svutils::scaffold_test;

    const YEAR: i16 = 2016;
    const DAY: i16 = 3;

    #[test]
    fn test_fn1_input() {
        scaffold_test(YEAR, DAY, "input.txt", "input-spec.1.txt", fn1);
    }

    #[test]
    fn test_fn2_input() {
        scaffold_test(YEAR, DAY, "input.txt", "input-spec.2.txt", fn2);
    }
}
