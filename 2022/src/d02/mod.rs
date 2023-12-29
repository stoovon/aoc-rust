extern crate core;

use std::fmt::Debug;
use std::str::FromStr;

////

// Could then move to a library. would like to explore this idea further.

pub fn lines_to_vec<R>(input: impl AsRef<str>) -> Result<Vec<R>, R::Err>
where
    R: FromStr,
    <R as FromStr>::Err: Debug,
{
    return input
        .as_ref()
        .lines()
        .map(|line| line.parse::<R>())
        .collect::<Result<_, _>>();
}

////

#[derive(Debug)]
enum Error {
    ParseError(String),
}

#[derive(Debug)]
struct Game {
    opponent: i64,
    player: i64,
}

impl std::str::FromStr for Game {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (opponent, player) = s
            .split_once(' ')
            .ok_or_else(|| Error::ParseError(s.to_owned()))?;

        let opponent: char = opponent
            .chars()
            .next()
            .ok_or_else(|| Error::ParseError(opponent.to_owned()))?;
        let player: char = player
            .chars()
            .next()
            .ok_or_else(|| Error::ParseError(player.to_owned()))?;

        return Ok(Game {
            opponent: opponent as i64 - 'A' as i64 + 1,
            player: player as i64 - 'X' as i64 + 1,
        });
    }
}

fn parse_input(input: &str) -> Vec<Game> {
    lines_to_vec::<Game>(input).unwrap()
}

fn playing_exactly_per_guide(play: Game) -> i64 {
    if (play.opponent == 3 && play.player == 1) || (play.opponent + 1 == play.player) {
        // 6 for a win, plus 1 for rock, 2 for paper, 3 for scissors
        return 6 + play.player as i64;
    } else if play.opponent == play.player {
        // 3 for a draw, plus 1 for rock, 2 for paper, 3 for scissors
        return 3 + play.player as i64;
    } else {
        // 0 for a loss, plus 1 for rock, 2 for paper, 3 for scissors
        return play.player as i64;
    }
}

pub fn fn1(input: &str) -> i64 {
    let input = parse_input(input);
    input
        .into_iter()
        .map(playing_exactly_per_guide)
        .sum::<i64>()
}

fn playing_to_get_result(play: Game) -> i64 {
    if play.player == 1 {
        // Playing to win so lag
        // Scissors (1-1) vs Rock (A)
        // Rock (2-1) vs Paper (B)
        // Paper (3-1) vs Scissors (C)
        return ((play.opponent as i64 - 2).rem_euclid(3) + 1) as i64;
    } else if play.player == 2 {
        // Playing to draw so match
        // Rock (1) vs Rock (A)
        // Paper (2) vs Paper (B)
        // Scissors (3) vs Scissors (C)
        return 3 + play.opponent as i64;
    } else {
        // Playing to win so lead
        // Paper (1+1) vs Rock (A)
        // Scissors (2+1) vs Paper (B)
        // Rock (3%3 + 1) vs Scissors (C)
        return 6 + (play.opponent as i64 % 3) + 1;
    }
}

pub fn fn2(input: &str) -> i64 {
    let input = parse_input(input);
    input.into_iter().map(playing_to_get_result).sum::<i64>()
}

#[cfg(test)]
mod tests {
    use super::*;
    use svutils::scaffold_test;

    const YEAR: i16 = 2022;
    const DAY: i16 = 2;

    #[test]
    fn test_fn1_example() {
        scaffold_test(YEAR, DAY, "example.txt", "example-spec.1.txt", fn1);
    }

    #[test]
    fn test_fn1_input() {
        scaffold_test(YEAR, DAY, "input.txt", "input-spec.1.txt", fn1);
    }

    #[test]
    fn test_fn2_example() {
        scaffold_test(YEAR, DAY, "example.txt", "example-spec.2.txt", fn2);
    }

    #[test]
    fn test_fn2_input() {
        scaffold_test(YEAR, DAY, "input.txt", "input-spec.2.txt", fn2);
    }
}
