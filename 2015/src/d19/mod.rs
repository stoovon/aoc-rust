extern crate core;

use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, PartialEq)]
struct Solve<'a> {
    molecule: &'a String,
    combinations: &'a HashMap<String, String>
}

pub fn fn1(input: &str) -> i64 {
    let mut molecule = "";
    let mut combinations: HashMap<String, String> = HashMap::new();
    let mut hs = HashSet::new();

    for line in input.lines() {
        if let Some((left, right)) = line.split_once(" => ") {
            combinations.insert(right.to_string(), left.to_string());
        } else {
            molecule = line
        }
    }

    for (key, val) in combinations {
        molecule.match_indices(&val).into_iter().for_each(|mol| {
            let (left, right) = molecule.split_at(mol.0);
            let right = right.to_string().split_off(val.len());
            hs.insert(format!("{left}{key}{right}"));
        });
    }

    hs.len() as i64
}

pub fn fn2(input: &str) -> i64 {
    let mut molecule = "".to_string();
    let mut combinations: HashMap<String, String> = HashMap::new();

    for line in input.lines() {
        if let Some((left, right)) = line.split_once(" => ") {
            combinations.insert(right.to_string(), left.to_string());
        } else {
            molecule = line.to_string()
        }
    }

    let mut mol = molecule.clone();
    let mut count = 0;

    loop {
        let mut done = true;

        combinations.clone().into_iter().for_each(|(key, val)| {
            if let Some(pos) = mol.find(&key) {
                let (left, right) = mol.split_at(pos);
                let right = right.to_string().split_off(key.len());
                mol = format!("{left}{val}{right}");
                done = false;
                count += 1;
            }
        });
        if done {
            break;
        }
    }

    count
}

#[cfg(test)]
mod tests {
    use super::*;
    use svutils::load_spec;

    // #[test]
    // fn test_fn1_example() {
    //     assert_eq!(fn1(include_str!("example.txt")), load_spec(include_str!("example-spec.1.txt")));
    // }

    #[test]
    fn test_fn1_test_case_name() {
        assert_eq!(fn1(include_str!("input.txt")), load_spec(include_str!("input-spec.1.txt")));
    }

    #[test]
    fn test_fn2_example() {
        assert_eq!(fn2(include_str!("example.txt")), load_spec(include_str!("example-spec.2.txt")));
    }

    #[test]
    fn test_fn2_test_case_name() {
        assert_eq!(fn2(include_str!("input.txt")), load_spec(include_str!("input-spec.2.txt")));
    }
}