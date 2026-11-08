use std::str::FromStr;
use svutils::SolveError;

struct Coordinates {
    row: u64,
    column: u64,
}

impl FromStr for Coordinates {
    type Err = SolveError;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let input = input.trim();
        if input.is_empty() {
            return Err(SolveError::EmptyInput);
        }
        if input.matches("row ").count() != 1 {
            return Err(SolveError::InvalidInput(
                "expected one row and column pair".into(),
            ));
        }
        let (_, coordinates) = input.split_once("row ").ok_or_else(|| {
            SolveError::InvalidInput("expected row and column coordinates".into())
        })?;
        let (row, column) = coordinates.split_once(", column ").ok_or_else(|| {
            SolveError::InvalidInput("expected 'row number, column number'".into())
        })?;
        let column = column.strip_suffix('.').unwrap_or(column);
        let positive = |text: &str| -> Result<u64, SolveError> {
            if text.is_empty() || !text.bytes().all(|byte| byte.is_ascii_digit()) {
                return Err(SolveError::InvalidInput(
                    "coordinates must be positive decimal integers".into(),
                ));
            }
            let value: u64 = text.parse()?;
            if value == 0 {
                return Err(SolveError::InvalidInput(
                    "coordinates must be positive".into(),
                ));
            }
            Ok(value)
        };
        Ok(Self {
            row: positive(row)?,
            column: positive(column)?,
        })
    }
}

impl Coordinates {
    fn position(&self) -> Result<u64, SolveError> {
        let diagonal = (self.row - 1)
            .checked_add(self.column - 1)
            .ok_or(SolveError::ArithmeticOverflow)?;
        let diagonal = u128::from(diagonal);
        let position = diagonal * (diagonal + 1) / 2 + u128::from(self.column);
        u64::try_from(position).map_err(|_| SolveError::ArithmeticOverflow)
    }
}

const SEED: u64 = 20151125;
const MULTIPLIER: u64 = 252533;
const MODULUS: u64 = 33554393;

fn num_to_code(num: u64) -> u64 {
    let mut code = SEED;
    let mut power = MULTIPLIER;
    let mut exponent = num - 1;
    while exponent > 0 {
        if exponent % 2 == 1 {
            code = code * power % MODULUS;
        }
        power = power * power % MODULUS;
        exponent /= 2;
    }
    code
}

pub fn part1(input: &str) -> Result<String, SolveError> {
    let coordinates: Coordinates = input.parse()?;
    Ok(num_to_code(coordinates.position()?).to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use svutils::scaffold_test_result;

    const YEAR: i16 = 2015;
    const DAY: i16 = 25;

    #[test]
    fn squared_powers_match_sequential_generation() {
        let mut expected = SEED;
        for position in 1..=20_000 {
            assert_eq!(num_to_code(position), expected);
            expected = expected * MULTIPLIER % MODULUS;
        }
    }

    #[test]
    fn large_positions_obey_the_single_step_recurrence() {
        for position in [1_000_003, 1_000_000_007, u64::MAX - 1] {
            assert_eq!(
                num_to_code(position + 1),
                num_to_code(position) * MULTIPLIER % MODULUS
            );
        }
    }

    #[test]
    fn coordinates_parse_once_from_a_phrase_or_message() {
        for input in [
            "row 17, column 9",
            " Calibration target: row 17, column 9.\r\n",
        ] {
            let coordinates: Coordinates = input.parse().unwrap();
            assert_eq!((coordinates.row, coordinates.column), (17, 9));
            assert_eq!(coordinates.position().unwrap(), 309);
            assert_eq!(part1(input).unwrap(), num_to_code(309).to_string());
        }
    }

    #[test]
    fn diagonal_positions_match_a_coordinate_walk() {
        let mut position = 0;
        for diagonal in 1..=40 {
            for column in 1..=diagonal {
                let row = diagonal + 1 - column;
                position += 1;
                assert_eq!(Coordinates { row, column }.position().unwrap(), position);
            }
        }
    }

    #[test]
    fn malformed_coordinates_and_position_overflow_return_errors() {
        for input in [
            "",
            "missing coordinates",
            "row 0, column 9",
            "row 17, column 0",
            "row -17, column 9",
            "row +17, column 9",
            "row 17.5, column 9",
            "row １７, column 9",
            "row 17 column 9",
            "row 17, column 9 trailing",
            "row 17, column 9.5",
            "row 17, column 9..",
            "row 17, column ",
            "row 17, column 9; row 18, column 10",
            "row 18446744073709551616, column 9",
        ] {
            assert!(part1(input).is_err(), "{input:?}");
        }
        for (row, column) in [(u64::MAX, 1), (u64::MAX, u64::MAX), (6_100_000_000, 1)] {
            let input = format!("row {row}, column {column}");
            assert!(matches!(part1(&input), Err(SolveError::ArithmeticOverflow)));
        }
        assert!(Coordinates {
            row: 6_000_000_000,
            column: 1
        }
        .position()
        .is_ok());
    }

    #[test]
    fn test_fn1_input() {
        scaffold_test_result(YEAR, DAY, "input.txt", "input-spec.1.txt", part1);
    }
}
