use std::collections::{HashMap, HashSet};
use svutils::SolveError;

#[derive(Clone, Copy)]
enum Operand<'a> {
    Literal(u16),
    Wire(&'a str),
}

#[derive(Clone, Copy)]
enum Gate<'a> {
    Assign(Operand<'a>),
    Not(Operand<'a>),
    And(Operand<'a>, Operand<'a>),
    Or(Operand<'a>, Operand<'a>),
    LeftShift(Operand<'a>, u32),
    RightShift(Operand<'a>, u32),
}

fn wire_name(name: &str) -> bool {
    !name.is_empty() && name.bytes().all(|byte| byte.is_ascii_lowercase())
}

impl<'a> Operand<'a> {
    fn parse(token: &'a str) -> Result<Self, SolveError> {
        if wire_name(token) {
            Ok(Self::Wire(token))
        } else if !token.is_empty() && token.bytes().all(|byte| byte.is_ascii_digit()) {
            Ok(Self::Literal(token.parse()?))
        } else {
            Err(SolveError::InvalidInput(
                "expected a wire name or a 16-bit decimal value".into(),
            ))
        }
    }
}

fn shift(token: &str) -> Result<u32, SolveError> {
    if token.is_empty() || !token.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err(SolveError::InvalidInput(
            "expected a decimal shift count".into(),
        ));
    }
    let count = token.parse()?;
    if count >= u16::BITS {
        return Err(SolveError::InvalidInput(
            "shift count must be between 0 and 15".into(),
        ));
    }
    Ok(count)
}

struct Circuit<'a> {
    gates: HashMap<&'a str, Gate<'a>>,
}

impl<'a> Circuit<'a> {
    fn parse(input: &'a str) -> Result<Self, SolveError> {
        if input.trim().is_empty() {
            return Err(SolveError::EmptyInput);
        }
        let mut gates = HashMap::new();
        for (index, line) in input.lines().enumerate() {
            let parse_line = || -> Result<(&'a str, Gate<'a>), SolveError> {
                let mut fields = [""; 5];
                let mut count = 0;
                for token in line.split_ascii_whitespace() {
                    let field = fields.get_mut(count).ok_or_else(|| {
                        SolveError::InvalidInput("too many tokens in gate instruction".into())
                    })?;
                    *field = token;
                    count += 1;
                }
                let (wire, gate) = match &fields[..count] {
                    [value, "->", wire] => (*wire, Gate::Assign(Operand::parse(value)?)),
                    ["NOT", value, "->", wire] => (*wire, Gate::Not(Operand::parse(value)?)),
                    [left, "AND", right, "->", wire] => (
                        *wire,
                        Gate::And(Operand::parse(left)?, Operand::parse(right)?),
                    ),
                    [left, "OR", right, "->", wire] => (
                        *wire,
                        Gate::Or(Operand::parse(left)?, Operand::parse(right)?),
                    ),
                    [value, "LSHIFT", count, "->", wire] => (
                        *wire,
                        Gate::LeftShift(Operand::parse(value)?, shift(count)?),
                    ),
                    [value, "RSHIFT", count, "->", wire] => (
                        *wire,
                        Gate::RightShift(Operand::parse(value)?, shift(count)?),
                    ),
                    _ => {
                        return Err(SolveError::InvalidInput(
                            "expected a gate expression followed by '-> wire'".into(),
                        ))
                    }
                };
                if !wire_name(wire) {
                    return Err(SolveError::InvalidInput(
                        "destination must be a lowercase wire name".into(),
                    ));
                }
                Ok((wire, gate))
            };
            let (wire, gate) = parse_line().map_err(|error| error.at_line(index + 1))?;
            if gates.insert(wire, gate).is_some() {
                return Err(SolveError::InvalidInput(format!(
                    "duplicate definition for wire '{wire}'"
                ))
                .at_line(index + 1));
            }
        }
        Ok(Self { gates })
    }

    fn evaluate(
        &self,
        target: &'a str,
        override_wire: Option<(&'a str, u16)>,
    ) -> Result<u16, SolveError> {
        let mut values = HashMap::new();
        if let Some((wire, value)) = override_wire {
            values.insert(wire, value);
        }
        let mut visiting = HashSet::new();
        let mut pending = vec![(target, false)];
        while let Some((wire, ready)) = pending.pop() {
            if values.contains_key(wire) {
                continue;
            }
            let gate = self
                .gates
                .get(wire)
                .ok_or_else(|| SolveError::InvalidInput(format!("undefined wire '{wire}'")))?;
            if ready {
                let value = |operand: Operand<'a>| -> Result<u16, SolveError> {
                    match operand {
                        Operand::Literal(value) => Ok(value),
                        Operand::Wire(name) => values.get(name).copied().ok_or_else(|| {
                            SolveError::InvalidInput(format!("unresolved wire '{name}'"))
                        }),
                    }
                };
                let result = match *gate {
                    Gate::Assign(a) => value(a)?,
                    Gate::Not(a) => !value(a)?,
                    Gate::And(a, b) => value(a)? & value(b)?,
                    Gate::Or(a, b) => value(a)? | value(b)?,
                    Gate::LeftShift(a, count) => value(a)? << count,
                    Gate::RightShift(a, count) => value(a)? >> count,
                };
                values.insert(wire, result);
                visiting.remove(wire);
                continue;
            }
            if !visiting.insert(wire) {
                return Err(SolveError::InvalidInput(format!(
                    "cycle involving wire '{wire}'"
                )));
            }
            pending.push((wire, true));
            let (a, b) = match *gate {
                Gate::Assign(a) | Gate::Not(a) | Gate::LeftShift(a, _) | Gate::RightShift(a, _) => {
                    (a, None)
                }
                Gate::And(a, b) | Gate::Or(a, b) => (a, Some(b)),
            };
            if let Some(Operand::Wire(name)) = b {
                pending.push((name, false));
            }
            if let Operand::Wire(name) = a {
                pending.push((name, false));
            }
        }
        values
            .get(target)
            .copied()
            .ok_or_else(|| SolveError::InvalidInput(format!("undefined wire '{target}'")))
    }
}

pub fn part1(input: &str) -> Result<String, SolveError> {
    Circuit::parse(input)?
        .evaluate("a", None)
        .map(|value| value.to_string())
}

pub fn part2(input: &str) -> Result<String, SolveError> {
    let circuit = Circuit::parse(input)?;
    let first = circuit.evaluate("a", None)?;
    circuit
        .evaluate("a", Some(("b", first)))
        .map(|value| value.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use svutils::scaffold_test_result;

    #[test]
    fn gates_and_override_have_16_bit_semantics() {
        let input = "7 -> b\nb LSHIFT 1 -> x\nNOT x -> a\n";
        assert_eq!(part1(input).unwrap(), "65521");
        assert_eq!(part2(input).unwrap(), "29");
        let circuit =
            Circuit::parse("43 AND 29 -> x\n43 OR 29 -> y\nx LSHIFT 2 -> z\ny RSHIFT 1 -> a")
                .unwrap();
        assert_eq!(circuit.evaluate("x", None).unwrap(), 9);
        assert_eq!(circuit.evaluate("z", None).unwrap(), 36);
        assert_eq!(circuit.evaluate("a", None).unwrap(), 31);
        assert_eq!(part1("65535 LSHIFT 1 -> a").unwrap(), "65534");
        assert_eq!(part1("NOT 0 -> a").unwrap(), "65535");
        assert_eq!(part1("\t9   RSHIFT  0 -> a\r\n").unwrap(), "9");
    }

    #[test]
    fn malformed_lines_are_errors() {
        for solver in [part1, part2] {
            assert!(solver("").is_err());
            for bad in [
                "",
                "9 => a",
                "9 -> a trailing",
                "NOT -> a",
                "9 XOR 7 -> a",
                "65536 -> a",
                "-1 -> a",
                "9 -> A",
                "9 -> 4",
                "9 LSHIFT 16 -> a",
                "9 RSHIFT -1 -> a",
                "9 -> a",
                "é -> a",
            ] {
                let input = format!("8 -> a\n{bad}\n");
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
    fn missing_wires_and_cycles_are_errors() {
        for input in ["x -> a", "a -> a", "b -> a\na -> b", "9 -> x"] {
            assert!(part1(input).is_err());
            assert!(part2(input).is_err());
        }
        assert_eq!(part1("8 -> x\nx AND x -> a").unwrap(), "8");
        assert_eq!(part1("y -> x\n8 -> y\nx OR y -> a").unwrap(), "8");
        let circuit = Circuit::parse("b -> a\na -> b").unwrap();
        assert_eq!(circuit.evaluate("a", Some(("b", 11))).unwrap(), 11);
    }

    #[test]
    fn generated_dags_match_direct_values() {
        fn name(mut index: usize) -> String {
            let mut value = String::from("w");
            loop {
                value.push(char::from(b'a' + (index % 26) as u8));
                index /= 26;
                if index == 0 {
                    return value;
                }
            }
        }
        let mut input = String::from("37 -> b\n");
        let mut values = vec![37u16];
        let mut names = vec![String::from("b")];
        for index in 1..=300 {
            let left = (index / 3 + 7) % index;
            let right = (index * 13 + 5) % index;
            let target = name(index);
            let (expression, value) = match index % 6 {
                0 => (names[left].clone(), values[left]),
                1 => (format!("NOT {}", names[left]), !values[left]),
                2 => (
                    format!("{} AND {}", names[left], names[right]),
                    values[left] & values[right],
                ),
                3 => (
                    format!("{} OR {}", names[left], names[right]),
                    values[left] | values[right],
                ),
                4 => (format!("{} LSHIFT 3", names[left]), values[left] << 3),
                _ => (format!("{} RSHIFT 2", names[left]), values[left] >> 2),
            };
            input.push_str(&format!("{expression} -> {target}\n"));
            names.push(target);
            values.push(value);
        }
        let circuit = Circuit::parse(&input).unwrap();
        for (name, expected) in names.iter().zip(values) {
            assert_eq!(circuit.evaluate(name, None).unwrap(), expected);
        }
    }

    #[test]
    fn long_chain_uses_explicit_stack() {
        let mut input = String::from("19 -> b\n");
        let mut previous = String::from("b");
        for index in 0..10000usize {
            let name: String = std::iter::once('w')
                .chain(
                    (0..4)
                        .map(|shift| char::from(b'a' + ((index / 26usize.pow(shift)) % 26) as u8)),
                )
                .collect();
            input.push_str(&format!("{previous} -> {name}\n"));
            previous = name;
        }
        input.push_str(&format!("{previous} -> a\n"));
        assert_eq!(part1(&input).unwrap(), "19");
        assert_eq!(part2(&input).unwrap(), "19");
    }

    #[test]
    fn test_part1_input() {
        scaffold_test_result(2015, 7, "input.txt", "input-spec.1.txt", part1);
    }

    #[test]
    fn test_part2_input() {
        scaffold_test_result(2015, 7, "input.txt", "input-spec.2.txt", part2);
    }
}
