use std::collections::HashMap;
use svutils::SolveError;

#[derive(Debug, Default)]
struct CubeDraw {
    red: i64,
    green: i64,
    blue: i64,
}

const AVAILABLE: CubeDraw = CubeDraw {
    red: 12,
    green: 13,
    blue: 14,
};

impl CubeDraw {
    fn fits_within(&self, available: &Self) -> bool {
        self.red <= available.red && self.green <= available.green && self.blue <= available.blue
    }

    fn power(&self) -> Result<i64, SolveError> {
        if self.red == 0 || self.green == 0 || self.blue == 0 {
            return Ok(0);
        }
        self.red
            .checked_mul(self.green)
            .and_then(|product| product.checked_mul(self.blue))
            .ok_or(SolveError::ArithmeticOverflow)
    }

    fn new(value: &str) -> Result<Self, SolveError> {
        let mut draw = CubeDraw::default();
        for part in value.split(',') {
            let mut split = part.split_whitespace();
            let count: i64 = split
                .next()
                .ok_or_else(|| SolveError::InvalidInput("missing cube count".into()))?
                .parse()?;
            if count < 0 {
                return Err(SolveError::InvalidInput(
                    "cube count cannot be negative".into(),
                ));
            }
            let color = split
                .next()
                .ok_or_else(|| SolveError::InvalidInput("missing cube colour".into()))?;
            if split.next().is_some() {
                return Err(SolveError::InvalidInput(format!(
                    "unexpected text in draw: {part}"
                )));
            }
            let target = match color {
                "red" => &mut draw.red,
                "green" => &mut draw.green,
                "blue" => &mut draw.blue,
                _ => {
                    return Err(SolveError::InvalidInput(format!(
                        "unknown cube colour: {color}"
                    )))
                }
            };
            *target = (*target)
                .checked_add(count)
                .ok_or(SolveError::ArithmeticOverflow)?;
        }

        Ok(draw)
    }
}

fn parse_games(input: &str) -> Result<HashMap<i64, Vec<CubeDraw>>, SolveError> {
    if input.is_empty() {
        return Err(SolveError::EmptyInput);
    }
    let mut games = HashMap::new();
    for (index, line) in input.lines().enumerate() {
        let invalid =
            |message: &str| SolveError::InvalidInput(format!("line {}: {message}", index + 1));
        let (header, draws) = line
            .split_once(':')
            .ok_or_else(|| invalid("missing ':' after game header"))?;
        let id: i64 = header
            .strip_prefix("Game ")
            .ok_or_else(|| invalid("expected 'Game ' before game ID"))?
            .trim()
            .parse()?;
        if id <= 0 {
            return Err(invalid("game ID must be positive"));
        }
        let draws = draws
            .split(';')
            .map(CubeDraw::new)
            .collect::<Result<Vec<_>, _>>()?;
        if games.insert(id, draws).is_some() {
            return Err(invalid("duplicate game ID"));
        }
    }
    Ok(games)
}

pub fn part1(input: &str) -> Result<String, SolveError> {
    let games = parse_games(input)?;
    let mut total = 0_i64;
    for (id, draws) in &games {
        if draws.iter().all(|draw| draw.fits_within(&AVAILABLE)) {
            total = total
                .checked_add(*id)
                .ok_or(SolveError::ArithmeticOverflow)?;
        }
    }
    Ok(total.to_string())
}

pub fn part2(input: &str) -> Result<String, SolveError> {
    let games = parse_games(input)?;
    let mut total = 0_i64;
    for draws in games.values() {
        let mut minimum = CubeDraw::default();
        for draw in draws {
            minimum.red = minimum.red.max(draw.red);
            minimum.green = minimum.green.max(draw.green);
            minimum.blue = minimum.blue.max(draw.blue);
        }
        total = total
            .checked_add(minimum.power()?)
            .ok_or(SolveError::ArithmeticOverflow)?;
    }
    Ok(total.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use svutils::scaffold_test_result as scaffold_test;

    const YEAR: i16 = 2023;
    const DAY: i16 = 2;

    #[test]
    fn bag_limits_are_inclusive_and_apply_to_every_draw() {
        let input = "Game 7: 12 red, 13 green, 14 blue; 1 red\nGame 8: 1 red; 15 blue";
        assert_eq!(part1(input).unwrap(), "7");
    }

    #[test]
    fn minimum_bag_uses_maxima_across_draws_not_sums() {
        assert_eq!(
            part2("Game 1: 4 red, 2 green; 6 red, 3 blue").unwrap(),
            "36"
        );
    }

    #[test]
    fn absent_colour_makes_power_zero() {
        assert_eq!(part2("Game 1: 4 red; 2 green").unwrap(), "0");
        let input = format!("Game 1: {} red, {} green", i64::MAX, i64::MAX);
        assert_eq!(part2(&input).unwrap(), "0");
    }

    #[test]
    fn arithmetic_overflow_returns_an_error() {
        let input = format!("Game 1: {} red, 2 green, 1 blue", i64::MAX);
        assert!(matches!(part2(&input), Err(SolveError::ArithmeticOverflow)));
        let input = format!("Game {}: 1 red\nGame 1: 1 blue", i64::MAX);
        assert!(matches!(part1(&input), Err(SolveError::ArithmeticOverflow)));
        let input = format!("Game 1: {} red, 1 red", i64::MAX);
        assert!(matches!(part1(&input), Err(SolveError::ArithmeticOverflow)));
    }

    #[test]
    fn malformed_game_headers_and_empty_documents_are_rejected() {
        for input in [
            "",
            "Game 1 3 blue",
            "Round 1: 3 blue",
            "Game nope: 3 blue",
            "Game 0: 3 blue",
            "Game 1: 3 blue\nGame 1: 2 red",
        ] {
            assert!(part1(input).is_err(), "{input}");
            assert!(part2(input).is_err(), "{input}");
        }
    }

    #[test]
    fn invalid_draws_are_rejected_by_both_parts() {
        for draw in ["three blue", "3 purple", "-1 red", "3", "3 blue extra", ""] {
            let input = format!("Game 1: {draw}");
            assert!(part1(&input).is_err(), "{input}");
            assert!(part2(&input).is_err(), "{input}");
        }
    }

    #[test]
    fn test_part1_input() {
        scaffold_test(YEAR, DAY, "input.txt", "input-spec.1.txt", part1);
    }

    #[test]
    fn test_part2_input() {
        scaffold_test(YEAR, DAY, "input.txt", "input-spec.2.txt", part2);
    }
}
