use std::fmt::Write;
use svutils::SolveError;

fn key(input: &str) -> Result<&str, SolveError> {
    let key = input.trim();
    if key.is_empty() {
        return Err(SolveError::EmptyInput);
    }
    if !key.bytes().all(|byte| byte.is_ascii_alphabetic()) {
        return Err(SolveError::InvalidInput(
            "expected an ASCII alphabetic secret key".into(),
        ));
    }
    Ok(key)
}

fn has_prefix(digest: &[u8; 16], six_zeroes: bool) -> bool {
    digest[0] == 0
        && digest[1] == 0
        && if six_zeroes {
            digest[2] == 0
        } else {
            digest[2] & 0xf0 == 0
        }
}

fn mine(key: &str, six_zeroes: bool) -> Result<u64, SolveError> {
    let mut candidate = String::from(key);
    let mut number = 1u64;
    loop {
        candidate.truncate(key.len());
        write!(&mut candidate, "{number}")
            .map_err(|_| SolveError::InvalidInput("could not format candidate".into()))?;
        if has_prefix(&md5::compute(candidate.as_bytes()).0, six_zeroes) {
            return Ok(number);
        }
        number = number
            .checked_add(1)
            .ok_or(SolveError::ArithmeticOverflow)?;
    }
}

pub fn part1(input: &str) -> Result<String, SolveError> {
    mine(key(input)?, false).map(|number| number.to_string())
}

pub fn part2(input: &str) -> Result<String, SolveError> {
    mine(key(input)?, true).map(|number| number.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use svutils::scaffold_test_result;

    #[test]
    fn byte_prefix_matches_hexadecimal_definition() {
        for third in 0..=255u8 {
            let mut bytes = [0xabu8; 16];
            bytes[..2].fill(0);
            bytes[2] = third;
            let hex = format!("{:x}", md5::Digest(bytes));
            assert_eq!(has_prefix(&bytes, false), hex.starts_with("00000"));
            assert_eq!(has_prefix(&bytes, true), hex.starts_with("000000"));
            for index in 0..2 {
                for nonzero in 1..=255u8 {
                    let mut invalid = bytes;
                    invalid[index] = nonzero;
                    assert!(!has_prefix(&invalid, false));
                    assert!(!has_prefix(&invalid, true));
                }
            }
        }
    }

    #[test]
    fn key_validation_and_outer_whitespace() {
        assert_eq!(key(" \t\r\nSyntheticKey\r\n").unwrap(), "SyntheticKey");
        for input in ["", " \r\n"] {
            assert!(matches!(part1(input), Err(SolveError::EmptyInput)));
            assert!(matches!(part2(input), Err(SolveError::EmptyInput)));
        }
        for input in ["two keys", "key\nkey", "key9", "clé", "key!"] {
            assert!(part1(input).is_err());
            assert!(part2(input).is_err());
        }
    }

    #[test]
    fn test_part1_input() {
        scaffold_test_result(2015, 4, "input.txt", "input-spec.1.txt", part1);
    }

    #[test]
    fn test_part2_input() {
        scaffold_test_result(2015, 4, "input.txt", "input-spec.2.txt", part2);
    }
}
