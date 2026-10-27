use svutils::SolveError;

struct Card {
    winning: Vec<u32>,
    held: Vec<u32>,
}

impl Card {
    fn matches(&self) -> usize {
        self.held
            .iter()
            .filter(|num| self.winning.contains(num))
            .count()
    }
}

fn parse_card(input: &str) -> Result<Card, SolveError> {
    let (header, card_part) = input
        .split_once(':')
        .ok_or_else(|| SolveError::InvalidInput("missing ':' after card header".into()))?;
    let mut header = header.split_whitespace();
    if header.next() != Some("Card") {
        return Err(SolveError::InvalidInput(
            "expected 'Card' before card ID".into(),
        ));
    }
    let id: u32 = header
        .next()
        .ok_or_else(|| SolveError::InvalidInput("missing card ID".into()))?
        .parse()?;
    if id == 0 {
        return Err(SolveError::InvalidInput("card ID must be positive".into()));
    }
    if header.next().is_some() {
        return Err(SolveError::InvalidInput(
            "unexpected text after card ID".into(),
        ));
    }
    let (winning_part, held_part) = card_part
        .split_once('|')
        .ok_or_else(|| SolveError::InvalidInput("missing '|' between number lists".into()))?;
    let winning = parse_numbers(winning_part)?;
    let held = parse_numbers(held_part)?;
    Ok(Card { winning, held })
}

fn parse_numbers(input: &str) -> Result<Vec<u32>, SolveError> {
    let numbers = input
        .split_whitespace()
        .map(|s| s.parse::<u32>())
        .collect::<Result<Vec<_>, _>>()?;
    for (idx, number) in numbers.iter().enumerate() {
        if numbers[..idx].contains(number) {
            return Err(SolveError::InvalidInput(format!(
                "duplicate number {number} in card number list"
            )));
        }
    }
    Ok(numbers)
}

fn parse_all(input: &str) -> Result<Vec<Card>, SolveError> {
    if input.trim().is_empty() {
        return Err(SolveError::EmptyInput);
    }
    input.lines().map(parse_card).collect()
}

pub fn part1(input: &str) -> Result<String, SolveError> {
    let cards = parse_all(input)?;
    let mut total = 0_i64;
    for card in &cards {
        let numbers_won = card.matches();
        if numbers_won > 0 {
            let exponent =
                u32::try_from(numbers_won - 1).map_err(|_| SolveError::ArithmeticOverflow)?;
            let points = 2_i64
                .checked_pow(exponent)
                .ok_or(SolveError::ArithmeticOverflow)?;
            total = total
                .checked_add(points)
                .ok_or(SolveError::ArithmeticOverflow)?;
        }
    }
    Ok(total.to_string())
}

pub fn part2(input: &str) -> Result<String, SolveError> {
    let cards = parse_all(input)?;
    let mut counts = vec![1_i64; cards.len()];
    for (idx, card) in cards.iter().enumerate() {
        let numbers_won = card.matches();
        let remaining = cards.len() - idx - 1;
        if numbers_won > remaining {
            return Err(SolveError::InvalidInput(format!(
                "card at position {} awards copies beyond the available cards",
                idx + 1
            )));
        }
        let n = counts[idx];
        for i in 0..numbers_won {
            counts[idx + i + 1] = counts[idx + i + 1]
                .checked_add(n)
                .ok_or(SolveError::ArithmeticOverflow)?;
        }
    }
    let total = counts.into_iter().try_fold(0_i64, |total, count| {
        total
            .checked_add(count)
            .ok_or(SolveError::ArithmeticOverflow)
    })?;
    Ok(total.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use svutils::scaffold_test_result;

    const YEAR: i16 = 2023;
    const DAY: i16 = 4;

    fn cards_awarding_all_remaining(card_count: usize) -> String {
        (0..card_count)
            .map(|idx| {
                let matches = (1..card_count - idx)
                    .map(|n| n.to_string())
                    .collect::<Vec<_>>()
                    .join(" ");
                format!("Card {}: {matches} | {matches}", idx + 1)
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn card_with_matches(count: usize) -> String {
        let numbers = (1..=count)
            .map(|n| n.to_string())
            .collect::<Vec<_>>()
            .join(" ");
        if count == 0 {
            "Card 1: 1 | 2".into()
        } else {
            format!("Card 1: {numbers} | {numbers}")
        }
    }

    #[test]
    fn points_double_after_the_first_match() {
        for (matches, expected) in [(0, "0"), (1, "1"), (2, "2"), (3, "4"), (4, "8")] {
            assert_eq!(part1(&card_with_matches(matches)).unwrap(), expected);
        }
    }

    #[test]
    fn points_and_total_report_overflow() {
        let largest = card_with_matches(63);
        assert_eq!(part1(&largest).unwrap(), (1_i64 << 62).to_string());
        for matches in [64, 65] {
            assert!(matches!(
                part1(&card_with_matches(matches)),
                Err(SolveError::ArithmeticOverflow)
            ));
        }
        let input = format!("{largest}\n{largest}");
        assert!(matches!(part1(&input), Err(SolveError::ArithmeticOverflow)));
    }

    #[test]
    fn copy_counts_and_total_report_overflow() {
        assert_eq!(
            part2(&cards_awarding_all_remaining(63)).unwrap(),
            i64::MAX.to_string()
        );
        // 64 cards overflow the total; 65 also overflow an individual count.
        for card_count in [64, 65] {
            assert!(matches!(
                part2(&cards_awarding_all_remaining(card_count)),
                Err(SolveError::ArithmeticOverflow)
            ));
        }
    }

    #[test]
    fn duplicate_numbers_within_either_list_are_rejected() {
        for input in ["Card 1: 7 7 | 8", "Card 1: 7 | 7 7"] {
            assert!(matches!(part1(input), Err(SolveError::InvalidInput(_))));
            assert!(matches!(part2(input), Err(SolveError::InvalidInput(_))));
        }
        let input = "Card 1: 7 | 7\nCard 2: 7 | 8";
        assert_eq!(part1(input).unwrap(), "1");
        assert_eq!(part2(input).unwrap(), "3");
    }

    #[test]
    fn malformed_headers_are_rejected_by_both_parts() {
        for header in [
            "Banana 1",
            "Card",
            "Card 0",
            "Card -1",
            "Card nope",
            "Card 1 extra",
            "Card1",
        ] {
            let input = format!("{header}: 1 | 2");
            assert!(part1(&input).is_err(), "{header}");
            assert!(part2(&input).is_err(), "{header}");
        }
    }

    #[test]
    fn header_spacing_is_flexible_and_ids_do_not_select_award_positions() {
        let input = "Card   9: 1 | 1\nCard\t4: 2 | 3";
        assert_eq!(part1(input).unwrap(), "1");
        assert_eq!(part2(input).unwrap(), "3");
    }

    #[test]
    fn empty_documents_are_rejected_by_both_parts() {
        for input in ["", " ", "\n\t\n"] {
            assert!(matches!(part1(input), Err(SolveError::EmptyInput)));
            assert!(matches!(part2(input), Err(SolveError::EmptyInput)));
        }
    }

    #[test]
    fn malformed_numbers_propagate_from_both_lists_and_both_parts() {
        for input in ["Card 1: nope | 1", "Card 1: 1 | nope"] {
            assert!(matches!(part1(input), Err(SolveError::InvalidNumber(_))));
            assert!(matches!(part2(input), Err(SolveError::InvalidNumber(_))));
        }
    }

    #[test]
    fn missing_separators_return_errors() {
        for input in ["Card 1 1 | 2", "Card 1: 1 2"] {
            assert!(matches!(part1(input), Err(SolveError::InvalidInput(_))));
            assert!(matches!(part2(input), Err(SolveError::InvalidInput(_))));
        }
    }

    #[test]
    fn copies_accumulate_across_cards() {
        let input = "Card 1: 1 2 | 1 2\nCard 2: 3 | 3\nCard 3: 4 | 5";
        assert_eq!(part2(input).unwrap(), "7");
    }

    #[test]
    fn awards_beyond_available_cards_are_rejected() {
        for input in ["Card 1: 1 | 1", "Card 1: 1 2 | 1 2\nCard 2: 3 | 4"] {
            assert!(matches!(part2(input), Err(SolveError::InvalidInput(_))));
        }
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
