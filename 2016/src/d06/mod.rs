use std::cmp::Reverse;
use svutils::SolveError;

fn histograms(input: &str) -> Result<Vec<[usize; 26]>, SolveError> {
    let width = input
        .lines()
        .next()
        .filter(|line| !line.is_empty())
        .ok_or(SolveError::EmptyInput)?
        .len();
    let mut counts = vec![[0; 26]; width];

    for (index, line) in input.lines().enumerate() {
        if line.len() != width {
            return Err(
                SolveError::InvalidInput("inconsistent row width".into()).at_line(index + 1)
            );
        }
        for (column, letter) in counts.iter_mut().zip(line.bytes()) {
            if !letter.is_ascii_lowercase() {
                return Err(
                    SolveError::InvalidInput("expected lowercase ASCII letters".into())
                        .at_line(index + 1),
                );
            }
            column[usize::from(letter - b'a')] += 1;
        }
    }
    Ok(counts)
}

fn decode<K: Ord>(input: &str, rank: impl Fn(usize) -> K) -> Result<String, SolveError> {
    let counts = histograms(input)?;
    Ok(counts
        .iter()
        .map(|column| {
            let (letter, _) = column
                .iter()
                .enumerate()
                .filter(|&(_, &count)| count != 0)
                .min_by_key(|&(_, &count)| rank(count))
                .expect("each column has at least one letter");
            char::from(b'a' + letter as u8)
        })
        .collect())
}

pub fn part1(input: &str) -> Result<String, SolveError> {
    decode(input, Reverse)
}

pub fn part2(input: &str) -> Result<String, SolveError> {
    decode(input, |count| count)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn selects_present_letters_and_breaks_ties_alphabetically() {
        let input = "yb\r\nza\r\nyc\r\nzb";
        assert_eq!(part1(input).unwrap(), "yb");
        assert_eq!(part2(input).unwrap(), "ya");
    }

    #[test]
    fn rejects_invalid_rows() {
        for solve in [part1, part2] {
            assert!(matches!(solve(""), Err(SolveError::EmptyInput)));
            for input in ["ab\na", "ab\nabc", "ab\n\n", "ab\nAa", "ab\né"] {
                assert!(matches!(
                    solve(input),
                    Err(SolveError::AtLine { line: 2, .. })
                ));
            }
        }
    }
}
