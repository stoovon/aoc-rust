use std::{collections::HashSet, str::FromStr};
use svutils::SolveError;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Property {
    Children,
    Cats,
    Samoyeds,
    Pomeranians,
    Akitas,
    Vizslas,
    Goldfish,
    Trees,
    Cars,
    Perfumes,
}

#[derive(Clone, Copy)]
enum Rules {
    Exact,
    Adjusted,
}

impl Property {
    fn parse(name: &str) -> Result<Self, SolveError> {
        match name {
            "children" => Ok(Self::Children),
            "cats" => Ok(Self::Cats),
            "samoyeds" => Ok(Self::Samoyeds),
            "pomeranians" => Ok(Self::Pomeranians),
            "akitas" => Ok(Self::Akitas),
            "vizslas" => Ok(Self::Vizslas),
            "goldfish" => Ok(Self::Goldfish),
            "trees" => Ok(Self::Trees),
            "cars" => Ok(Self::Cars),
            "perfumes" => Ok(Self::Perfumes),
            _ => Err(SolveError::InvalidInput("unknown Sue property".into())),
        }
    }

    fn reference(self) -> u64 {
        match self {
            Self::Children => 3,
            Self::Cats => 7,
            Self::Samoyeds => 2,
            Self::Pomeranians => 3,
            Self::Akitas | Self::Vizslas => 0,
            Self::Goldfish => 5,
            Self::Trees => 3,
            Self::Cars => 2,
            Self::Perfumes => 1,
        }
    }

    fn matches(self, value: u64, rules: Rules) -> bool {
        let reference = self.reference();
        match (rules, self) {
            (Rules::Adjusted, Self::Cats | Self::Trees) => value > reference,
            (Rules::Adjusted, Self::Pomeranians | Self::Goldfish) => value < reference,
            _ => value == reference,
        }
    }
}

fn decimal(text: &str) -> Result<u64, SolveError> {
    if text.is_empty() || !text.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err(SolveError::InvalidInput(
            "expected a nonnegative decimal integer".into(),
        ));
    }
    Ok(text.parse()?)
}

struct AuntSue {
    number: u64,
    properties: Vec<(Property, u64)>,
}

impl FromStr for AuntSue {
    type Err = SolveError;

    fn from_str(line: &str) -> Result<Self, Self::Err> {
        let (head, tail) = line.split_once(':').ok_or_else(|| {
            SolveError::InvalidInput("expected 'Sue number: property: count, ...'".into())
        })?;
        let mut head = head.split_ascii_whitespace();
        let (Some("Sue"), Some(number), None) = (head.next(), head.next(), head.next()) else {
            return Err(SolveError::InvalidInput(
                "expected 'Sue' and a positive identifier".into(),
            ));
        };
        let number = decimal(number)?;
        if number == 0 {
            return Err(SolveError::InvalidInput(
                "Sue identifiers must be positive".into(),
            ));
        }
        let mut properties = Vec::new();
        for field in tail.split(',') {
            let (name, value) = field
                .split_once(':')
                .ok_or_else(|| SolveError::InvalidInput("expected a property and count".into()))?;
            let property = Property::parse(name.trim())?;
            let value = decimal(value.trim())?;
            if properties.iter().any(|&(existing, _)| existing == property) {
                return Err(SolveError::InvalidInput(
                    "duplicate property in Sue record".into(),
                ));
            }
            properties.push((property, value));
        }
        Ok(Self { number, properties })
    }
}

impl AuntSue {
    fn matches(&self, rules: Rules) -> bool {
        self.properties
            .iter()
            .all(|&(property, value)| property.matches(value, rules))
    }
}

fn solve(input: &str, rules: Rules) -> Result<String, SolveError> {
    if input.trim().is_empty() {
        return Err(SolveError::EmptyInput);
    }
    let mut identifiers = HashSet::new();
    let mut matched = None;
    for (index, line) in input.lines().enumerate() {
        let aunt: AuntSue = line
            .parse()
            .map_err(|error: SolveError| error.at_line(index + 1))?;
        if !identifiers.insert(aunt.number) {
            return Err(
                SolveError::InvalidInput("duplicate Sue identifier".into()).at_line(index + 1)
            );
        }
        if aunt.matches(rules) && matched.replace(aunt.number).is_some() {
            return Err(SolveError::InvalidInput(
                "multiple Sues match the reference".into(),
            ));
        }
    }
    matched
        .map(|number| number.to_string())
        .ok_or_else(|| SolveError::InvalidInput("no Sue matches the reference".into()))
}

pub fn part1(input: &str) -> Result<String, SolveError> {
    solve(input, Rules::Exact)
}
pub fn part2(input: &str) -> Result<String, SolveError> {
    solve(input, Rules::Adjusted)
}

#[cfg(test)]
mod tests {
    use super::*;
    use svutils::scaffold_test_result;

    const PROPERTIES: [(&str, u64); 10] = [
        ("children", 3),
        ("cats", 7),
        ("samoyeds", 2),
        ("pomeranians", 3),
        ("akitas", 0),
        ("vizslas", 0),
        ("goldfish", 5),
        ("trees", 3),
        ("cars", 2),
        ("perfumes", 1),
    ];

    #[test]
    fn every_property_obeys_exact_and_strict_rules() {
        for (index, &(name, reference)) in PROPERTIES.iter().enumerate() {
            for value in 0..=10 {
                let aunt: AuntSue = format!("Sue 41: {name}: {value}").parse().unwrap();
                assert_eq!(aunt.matches(Rules::Exact), value == reference);
                let adjusted = match index {
                    1 | 7 => value > reference,
                    3 | 6 => value < reference,
                    _ => value == reference,
                };
                assert_eq!(aunt.matches(Rules::Adjusted), adjusted);
            }
        }
        for (name, reference) in [("goldfish", 5), ("pomeranians", 3)] {
            let input = format!(
                "Sue 41: {name}: {reference}\nSue 72: {name}: {}",
                reference - 1
            );
            assert_eq!(part1(&input).unwrap(), "41");
            assert_eq!(part2(&input).unwrap(), "72");
        }
    }

    #[test]
    fn generated_partial_reports_match_independent_rules() {
        for mut code in 0..3usize.pow(10) {
            let mut properties = Vec::new();
            let mut exact = true;
            let mut adjusted = true;
            for (index, &(name, reference)) in PROPERTIES.iter().enumerate() {
                let choice = code % 3;
                code /= 3;
                if choice == 0 {
                    continue;
                }
                let value = if choice == 1 {
                    reference
                } else {
                    match index {
                        3 | 6 => reference - 1,
                        _ => reference + 1,
                    }
                };
                exact &= value == reference;
                adjusted &= match index {
                    1 | 7 => value > reference,
                    3 | 6 => value < reference,
                    _ => value == reference,
                };
                properties.push((Property::parse(name).unwrap(), value));
            }
            let aunt = AuntSue {
                number: 41,
                properties,
            };
            assert_eq!(aunt.matches(Rules::Exact), exact);
            assert_eq!(aunt.matches(Rules::Adjusted), adjusted);
        }
    }

    #[test]
    fn matching_requires_one_candidate_and_validates_later_records() {
        for solver in [part1, part2] {
            assert_eq!(
                solver("Sue 41: akitas: 0\nSue 72: akitas: 1").unwrap(),
                "41"
            );
            assert_eq!(
                solver("Sue 72: akitas: 1\nSue 41: akitas: 0").unwrap(),
                "41"
            );
            assert!(solver("Sue 41: akitas: 1")
                .unwrap_err()
                .to_string()
                .contains("no Sue"));
            assert!(solver("Sue 41: akitas: 0\nSue 72: cars: 2")
                .unwrap_err()
                .to_string()
                .contains("multiple"));
            assert!(solver("Sue 41: akitas: 0\nSue 41: akitas: 1")
                .unwrap_err()
                .to_string()
                .contains("duplicate Sue"));
            assert!(solver("Sue 41: akitas: 0\nbroken")
                .unwrap_err()
                .to_string()
                .starts_with("line 2:"));
            assert_eq!(
                solver("Sue 18446744073709551615: akitas: 0").unwrap(),
                u64::MAX.to_string()
            );
        }
        assert_eq!(
            part1(" \tSue\t41 : cars : 2 , perfumes: 1\r\n").unwrap(),
            "41"
        );
        assert_eq!(part2("Sue 41: cats: 18446744073709551615").unwrap(), "41");
    }

    #[test]
    fn parser_rejects_bad_ids_counts_and_properties() {
        for solver in [part1, part2] {
            for empty in ["", " \r\n"] {
                assert!(matches!(solver(empty), Err(SolveError::EmptyInput)));
            }
            for bad in [
                "",
                "Sue 0: cars: 2",
                "Aunt 41: cars: 2",
                "Sue Sue 41: cars: 2",
                "Sue -41: cars: 2",
                "Sue +41: cars: 2",
                "Sue 18446744073709551616: cars: 2",
                "Sue 41",
                "Sue 41:",
                "Sue 41: cars: -2",
                "Sue 41: cars: +2",
                "Sue 41: cars: 2.0",
                "Sue 41: cars: 18446744073709551616",
                "Sue 41: cars: 2, cars: 2",
                "Sue 41: cars: 2, cars: 3",
                "Sue 41: dogs: 2",
                "Sue 41: cars: 2,",
                "Sue 41: cars: 2 extra",
                "Sue 41: cars: 2: 3",
                "Sue 41: cars: ２",
            ] {
                let input = format!("Sue 72: akitas: 1\n{bad}\n");
                assert!(
                    solver(&input)
                        .unwrap_err()
                        .to_string()
                        .starts_with("line 2:"),
                    "{bad:?}"
                );
            }
        }
    }

    #[test]
    fn test_part1_input() {
        scaffold_test_result(2015, 16, "input.txt", "input-spec.1.txt", part1);
    }
    #[test]
    fn test_part2_input() {
        scaffold_test_result(2015, 16, "input.txt", "input-spec.2.txt", part2);
    }
}
