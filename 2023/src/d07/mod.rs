extern crate core;

use itertools::Itertools;

fn card_strength(c: char, p2: bool) -> i64 {
    match c {
        'A' => 14,
        'K' => 13,
        'Q' => 12,
        'J' => {
            if p2 {
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

fn hand_kind(counts: &[i64], jokers: i64) -> i64 {
    match (*counts.iter().max().unwrap_or(&0), jokers) {
        // Also covers JJJJJ
        (a, b) if a + b == 5 => 6,
        (a, b) if a + b == 4 => 5,
        (3, 0) => {
            if counts.contains(&2) {
                4
            } else {
                3
            }
        }
        (2, _) => {
            let pairs = counts.iter().filter(|&&v| v == 2).count();
            match (pairs, jokers) {
                // Power up
                (2, 1) => 4,
                (1, 1) => 3,
                (2, 0) => 2,
                _ => 1,
            }
        }
        (1, 2) => 3,
        (1, 1) => 1,
        _ => 0,
    }
}

fn hand_strength(cards: &str, p2: bool) -> (i64, i64) {
    let counts_by_card = cards.chars().counts();
    let counts = counts_by_card
        .iter()
        .filter(|&(&k, _)| k != 'J' || !p2)
        .map(|(_, &v)| v as i64)
        .collect::<Vec<_>>();
    let jokers = if p2 {
        *counts_by_card.get(&'J').unwrap_or(&0) as i64
    } else {
        0
    };
    let idx = cards
        .chars()
        .fold(0, |acc, c| (acc << 4) + card_strength(c, p2));
    (hand_kind(&counts, jokers), idx)
}

pub fn parse(input: &str) -> Vec<(&str, u8, (i64, i64), (i64, i64))> {
    input
        .split('\n')
        .map(|l| {
            let (cards, bid) = l.split_once(' ').unwrap();
            let p1key = hand_strength(cards, false);
            let p2key = hand_strength(cards, true);
            (cards, bid.parse::<u8>().unwrap(), p1key, p2key)
        })
        .collect_vec()
}

pub fn fn1(input: &str) -> i64 {
    let mut cards = input.split('\n').map(|l| {
        let (cards, bid) = l.split_once(' ').unwrap();
        let p1key = hand_strength(cards, false);
        let p2key = hand_strength(cards, true);
        (cards, bid.parse().unwrap(), p1key, p2key)
      }).collect::<Vec<_>>();
      cards.sort_unstable_by_key(|&(_,_,key,_)| key);

    let result: usize = cards.iter().enumerate().map(|(i, (_,bid,_,_))| (i + 1) * bid).sum();
    result as i64
}

pub fn fn2(input: &str) -> i64 {
    let mut cards = input.split('\n').map(|l| {
        let (cards, bid) = l.split_once(' ').unwrap();
        let p1key = hand_strength(cards, false);
        let p2key = hand_strength(cards, true);
        (cards, bid.parse().unwrap(), p1key, p2key)
      }).collect::<Vec<_>>();

      // PAY CLOSE ATTENTION TO THE THREE UNDERBARS; it's two for fn1
      cards.sort_unstable_by_key(|&(_,_,_,key)| key);

    let result: usize = cards.iter().enumerate().map(|(i, (_,bid,_,_))| (i + 1) * bid).sum();
    result as i64
}

#[cfg(test)]
mod tests {
    use super::*;
    use itertools::Itertools;
    use svutils::load_spec;

    #[test]
    fn test_fn1_example() {
        assert_eq!(
            fn1(include_str!("example.txt")), load_spec(include_str!("example-spec.1.txt"))
        );
    }

    #[test]
    fn test_fn1_input() {
        assert_eq!(
            fn1(include_str!("input.txt")), load_spec(include_str!("input-spec.1.txt"))
        );
    }

    #[test]
    fn test_fn2_example() {
        assert_eq!(
            fn2(include_str!("example.txt")), load_spec(include_str!("example-spec.2.txt"))
        );
    }

    #[test]
    fn test_fn2_input() {
        assert_eq!(
            fn2(include_str!("input.txt")), load_spec(include_str!("input-spec.2.txt"))
        );
    }
}
