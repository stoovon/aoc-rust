use svutils::SolveError;

fn records(input: &str) -> Result<Vec<&[u8]>, SolveError> {
    if input.trim().is_empty() {
        return Err(SolveError::EmptyInput);
    }
    input
        .lines()
        .enumerate()
        .map(|(index, line)| {
            if line.is_empty() || !line.bytes().all(|byte| byte.is_ascii_lowercase()) {
                return Err(SolveError::InvalidInput(
                    "expected a nonempty lowercase ASCII string".into(),
                )
                .at_line(index + 1));
            }
            Ok(line.as_bytes())
        })
        .collect()
}

fn nice_part1(word: &[u8]) -> bool {
    let vowels = word
        .iter()
        .filter(|&&byte| matches!(byte, b'a' | b'e' | b'i' | b'o' | b'u'))
        .count();
    vowels >= 3
        && word.windows(2).any(|pair| pair[0] == pair[1])
        && !word
            .windows(2)
            .any(|pair| matches!(pair, b"ab" | b"cd" | b"pq" | b"xy"))
}

fn repeated_pair(word: &[u8]) -> bool {
    let mut earliest = [None; 26 * 26];
    for (index, pair) in word.windows(2).enumerate() {
        let key = usize::from(pair[0] - b'a') * 26 + usize::from(pair[1] - b'a');
        match earliest[key] {
            Some(first) if index - first >= 2 => return true,
            None => earliest[key] = Some(index),
            _ => (),
        }
    }
    false
}

fn nice_part2(word: &[u8]) -> bool {
    word.windows(3).any(|triple| triple[0] == triple[2]) && repeated_pair(word)
}

pub fn part1(input: &str) -> Result<String, SolveError> {
    Ok(records(input)?
        .into_iter()
        .filter(|word| nice_part1(word))
        .count()
        .to_string())
}

pub fn part2(input: &str) -> Result<String, SolveError> {
    Ok(records(input)?
        .into_iter()
        .filter(|word| nice_part2(word))
        .count()
        .to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use svutils::scaffold_test_result;

    fn reference1(word: &[u8]) -> bool {
        let vowels = word.iter().filter(|byte| b"aeiou".contains(byte)).count();
        let mut double = false;
        let mut forbidden = false;
        for index in 1..word.len() {
            double |= word[index - 1] == word[index];
            for pair in [b"ab", b"cd", b"pq", b"xy"] {
                forbidden |= &word[index - 1..=index] == pair;
            }
        }
        vowels >= 3 && double && !forbidden
    }

    fn reference2(word: &[u8]) -> bool {
        let mut pair = false;
        let mut sandwich = false;
        for a in 0..word.len().saturating_sub(1) {
            for b in a + 2..word.len().saturating_sub(1) {
                pair |= word[a] == word[b] && word[a + 1] == word[b + 1];
            }
        }
        for index in 2..word.len() {
            sandwich |= word[index] == word[index - 2];
        }
        pair && sandwich
    }

    #[test]
    fn generated_words_match_independent_rules() {
        let alphabet = b"abe";
        for length in 1..=8 {
            for mut encoded in 0..3usize.pow(length) {
                let word: Vec<_> = (0..length)
                    .map(|_| {
                        let byte = alphabet[encoded % 3];
                        encoded /= 3;
                        byte
                    })
                    .collect();
                assert_eq!(nice_part1(&word), reference1(&word));
                assert_eq!(nice_part2(&word), reference2(&word));
            }
        }
    }

    #[test]
    fn overlap_and_independent_conditions() {
        assert!(!repeated_pair(b"zzz"));
        assert!(repeated_pair(b"zzzz"));
        assert!(!nice_part2(b"zzvwzz"));
        assert!(!nice_part2(b"zvz"));
        assert!(nice_part2(b"zzzz"));
        for pair in ["ab", "cd", "pq", "xy"] {
            assert!(!nice_part1(format!("uuuee{pair}").as_bytes()));
        }
        assert!(nice_part1(b"uueeff"));
        assert!(!nice_part1(b"ueueu"));
        assert!(!nice_part1(b"zz"));
    }

    #[test]
    fn records_validate_and_count_duplicates() {
        assert_eq!(part1("uueeff\r\nuueeff\r\n").unwrap(), "2");
        assert_eq!(part2("zzzz\r\nzzzz\r\n").unwrap(), "2");
        assert_eq!(part1("z").unwrap(), "0");
        assert_eq!(part2("z").unwrap(), "0");
        for solver in [part1, part2] {
            for input in ["", " \r\n"] {
                assert!(matches!(solver(input), Err(SolveError::EmptyInput)));
            }
            for invalid in ["", "a b", "ABC", "é", "a1", " z", "z "] {
                assert!(solver(&format!("zzzz\n{invalid}\n"))
                    .unwrap_err()
                    .to_string()
                    .starts_with("line 2:"));
            }
        }
    }

    #[test]
    fn test_part1_input() {
        scaffold_test_result(2015, 5, "input.txt", "input-spec.1.txt", part1);
    }

    #[test]
    fn test_part2_input() {
        scaffold_test_result(2015, 5, "input.txt", "input-spec.2.txt", part2);
    }
}
