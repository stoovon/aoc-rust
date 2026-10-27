use std::collections::{HashMap, HashSet};
use svutils::SolveError;

#[derive(Debug, Eq, PartialEq, Hash, Clone, Copy)]
struct Coords {
    x: i64,
    y: i64,
}

#[derive(Debug, Eq, PartialEq, Hash, Clone, Copy)]
struct Symbol {
    position: Coords,
    character: char,
}

#[derive(Debug)]
struct Number {
    digit_cells: Vec<Coords>,
    value: i64,
}

#[derive(Debug)]
struct Schematic {
    symbols: HashMap<Coords, char>,
    numbers: Vec<Number>,
}

fn neighboring_symbols<'a>(
    number: &'a Number,
    symbols: &'a HashMap<Coords, char>,
) -> impl Iterator<Item = Symbol> + 'a {
    number.digit_cells.iter().flat_map(move |cell| {
        (cell.x - 1..=cell.x + 1).flat_map(move |x| {
            (cell.y - 1..=cell.y + 1).filter_map(move |y| {
                let position = Coords { x, y };
                symbols.get(&position).map(|&character| Symbol {
                    position,
                    character,
                })
            })
        })
    })
}

fn unique_neighboring_symbols<'a>(
    number: &'a Number,
    symbols: &'a HashMap<Coords, char>,
) -> impl Iterator<Item = Symbol> + 'a {
    let mut seen = HashSet::new();
    neighboring_symbols(number, symbols).filter(move |symbol| seen.insert(symbol.position))
}

fn gear_numbers(schematic: &Schematic) -> HashMap<Coords, Vec<i64>> {
    let mut gears: HashMap<Coords, Vec<i64>> = HashMap::new();
    for number in &schematic.numbers {
        for symbol in unique_neighboring_symbols(number, &schematic.symbols) {
            if symbol.character == '*' {
                gears.entry(symbol.position).or_default().push(number.value);
            }
        }
    }
    gears
}

fn parse_lines(input: &str) -> Result<Schematic, SolveError> {
    let mut symbols = HashMap::new();
    let mut numbers: Vec<Number> = Vec::new();

    for (y, line) in input.lines().enumerate() {
        let y = y as i64;

        let mut digits = String::new();
        let mut digit_cells = Vec::new();

        for (x, c) in line.chars().chain(std::iter::once('.')).enumerate() {
            let x = x as i64;
            if c.is_ascii_digit() {
                digits.push(c);
                digit_cells.push(Coords { x, y })
            } else {
                if !digits.is_empty() {
                    numbers.push(Number {
                        digit_cells,
                        value: digits.parse::<i64>()?,
                    });

                    digits.clear();
                    digit_cells = Vec::new();
                }

                if c != '.' {
                    symbols.insert(Coords { x, y }, c);
                }
            }
        }
    }

    Ok(Schematic { symbols, numbers })
}

pub fn part1(input: &str) -> Result<String, SolveError> {
    let schematic = parse_lines(input)?;
    let mut total = 0_i64;
    for number in &schematic.numbers {
        if neighboring_symbols(number, &schematic.symbols)
            .next()
            .is_some()
        {
            total = total
                .checked_add(number.value)
                .ok_or(SolveError::ArithmeticOverflow)?;
        }
    }
    Ok(total.to_string())
}

pub fn part2(input: &str) -> Result<String, SolveError> {
    let schematic = parse_lines(input)?;
    let mut total = 0_i64;
    for numbers in gear_numbers(&schematic).values() {
        if let [left, right] = numbers.as_slice() {
            let ratio = left
                .checked_mul(*right)
                .ok_or(SolveError::ArithmeticOverflow)?;
            total = total
                .checked_add(ratio)
                .ok_or(SolveError::ArithmeticOverflow)?;
        }
    }
    Ok(total.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use svutils::scaffold_test_result;

    const YEAR: i16 = 2023;
    const DAY: i16 = 3;

    #[test]
    fn star_touching_three_numbers_is_not_a_gear() {
        let input = "1.2\n.*.\n.3.";
        assert_eq!(part1(input).unwrap(), "6");
        assert_eq!(part2(input).unwrap(), "0");
    }

    #[test]
    fn end_of_line_flushes_only_pending_numbers() {
        let parsed = parse_lines("12\n34.\n...").unwrap();
        let values: Vec<_> = parsed.numbers.iter().map(|number| number.value).collect();
        assert_eq!(values, vec![12, 34]);
        assert!(parsed.symbols.is_empty());
    }

    #[test]
    fn x_is_a_symbol_instead_of_a_line_terminator() {
        assert_eq!(part1("1x2").unwrap(), "3");
    }

    #[test]
    fn number_can_belong_to_two_gears_but_counts_once_in_part_one() {
        assert_eq!(part1("1*2*3").unwrap(), "6");
        assert_eq!(part2("1*2*3").unwrap(), "8");
    }

    #[test]
    fn multiple_digits_touching_a_gear_link_the_number_once() {
        assert_eq!(part2("12\n*3").unwrap(), "36");
    }

    #[test]
    fn unrepresentable_number_is_an_error_in_both_parts() {
        let input = format!("{}*", "9".repeat(30));
        assert!(matches!(part1(&input), Err(SolveError::InvalidNumber(_))));
        assert!(matches!(part2(&input), Err(SolveError::InvalidNumber(_))));
    }

    #[test]
    fn answer_overflow_is_an_error() {
        let sum_overflow = format!("{}*1", i64::MAX);
        assert!(matches!(
            part1(&sum_overflow),
            Err(SolveError::ArithmeticOverflow)
        ));
        let product_overflow = format!("{}*2", i64::MAX);
        assert!(matches!(
            part2(&product_overflow),
            Err(SolveError::ArithmeticOverflow)
        ));
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
    fn equal_values_at_different_positions_are_distinct_gear_numbers() {
        assert_eq!(part1("12*12").unwrap(), "24");
        assert_eq!(part2("12*12").unwrap(), "144");
        assert_eq!(part1("12+12").unwrap(), "24");
        assert_eq!(part2("12+12").unwrap(), "0");
    }
}
