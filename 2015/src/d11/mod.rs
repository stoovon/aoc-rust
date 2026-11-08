use svutils::SolveError;

const MAX_CANDIDATES: usize = 1_000_000;

struct Password {
    letters: [u8; 8],
}

impl Password {
    fn parse(input: &str) -> Result<Self, SolveError> {
        let input = input.trim();
        if input.is_empty() {
            return Err(SolveError::EmptyInput);
        }
        let letters: [u8; 8] = input.as_bytes().try_into().map_err(|_| {
            SolveError::InvalidInput("expected exactly eight lowercase ASCII letters".into())
        })?;
        if !letters.iter().all(u8::is_ascii_lowercase) {
            return Err(SolveError::InvalidInput(
                "expected exactly eight lowercase ASCII letters".into(),
            ));
        }
        Ok(Self { letters })
    }

    // The caller has already removed forbidden letters from the current candidate.
    fn next_candidate(&mut self) -> Result<(), SolveError> {
        let Some(position) = self.letters.iter().rposition(|&letter| letter != b'z') else {
            return Err(SolveError::InvalidInput(
                "no greater password exists".into(),
            ));
        };
        self.letters[position] += 1;
        if matches!(self.letters[position], b'i' | b'o' | b'l') {
            self.letters[position] += 1;
        }
        self.letters[position + 1..].fill(b'a');
        Ok(())
    }

    fn skip_forbidden(&mut self) -> bool {
        if let Some(position) = self
            .letters
            .iter()
            .position(|letter| matches!(letter, b'i' | b'o' | b'l'))
        {
            self.letters[position] += 1;
            self.letters[position + 1..].fill(b'a');
            true
        } else {
            false
        }
    }

    fn has_straight(&self) -> bool {
        self.letters
            .windows(3)
            .any(|run| run[0] + 1 == run[1] && run[1] + 1 == run[2])
    }

    fn has_distinct_pairs(&self) -> bool {
        let mut first = None;
        let mut position = 0;
        while position + 1 < self.letters.len() {
            if self.letters[position] == self.letters[position + 1] {
                let letter = self.letters[position];
                if first.is_some_and(|previous| previous != letter) {
                    return true;
                }
                first = Some(letter);
                position += 2;
            } else {
                position += 1;
            }
        }
        false
    }

    fn is_valid(&self) -> bool {
        self.has_straight()
            && self.has_distinct_pairs()
            && !self
                .letters
                .iter()
                .any(|letter| matches!(letter, b'i' | b'o' | b'l'))
    }

    fn next_valid(&mut self, remaining: &mut usize) -> Result<(), SolveError> {
        if !self.skip_forbidden() {
            self.next_candidate()?;
        }
        loop {
            *remaining = remaining.checked_sub(1).ok_or(SolveError::ResourceLimit(
                "password candidate budget exceeded",
            ))?;
            if self.is_valid() {
                return Ok(());
            }
            self.next_candidate()?;
        }
    }

    fn into_string(self) -> String {
        self.letters.into_iter().map(char::from).collect()
    }
}

fn solve(input: &str, successors: usize) -> Result<String, SolveError> {
    let mut password = Password::parse(input)?;
    let mut remaining = MAX_CANDIDATES;
    for _ in 0..successors {
        password.next_valid(&mut remaining)?;
    }
    Ok(password.into_string())
}

pub fn part1(input: &str) -> Result<String, SolveError> {
    solve(input, 1)
}
pub fn part2(input: &str) -> Result<String, SolveError> {
    solve(input, 2)
}

#[cfg(test)]
mod tests {
    use super::*;
    use svutils::scaffold_test_result;

    fn reference_valid(letters: &[u8; 8]) -> bool {
        let allowed = letters
            .iter()
            .all(|letter| ![b'i', b'o', b'l'].contains(letter));
        let straight = (0..6).any(|index| {
            letters[index + 1] as i16 - letters[index] as i16 == 1
                && letters[index + 2] as i16 - letters[index + 1] as i16 == 1
        });
        let pairs = (0..7).any(|a| {
            (a + 2..7).any(|b| {
                letters[a] == letters[a + 1]
                    && letters[b] == letters[b + 1]
                    && letters[a] != letters[b]
            })
        });
        allowed && straight && pairs
    }

    fn reference_next(letters: &mut [u8; 8]) -> bool {
        for index in (0..8).rev() {
            if letters[index] < b'z' {
                letters[index] += 1;
                return true;
            }
            letters[index] = b'a';
        }
        false
    }

    fn reference_successor(mut letters: [u8; 8]) -> Option<[u8; 8]> {
        while reference_next(&mut letters) {
            if reference_valid(&letters) {
                return Some(letters);
            }
        }
        None
    }

    #[test]
    fn recognisers_match_independent_rules() {
        for mut encoded in 0..6usize.pow(8) {
            let mut letters = [0; 8];
            for letter in &mut letters {
                *letter = [b'a', b'b', b'c', b'i', b'x', b'z'][encoded % 6];
                encoded /= 6;
            }
            let password = Password { letters };
            assert_eq!(password.is_valid(), reference_valid(&letters));
        }
        for input in ["abcaaaaa", "abcaiaab", "abcaalbb", "abcaaobb"] {
            assert!(!Password::parse(input).unwrap().is_valid());
        }
        for input in ["abcaaabb", "aabbcdef"] {
            assert!(Password::parse(input).unwrap().is_valid());
        }
    }

    #[test]
    fn search_matches_unskipped_enumeration() {
        for index in 0..100 {
            let mut seed = *b"abcaaazz";
            seed[5] = b'a' + (index / 26) as u8;
            seed[6] = b'a' + (index % 26) as u8;
            let first = reference_successor(seed).unwrap();
            let second = reference_successor(first).unwrap();
            let text = std::str::from_utf8(&seed).unwrap();
            assert_eq!(part1(text).unwrap().as_bytes(), first);
            assert_eq!(part2(text).unwrap().as_bytes(), second);
        }
    }

    #[test]
    fn skipping_and_carry_boundaries() {
        for (seed, expected) in [
            ("cdfiaazz", "cdfjaaaa"),
            ("iloooooo", "jaaaaaaa"),
            ("cdflaazz", "cdfmaaaa"),
            ("cdfoaazz", "cdfpaaaa"),
        ] {
            let mut password = Password::parse(seed).unwrap();
            password.skip_forbidden();
            assert_eq!(password.into_string(), expected);
        }
        let mut password = Password::parse("cdefghzz").unwrap();
        password.next_candidate().unwrap();
        assert_eq!(password.into_string(), "cdefgjaa");
        let mut password = Password::parse("zzzzzzzz").unwrap();
        assert!(password.next_candidate().is_err());
        assert_eq!(password.into_string(), "zzzzzzzz");
        for solver in [part1, part2] {
            for seed in ["zzzzzzzz", "zzzzzzzy"] {
                assert!(solver(seed).is_err());
            }
        }
    }

    #[test]
    fn parsing_and_search_budget() {
        for input in [
            "",
            " ",
            "abc",
            "abcdefghi",
            "ABCaaabb",
            "abca abb",
            "abcaaab7",
            "ébcaaabb",
        ] {
            assert!(part1(input).is_err());
            assert!(part2(input).is_err());
        }
        assert_eq!(part1(" \tabcaaaba\r\n").unwrap(), "abcaaabb");
        let mut password = Password::parse("abcaaaba").unwrap();
        assert!(matches!(
            password.next_valid(&mut 0),
            Err(SolveError::ResourceLimit(_))
        ));
        let mut password = Password::parse("abcaaaba").unwrap();
        password.next_valid(&mut 1).unwrap();
        assert_eq!(password.into_string(), "abcaaabb");
        let mut password = Password::parse("abcaaaba").unwrap();
        let mut remaining = 1;
        password.next_valid(&mut remaining).unwrap();
        assert!(matches!(
            password.next_valid(&mut remaining),
            Err(SolveError::ResourceLimit(_))
        ));
    }

    #[test]
    fn test_part1_input() {
        scaffold_test_result(2015, 11, "input.txt", "input-spec.1.txt", part1);
    }
    #[test]
    fn test_part2_input() {
        scaffold_test_result(2015, 11, "input.txt", "input-spec.2.txt", part2);
    }
}
