extern crate core;

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
struct Component {
    symbol: Symbol,
    nearby_nums: Vec<i64>,
}

#[derive(Debug)]
struct Number {
    digit_cells: Vec<Coords>,
    value: i64,
}

#[derive(Debug)]
struct Schematic {
    components: Vec<Symbol>,
    numbers: Vec<Number>,
}

fn neighboring_symbols<'number, 'map: 'number>(
    number: &'number Number,
    components: &'map HashMap<Coords, Component>,
) -> impl Iterator<Item = &'map Component> + 'number {
    number.digit_cells.iter().flat_map(move |cell| {
        (cell.x - 1..=cell.x + 1).flat_map(move |x| {
            (cell.y - 1..=cell.y + 1).filter_map(move |y| {
                if x == cell.x && y == cell.y {
                    return None;
                }
                components.get(&Coords { x, y })
            })
        })
    })
}

fn has_neighbor(number: &Number, components: &HashMap<Coords, Component>) -> bool {
    neighboring_symbols(number, components).next().is_some()
}

fn find_neighbors<'a>(
    number: &Number,
    components: &'a HashMap<Coords, Component>,
) -> Vec<&'a Component> {
    let mut seen = HashSet::new();
    neighboring_symbols(number, components)
        .filter(|component| seen.insert(component.symbol.position))
        .collect()
}

fn component_map(components: &[Symbol]) -> HashMap<Coords, Component> {
    let mut coords: HashMap<Coords, Component> = HashMap::new();

    for comp in components {
        coords.insert(
            comp.position,
            Component {
                symbol: *comp,
                nearby_nums: Vec::new(),
            },
        );
    }
    coords
}

fn link(input: &Schematic) -> Vec<Component> {
    let coords = component_map(&input.components);

    let mut bindings: HashMap<Symbol, Vec<i64>> = HashMap::new();

    for number in input.numbers.iter() {
        for component_hit in find_neighbors(number, &coords) {
            let key = component_hit.symbol;

            bindings.entry(key).or_default().push(number.value);
        }
    }

    bindings
        .into_iter()
        .map(|(coords, vals)| Component {
            symbol: coords,
            nearby_nums: vals,
        })
        .collect()
}

fn parse_lines(input: &str) -> Result<Schematic, SolveError> {
    let mut components: Vec<Symbol> = Vec::new();
    let mut numbers: Vec<Number> = Vec::new();

    for (y, line) in input.lines().enumerate() {
        let y = y as i64;

        let mut built_part_number = "".to_string();
        let mut digit_cells = Vec::new();

        for (x, c) in line.chars().chain(std::iter::once('.')).enumerate() {
            let x = x as i64;
            if c.is_ascii_digit() {
                built_part_number.push(c);
                digit_cells.push(Coords { x, y })
            } else {
                if !built_part_number.is_empty() {
                    numbers.push(Number {
                        digit_cells,
                        value: built_part_number.parse::<i64>()?,
                    });

                    built_part_number.clear();
                    digit_cells = Vec::new();
                }

                if c != '.' {
                    components.push(Symbol {
                        position: Coords { x, y },
                        character: c,
                    })
                }
            }
        }
    }

    Ok(Schematic {
        components,
        numbers,
    })
}

pub fn part1(input: &str) -> Result<String, SolveError> {
    let mut result = 0_i64;

    let pr = parse_lines(input)?;
    let coords = component_map(&pr.components);

    for number in &pr.numbers {
        if has_neighbor(number, &coords) {
            result = result
                .checked_add(number.value)
                .ok_or(SolveError::ArithmeticOverflow)?;
        }
    }

    Ok(result.to_string())
}

pub fn part2(input: &str) -> Result<String, SolveError> {
    let mut result = 0_i64;

    let pr = parse_lines(input)?;
    let linked = link(&pr);

    for component in linked.iter() {
        if component.symbol.character == '*' {
            if let [left, right] = component.nearby_nums.as_slice() {
                let ratio = left
                    .checked_mul(*right)
                    .ok_or(SolveError::ArithmeticOverflow)?;
                result = result
                    .checked_add(ratio)
                    .ok_or(SolveError::ArithmeticOverflow)?;
            }
        }
    }

    Ok(result.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use svutils::scaffold_test_result as scaffold_test;

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
        assert!(parsed.components.is_empty());
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
        scaffold_test(YEAR, DAY, "input.txt", "input-spec.1.txt", part1);
    }

    #[test]
    fn test_part2_input() {
        scaffold_test(YEAR, DAY, "input.txt", "input-spec.2.txt", part2);
    }
}
