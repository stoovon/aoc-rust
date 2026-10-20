use svutils::SolveError;

const DIGIT_WORDS: [(&str, i64); 9] = [
    ("one", 1),
    ("two", 2),
    ("three", 3),
    ("four", 4),
    ("five", 5),
    ("six", 6),
    ("seven", 7),
    ("eight", 8),
    ("nine", 9),
];

fn recognise_numeric_digit(suffix: &str) -> Option<i64> {
    suffix.chars().next()?.to_digit(10).map(i64::from)
}

fn recognise_word_or_digit(suffix: &str) -> Option<i64> {
    recognise_numeric_digit(suffix).or_else(|| {
        DIGIT_WORDS
            .iter()
            .find_map(|(text, value)| suffix.starts_with(text).then_some(*value))
    })
}

fn solve(input: &str, recognise: impl Fn(&str) -> Option<i64>) -> Result<String, SolveError> {
    if input.is_empty() {
        return Err(SolveError::EmptyInput);
    }
    let mut total = 0_i64;
    for (index, line) in input.lines().enumerate() {
        let mut digits = line
            .char_indices()
            .filter_map(|(offset, _)| recognise(&line[offset..]));
        let first = digits
            .next()
            .ok_or(SolveError::MissingDigit { line: index + 1 })?;
        let last = digits.next_back().unwrap_or(first);
        let value = 10 * first + last;
        total = total
            .checked_add(value)
            .ok_or(SolveError::ArithmeticOverflow)?;
    }
    Ok(total.to_string())
}

pub fn part1(input: &str) -> Result<String, SolveError> {
    solve(input, recognise_numeric_digit)
}

pub fn part2(input: &str) -> Result<String, SolveError> {
    solve(input, recognise_word_or_digit)
}

#[cfg(test)]
mod tests {
    use super::*;
    use svutils::scaffold_test_result;

    const YEAR: i16 = 2023;
    const DAY: i16 = 1;

    #[test]
    fn empty_document_is_rejected() {
        assert!(matches!(part1(""), Err(SolveError::EmptyInput)));
        assert!(matches!(part2(""), Err(SolveError::EmptyInput)));
    }

    #[test]
    fn words_and_numeric_digits_are_combined_only_in_part_two() {
        assert_eq!(part1("two7nine").unwrap(), "77");
        assert_eq!(part2("two7nine").unwrap(), "29");
        assert_eq!(part2("7two8").unwrap(), "78");
    }

    #[test]
    fn numeric_zero_is_a_digit_but_the_word_zero_is_not() {
        assert_eq!(part1("0").unwrap(), "0");
        assert_eq!(part2("0").unwrap(), "0");
        assert_eq!(part2("zero7").unwrap(), "77");
        assert!(matches!(
            part2("zero"),
            Err(SolveError::MissingDigit { line: 1 })
        ));
    }

    #[test]
    fn overlapping_words_remain_visible() {
        assert_eq!(part2("twone").unwrap(), "21");
        assert_eq!(part2("oneight").unwrap(), "18");
    }

    #[test]
    fn word_matching_respects_utf8_boundaries() {
        assert_eq!(part2("éone🦀two").unwrap(), "12");
    }

    #[test]
    fn single_word_supplies_both_places() {
        assert_eq!(part2("seven").unwrap(), "77");
    }

    #[test]
    fn part_two_reports_missing_digits() {
        assert!(matches!(
            part2("one\nletters"),
            Err(SolveError::MissingDigit { line: 2 })
        ));
    }

    #[test]
    fn one_digit_supplies_both_places() {
        assert_eq!(part1("a7b").unwrap(), "77");
    }

    #[test]
    fn uses_first_and_last_digits() {
        assert_eq!(part1("a3b1c8d").unwrap(), "38");
    }

    #[test]
    fn missing_digit_reports_its_line() {
        assert!(matches!(
            part1("a7b\nletters"),
            Err(SolveError::MissingDigit { line: 2 })
        ));
    }

    #[test]
    fn test_part1_example() {
        scaffold_test_result(YEAR, DAY, "example.txt", "example-spec.1.txt", part1);
    }

    #[test]
    fn test_part1_input() {
        scaffold_test_result(YEAR, DAY, "input.txt", "input-spec.1.txt", part1);
    }

    #[test]
    fn test_part2_input() {
        scaffold_test_result(YEAR, DAY, "input.txt", "input-spec.2.txt", part2);
    }
}
