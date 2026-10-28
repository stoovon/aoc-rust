use itertools::Itertools;
use svutils::SolveError;

fn card_strength(c: char, jokers_wild: bool) -> i64 {
    match c {
        'A' => 14,
        'K' => 13,
        'Q' => 12,
        'J' => {
            if jokers_wild {
                0
            } else {
                11
            }
        }
        'T' => 10,
        '9' => 9,
        '8' => 8,
        '7' => 7,
        '6' => 6,
        '5' => 5,
        '4' => 4,
        '3' => 3,
        '2' => 2,
        _ => unreachable!(),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum HandKind {
    HighCard,
    OnePair,
    TwoPair,
    ThreeOfAKind,
    FullHouse,
    FourOfAKind,
    FiveOfAKind,
}

fn hand_kind(counts: &[i64], jokers: i64) -> HandKind {
    match (*counts.iter().max().unwrap_or(&0), jokers) {
        (a, b) if a + b == 5 => HandKind::FiveOfAKind,
        (a, b) if a + b == 4 => HandKind::FourOfAKind,
        (3, 0) => {
            if counts.contains(&2) {
                HandKind::FullHouse
            } else {
                HandKind::ThreeOfAKind
            }
        }
        (2, _) => {
            let pairs = counts.iter().filter(|&&v| v == 2).count();
            match (pairs, jokers) {
                (2, 1) => HandKind::FullHouse,
                (1, 1) => HandKind::ThreeOfAKind,
                (2, 0) => HandKind::TwoPair,
                _ => HandKind::OnePair,
            }
        }
        (1, 2) => HandKind::ThreeOfAKind,
        (1, 1) => HandKind::OnePair,
        _ => HandKind::HighCard,
    }
}

fn hand_strength(cards: &str, jokers_wild: bool) -> (HandKind, i64) {
    let counts_by_card = cards.chars().counts();
    let counts = counts_by_card
        .iter()
        .filter(|&(&k, _)| k != 'J' || !jokers_wild)
        .map(|(_, &v)| v as i64)
        .collect::<Vec<_>>();
    let jokers = if jokers_wild {
        *counts_by_card.get(&'J').unwrap_or(&0) as i64
    } else {
        0
    };
    let tie_break = cards
        .chars()
        .fold(0, |acc, c| (acc << 4) + card_strength(c, jokers_wild));
    (hand_kind(&counts, jokers), tie_break)
}

struct Hand<'a> {
    cards: &'a str,
    bid: i64,
}

struct RankedHand {
    strength: (HandKind, i64),
    bid: i64,
}

impl Hand<'_> {
    fn ranked(&self, jokers_wild: bool) -> RankedHand {
        RankedHand {
            strength: hand_strength(self.cards, jokers_wild),
            bid: self.bid,
        }
    }
}

fn parse_hands(input: &str) -> Result<Vec<Hand<'_>>, SolveError> {
    if input.trim().is_empty() {
        return Err(SolveError::EmptyInput);
    }
    input
        .lines()
        .enumerate()
        .map(|(idx, line)| {
            let invalid = |message: &str| SolveError::InvalidInput(message.into()).at_line(idx + 1);
            let mut fields = line.split_whitespace();
            let cards = fields.next().ok_or_else(|| invalid("missing hand"))?;
            if cards.len() != 5 || !cards.bytes().all(|card| b"23456789TJQKA".contains(&card)) {
                return Err(invalid("hand must contain exactly five valid cards"));
            }
            let bid: i64 = fields
                .next()
                .ok_or_else(|| invalid("missing bid"))?
                .parse()
                .map_err(|error| SolveError::from(error).at_line(idx + 1))?;
            if bid < 0 {
                return Err(invalid("bid cannot be negative"));
            }
            if fields.next().is_some() {
                return Err(invalid("unexpected text after bid"));
            }
            Ok(Hand { cards, bid })
        })
        .collect()
}

fn total_winnings(mut hands: Vec<RankedHand>) -> Result<String, SolveError> {
    hands.sort_unstable_by_key(|hand| hand.strength);
    // Duplicate hands have no puzzle-defined ordering, so their winnings are ambiguous.
    if hands
        .windows(2)
        .any(|pair| pair[0].strength == pair[1].strength)
    {
        return Err(SolveError::InvalidInput("duplicate hand".into()));
    }
    let mut total = 0_i64;
    for (idx, hand) in hands.into_iter().enumerate() {
        let rank = i64::try_from(idx + 1).map_err(|_| SolveError::ArithmeticOverflow)?;
        let winnings = rank
            .checked_mul(hand.bid)
            .ok_or(SolveError::ArithmeticOverflow)?;
        total = total
            .checked_add(winnings)
            .ok_or(SolveError::ArithmeticOverflow)?;
    }
    Ok(total.to_string())
}

pub fn part1(input: &str) -> Result<String, SolveError> {
    let hands = parse_hands(input)?;
    let ranked = hands.iter().map(|hand| hand.ranked(false)).collect();
    total_winnings(ranked)
}

pub fn part2(input: &str) -> Result<String, SolveError> {
    let hands = parse_hands(input)?;
    let ranked = hands.iter().map(|hand| hand.ranked(true)).collect();
    total_winnings(ranked)
}

#[cfg(test)]
mod tests {
    use super::*;
    use svutils::scaffold_test_result;

    const YEAR: i16 = 2023;
    const DAY: i16 = 7;

    #[test]
    fn duplicate_hands_are_rejected_regardless_of_bid_or_input_position() {
        for input in ["AAAAA 1\nAAAAA 10", "AAAAA 1\n22222 2\nAAAAA 1"] {
            for solver in [part1, part2] {
                let error = solver(input).unwrap_err();
                assert!(matches!(error, SolveError::InvalidInput(_)));
                assert!(error.to_string().contains("duplicate hand"));
            }
        }
        let input = "AAAA2 1\n2AAAA 10";
        assert_eq!(part1(input).unwrap(), "12");
        assert_eq!(part2(input).unwrap(), "12");
    }

    #[test]
    fn ordinary_categories_are_named_and_ordered_weakest_first() {
        let cases = [
            ("23456", HandKind::HighCard),
            ("22345", HandKind::OnePair),
            ("22334", HandKind::TwoPair),
            ("22234", HandKind::ThreeOfAKind),
            ("22233", HandKind::FullHouse),
            ("22223", HandKind::FourOfAKind),
            ("22222", HandKind::FiveOfAKind),
        ];
        for (cards, expected) in cases {
            assert_eq!(hand_strength(cards, false).0, expected);
        }
        for pair in cases.windows(2) {
            assert!(hand_strength(pair[0].0, false) < hand_strength(pair[1].0, false));
        }
    }

    #[test]
    fn jokers_upgrade_categories_including_all_jokers() {
        for (cards, expected) in [
            ("J2345", HandKind::OnePair),
            ("JJ234", HandKind::ThreeOfAKind),
            ("J2233", HandKind::FullHouse),
            ("JJ223", HandKind::FourOfAKind),
            ("JJJ23", HandKind::FourOfAKind),
            ("JJJJ2", HandKind::FiveOfAKind),
            ("JJJJJ", HandKind::FiveOfAKind),
        ] {
            assert_eq!(hand_strength(cards, true).0, expected, "{cards}");
        }
    }

    #[test]
    fn equal_categories_compare_original_cards_left_to_right() {
        let lower = hand_strength("3AAAA", false);
        let higher = hand_strength("4444A", false);
        assert_eq!(lower.0, HandKind::FourOfAKind);
        assert_eq!(higher.0, HandKind::FourOfAKind);
        assert!(higher > lower);
        assert!(hand_strength("AAAA2", false) > hand_strength("2AAAA", false));
    }

    #[test]
    fn wild_jokers_upgrade_category_but_remain_low_in_tie_breaks() {
        let wild = hand_strength("JAAA3", true);
        let ordinary = hand_strength("KKKK3", true);
        assert_eq!(wild.0, HandKind::FourOfAKind);
        assert_eq!(ordinary.0, HandKind::FourOfAKind);
        assert!(wild < ordinary);
        let input = "JAAA3 10\nKKKK3 1";
        assert_eq!(part2(input).unwrap(), "12");
    }

    #[test]
    fn malformed_hands_and_bids_return_errors_from_both_parts() {
        for input in [
            "",
            " ",
            "AAAA 3",
            "AAAAAA 3",
            "AAXAA 3",
            "🂡AAA 3",
            "AAAAA",
            "AAAAA nope",
            "AAAAA -1",
            "AAAAA 3 extra",
            "AAAAA 3\n\n22222 5",
        ] {
            for solver in [part1, part2] {
                assert!(solver(input).is_err(), "{input}");
            }
        }
    }

    #[test]
    fn bids_are_not_limited_to_u8_and_whitespace_is_flexible() {
        let input = "AAAAA\t300\r\n22222   5\r\n";
        assert_eq!(part1(input).unwrap(), "605");
        assert_eq!(part2(input).unwrap(), "605");
    }

    #[test]
    fn selected_rules_produce_different_rankings() {
        let input = "JJJJ2 10\nQQQQ2 1";
        assert_eq!(part1(input).unwrap(), "12");
        assert_eq!(part2(input).unwrap(), "21");
    }

    #[test]
    fn winnings_report_product_and_total_overflow() {
        let input = format!("22222 0\nAAAAA {}", i64::MAX);
        assert!(matches!(part1(&input), Err(SolveError::ArithmeticOverflow)));
        let input = format!("22222 {}\nAAAAA 1", i64::MAX);
        assert!(matches!(part2(&input), Err(SolveError::ArithmeticOverflow)));
    }

    #[test]
    fn trailing_newline_does_not_create_an_empty_hand() {
        let input = "AAAAA 3\n22222 5";
        for solver in [part1, part2] {
            assert_eq!(solver(input).unwrap(), "11");
            assert_eq!(solver(&format!("{input}\n")).unwrap(), "11");
        }
        assert_eq!(parse_hands("AAAAA 3\n").unwrap().len(), 1);
    }

    #[test]
    fn test_part1_input() {
        scaffold_test_result(YEAR, DAY, "input.txt", "input-spec.1.txt", part1);
    }

    #[test]
    fn test_part2_input() {
        scaffold_test_result(YEAR, DAY, "input.txt", "input-spec.2.txt", part2);
    }

    #[test]
    fn malformed_number_retains_its_line_and_source() {
        use std::error::Error;
        let error = part1("AAAAA 1\n22222 nope").unwrap_err();
        assert!(matches!(error, SolveError::AtLine { line: 2, .. }));
        let source = error
            .source()
            .unwrap()
            .downcast_ref::<SolveError>()
            .unwrap();
        assert!(matches!(source, SolveError::InvalidNumber(_)));
        assert!(source.source().is_some());
    }
}
