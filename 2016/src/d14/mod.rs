use std::{collections::VecDeque, fmt::Write};
use svutils::SolveError;

const HEX: &[u8; 16] = b"0123456789abcdef";

#[derive(Clone, Copy, Default)]
struct Runs {
    triple: Option<usize>,
    quintuples: u16,
}

impl Runs {
    fn from_digest(digest: md5::Digest) -> Self {
        let mut runs = Self::default();
        let mut previous = 16;
        let mut length = 0;
        for digit in digest.0.into_iter().flat_map(|byte| [byte >> 4, byte & 15]) {
            if digit == previous {
                length += 1;
            } else {
                previous = digit;
                length = 1;
            }
            if length == 3 {
                runs.triple.get_or_insert(usize::from(digit));
            }
            if length == 5 {
                runs.quintuples |= 1 << digit;
            }
        }
        runs
    }
}

fn key_index(
    hashes: impl Iterator<Item = Runs>,
    lookahead: usize,
    keys: usize,
) -> Result<usize, SolveError> {
    let mut window = VecDeque::new();
    let mut counts = [0usize; 16];
    let mut found = 0;
    for (hash_index, hash) in hashes.enumerate() {
        for (digit, count) in counts.iter_mut().enumerate() {
            *count += usize::from(hash.quintuples & (1 << digit) != 0);
        }
        window.push_back(hash);
        if window.len() <= lookahead {
            continue;
        }
        let candidate = window.pop_front().unwrap();
        for (digit, count) in counts.iter_mut().enumerate() {
            *count -= usize::from(candidate.quintuples & (1 << digit) != 0);
        }
        if candidate.triple.is_some_and(|digit| counts[digit] != 0) {
            found += 1;
            if found == keys {
                return Ok(hash_index - lookahead);
            }
        }
    }
    Err(SolveError::NoSolution("hash indices exhausted"))
}

fn solve(input: &str, stretch: usize) -> Result<String, SolveError> {
    let salt = input.trim();
    if salt.is_empty() {
        return Err(SolveError::EmptyInput);
    }
    if salt.chars().any(char::is_whitespace) {
        return Err(SolveError::InvalidInput(
            "expected one salt without whitespace".into(),
        ));
    }

    let mut candidate = String::from(salt);
    let hashes = (0..=usize::MAX).map(|index| {
        candidate.truncate(salt.len());
        write!(&mut candidate, "{index}").expect("writing to a String cannot fail");
        let mut digest = md5::compute(candidate.as_bytes());
        let mut hex = [0; 32];
        for _ in 0..stretch {
            for (byte, pair) in digest.0.iter().zip(hex.chunks_exact_mut(2)) {
                pair[0] = HEX[usize::from(byte >> 4)];
                pair[1] = HEX[usize::from(byte & 15)];
            }
            digest = md5::compute(hex);
        }
        Runs::from_digest(digest)
    });
    Ok(key_index(hashes, 1_000, 64)?.to_string())
}

pub fn part1(input: &str) -> Result<String, SolveError> {
    solve(input, 0)
}

pub fn part2(input: &str) -> Result<String, SolveError> {
    solve(input, 2016)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn retains_first_triple_and_all_quintuples() {
        let mut bytes = [0x12; 16];
        bytes[..7].copy_from_slice(&[0x77, 0x71, 0xaa, 0xaa, 0xab, 0xbb, 0xbb]);
        let runs = Runs::from_digest(md5::Digest(bytes));
        assert_eq!(runs.triple, Some(7));
        assert_eq!(runs.quintuples, (1 << 10) | (1 << 11));
    }

    #[test]
    fn excludes_self_and_includes_far_endpoint() {
        let candidate = Runs {
            triple: Some(3),
            quintuples: 1 << 3,
        };
        let empty = Runs::default();
        let confirms = Runs {
            triple: None,
            quintuples: 1 << 3,
        };
        assert!(key_index([candidate, empty, empty, confirms].into_iter(), 2, 1).is_err());
        assert_eq!(
            key_index([candidate, empty, confirms].into_iter(), 2, 1).unwrap(),
            0
        );
        assert_eq!(
            key_index([empty, candidate, confirms].into_iter(), 1, 1).unwrap(),
            1
        );
    }
}
