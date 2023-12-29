extern crate core;

fn read_line(line: &str) -> (usize, usize) {
    let mem_len = line.as_bytes().len();

    let mut char_len = 0;
    let mut string = &line.as_bytes()[1..mem_len - 1];
    while !string.is_empty() {
        let (ch, rest) = string.split_first().unwrap();
        if let b'\\' = ch {
            match rest.split_first().unwrap() {
                (b'"' | b'\\', _) => {
                    string = &rest[1..];
                }
                (b'x', _) => {
                    string = &rest[3..];
                }
                _ => unreachable!(),
            }
        } else {
            string = rest;
        }
        char_len += 1;
    }
    (mem_len, char_len)
}

fn encode_line(line: &str) -> (usize, usize) {
    let mem_len = line.as_bytes().len();

    let enc_len = mem_len
        + 2
        + line
            .as_bytes()
            .iter()
            .filter(|&ch| ch == &b'"' || ch == &b'\\')
            .count();

    (mem_len, enc_len)
}

pub fn fn1(input: &str) -> i64 {
    input.lines().map(read_line).map(|(m, c)| m - c).sum::<usize>() as i64
}

pub fn fn2(input: &str) -> i64 {
    input.lines().map(encode_line).map(|(m, c)| c - m).sum::<usize>() as i64
}

#[cfg(test)]
mod tests {
    use super::*;
    use svutils::scaffold_test;

    const YEAR: i16 = 2015;
    const DAY: i16 = 8;

    #[test]
    fn test_fn1_example() {
        scaffold_test(YEAR, DAY, "example.1.txt", "example-spec.1.txt", fn1);
    }

    #[test]
    fn test_fn1_input() {
        scaffold_test(YEAR, DAY, "input.txt", "input-spec.1.txt", fn1);
    }

    #[test]    
    fn test_fn2_example() {
        scaffold_test(YEAR, DAY, "example.2.txt", "example-spec.2.txt", fn2);
    }

    #[test]
    fn test_fn2_input() {
        scaffold_test(YEAR, DAY, "input.txt", "input-spec.2.txt", fn2);
    }
}
