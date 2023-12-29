extern crate core;

use std::collections::HashMap;

type Result = (u16, u16);

enum Gate<'a> {
    Wire(&'a str),
    Not(&'a str),
    And(&'a str, &'a str),
    Or(&'a str, &'a str),
    LeftShift(&'a str, u16),
    RightShift(&'a str, u16),
}

pub fn parse(input: &str, chosen_wire: &str, override_wire: &str) -> Result {
    let mut tokens = input.split_ascii_whitespace();
    let mut circuit = HashMap::new();

    while let (Some(first), Some(second)) = (tokens.next(), tokens.next()) {
        let gate = if first == "NOT" {
            let _third = tokens.next().unwrap();
            Gate::Not(second)
        } else if second == "->" {
            Gate::Wire(first)
        } else {
            let third = tokens.next().unwrap();
            let _fourth = tokens.next().unwrap();

            match second {
                "AND" => Gate::And(first, third),
                "OR" => Gate::Or(first, third),
                "LSHIFT" => Gate::LeftShift(first, third.parse::<u16>().unwrap()),
                "RSHIFT" => Gate::RightShift(first, third.parse::<u16>().unwrap()),
                _ => unreachable!(),
            }
        };

        let wire = tokens.next().unwrap();
        circuit.insert(wire, gate);
    }

    let mut cache = HashMap::new();
    let result1 = signal(chosen_wire, &circuit, &mut cache);

    cache.clear();
    cache.insert(override_wire, result1);
    let result2 = signal(chosen_wire, &circuit, &mut cache);

    (result1, result2)
}

fn signal<'a>(
    key: &'a str,
    circuit: &HashMap<&'a str, Gate<'a>>,
    cache: &mut HashMap<&'a str, u16>,
) -> u16 {
    if let Some(result) = cache.get(key) {
        return *result;
    }

    let result = if key.chars().next().unwrap().is_ascii_digit() {
        key.parse::<u16>().unwrap()
    } else {
        match circuit[key] {
            Gate::Wire(w) => signal(w, circuit, cache),
            Gate::Not(w) => !signal(w, circuit, cache),
            Gate::And(l, r) => signal(l, circuit, cache) & signal(r, circuit, cache),
            Gate::Or(l, r) => signal(l, circuit, cache) | signal(r, circuit, cache),
            Gate::LeftShift(w, n) => signal(w, circuit, cache) << n,
            Gate::RightShift(w, n) => signal(w, circuit, cache) >> n,
        }
    };

    cache.insert(key, result);
    result
}

pub fn fn1(input: &str, chosen_wire: &str, override_wire: &str) -> i64 {
    let (result, _) = parse(input, chosen_wire, override_wire);
    result as i64
}

pub fn fn2(input: &str, chosen_wire: &str, override_wire: &str) -> i64 {
    let (_, result) = parse(input, chosen_wire, override_wire);
    result as i64
}

#[cfg(test)]
mod tests {
    use super::*;
    use svutils::scaffold_test;

    const YEAR: i16 = 2015;
    const DAY: i16 = 7;

    #[test]
    fn test_fn1_example() {
        scaffold_test(YEAR, DAY, "example.txt", "example-spec.1.txt", |input| {fn1(input, "h", "i")});
    }

    #[test]
    fn test_fn1_input() {
        scaffold_test(YEAR, DAY, "input.txt", "input-spec.1.txt", |input| {fn1(input, "a", "b")});
    }

    #[test]
    fn test_fn2_example() {
        scaffold_test(YEAR, DAY, "example.txt", "example-spec.2.txt", |input| {fn2(input, "h", "i")});
    }

    #[test]
    fn test_fn2_input() {
        scaffold_test(YEAR, DAY, "input.txt", "input-spec.2.txt", |input| {fn2(input, "a", "b")});
    }
}
