use std::fmt::Write;
use svutils::SolveError;

const PASSWORD_LEN: usize = 8;
const EXHAUSTED: SolveError =
    SolveError::NoSolution("candidate counter exhausted before completing the password");
const HEX: &[u8; 16] = b"0123456789abcdef";

fn password_digits(digest: &md5::Digest) -> Option<(u8, u8)> {
    (digest[0] == 0 && digest[1] == 0 && digest[2] & 0xf0 == 0)
        .then_some((digest[2] & 0x0f, digest[3] >> 4))
}

fn matching_digits(input: &str) -> impl Iterator<Item = (u8, u8)> {
    let prefix_len = input.len();
    let mut candidate = String::with_capacity(prefix_len + 20);
    candidate.push_str(input);

    (0..=u64::MAX).filter_map(move |index| {
        candidate.truncate(prefix_len);
        write!(&mut candidate, "{index}").expect("writing to a String cannot fail");
        password_digits(&md5::compute(candidate.as_bytes()))
    })
}

fn door_id(input: &str) -> Result<&str, SolveError> {
    let mut lines = input.lines();
    let id = lines
        .next()
        .filter(|line| !line.is_empty())
        .ok_or(SolveError::EmptyInput)?;
    if lines.next().is_some() || id.chars().any(char::is_whitespace) {
        return Err(SolveError::InvalidInput(
            "expected one door ID without whitespace".into(),
        ));
    }
    Ok(id)
}

fn sequential_password(digits: impl Iterator<Item = (u8, u8)>) -> Result<String, SolveError> {
    let password: String = digits
        .take(PASSWORD_LEN)
        .map(|(digit, _)| char::from(HEX[usize::from(digit)]))
        .collect();
    if password.len() == PASSWORD_LEN {
        Ok(password)
    } else {
        Err(EXHAUSTED)
    }
}

fn positional_password(digits: impl Iterator<Item = (u8, u8)>) -> Result<String, SolveError> {
    let mut password = [None; PASSWORD_LEN];
    let mut remaining = password.len();
    for (slot, digit) in digits {
        if let Some(entry @ None) = password.get_mut(usize::from(slot)) {
            *entry = Some(char::from(HEX[usize::from(digit)]));
            remaining -= 1;
            if remaining == 0 {
                return Ok(password.into_iter().flatten().collect());
            }
        }
    }
    Err(EXHAUSTED)
}

pub fn part1(input: &str) -> Result<String, SolveError> {
    sequential_password(matching_digits(door_id(input)?))
}

pub fn part2(input: &str) -> Result<String, SolveError> {
    positional_password(matching_digits(door_id(input)?))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn checks_prefix_and_extracts_digits() {
        let mut bytes = [0; 16];
        bytes[2] = 0x07;
        bytes[3] = 0xab;
        assert_eq!(password_digits(&md5::Digest(bytes)), Some((7, 10)));
        for (index, bit) in [(0, 1), (1, 1), (2, 0x10)] {
            let mut invalid = bytes;
            invalid[index] |= bit;
            assert_eq!(password_digits(&md5::Digest(invalid)), None);
        }
    }

    #[test]
    fn completes_passwords_and_ignores_duplicate_or_invalid_slots() {
        let digits = [
            (15, 0),
            (2, 9),
            (2, 0),
            (7, 14),
            (0, 11),
            (6, 12),
            (1, 8),
            (5, 6),
            (4, 3),
            (3, 15),
        ];
        assert_eq!(sequential_password(digits.into_iter()).unwrap(), "f2270615");
        assert_eq!(positional_password(digits.into_iter()).unwrap(), "b89f36ce");
    }

    #[test]
    fn exhaustion_reports_no_solution() {
        let digits = [(0, 1); 7];
        assert!(matches!(
            sequential_password(digits.into_iter()),
            Err(SolveError::NoSolution(_))
        ));
        assert!(matches!(
            positional_password(digits.into_iter()),
            Err(SolveError::NoSolution(_))
        ));
    }

    #[test]
    fn validates_door_id() {
        for input in ["room", "room\n", "room\r\n"] {
            assert_eq!(door_id(input).unwrap(), "room");
        }
        assert!(matches!(door_id(""), Err(SolveError::EmptyInput)));
        for input in ["two words", " room", "room\n\n", "room\nextra"] {
            assert!(matches!(door_id(input), Err(SolveError::InvalidInput(_))));
        }
    }
}
