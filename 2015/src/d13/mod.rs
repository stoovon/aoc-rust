use std::collections::{HashMap, HashSet};
use std::str::FromStr;

extern crate core;

struct Party {
    happiness: HashMap<(String, String), isize>,
    people: HashSet<String>,
}

impl FromStr for Party {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut party = Party::new();

        for line in s.lines() {
            let mut words = line.split_whitespace();
            let person = words.next().unwrap();
            let happiness = words.nth(2).unwrap().parse::<isize>().unwrap();
            let other = words.nth(6).unwrap().trim_end_matches('.');
            if line.contains("lose") {
                party.add(person, other, -happiness);
            } else {
                party.add(person, other, happiness);
            }
        }

        Ok(party)
    }
}

impl Party {
    fn new() -> Party {
        Party {
            happiness: HashMap::new(),
            people: HashSet::new(),
        }
    }

    fn add(&mut self, person: &str, other: &str, happiness: isize) {
        self.happiness
            .insert((person.to_string(), other.to_string()), happiness);
        self.people.insert(person.to_string());
        self.people.insert(other.to_string());
    }

    fn happiness(&self, person: &str, other: &str) -> isize {
        self.happiness
            .get(&(person.to_string(), other.to_string()))
            .unwrap_or(&0)
            + self
                .happiness
                .get(&(other.to_string(), person.to_string()))
                .unwrap_or(&0)
    }

    fn people(&self) -> Vec<String> {
        self.people.iter().cloned().collect()
    }
}

fn calculate_permutations(items: Vec<String>) -> Vec<Vec<String>> {
    if items.len() == 1 {
        return vec![items.to_vec()];
    }

    let mut permutations = Vec::new();
    for i in 0..items.len() {
        let mut items = items.to_vec();
        let item = items.remove(i);
        for mut permutation in calculate_permutations(items) {
            permutation.push(item.clone());
            permutations.push(permutation);
        }
    }

    permutations
}

pub fn fn1(input: &str) -> i64 {
    let party: Party = input.parse().unwrap();

    let mut max_happiness = 0;

    for permutation in calculate_permutations(party.people()) {
        let mut happiness = 0;
        for i in 0..permutation.len() {
            let person = &permutation[i];
            let other = &permutation[(i + 1) % permutation.len()];
            happiness += party.happiness(person, other);
        }
        if happiness > max_happiness {
            max_happiness = happiness;
        }
    }

    max_happiness as i64
}

pub fn fn2(input: &str) -> i64 {
    let mut party: Party = input.parse().unwrap();

    for person in party.people() {
        party.add("Me", &person, 0);
        party.add(&person, "Me", 0);
    }

    let mut max_happiness = 0;

    for permutation in calculate_permutations(party.people()) {
        let mut happiness = 0;
        for i in 0..permutation.len() {
            let person = &permutation[i];
            let other = &permutation[(i + 1) % permutation.len()];
            happiness += party.happiness(person, other);
        }
        if happiness > max_happiness {
            max_happiness = happiness;
        }
    }

    max_happiness as i64
}

#[cfg(test)]
mod tests {
    use super::*;
    use svutils::scaffold_test;

    const YEAR: i16 = 2015;
    const DAY: i16 = 13;

    #[test]
    fn test_fn1_example() {
        scaffold_test(YEAR, DAY, "example.txt", "example-spec.1.txt", fn1);
    }

    #[test]
    fn test_fn1_input() {
        scaffold_test(YEAR, DAY, "input.txt", "input-spec.1.txt", fn1);
    }

    // #[test]
    // fn test_fn2_example() {
    //     scaffold_test(YEAR, DAY, "example.txt", "example-spec.2.txt", fn2);
    // }

    #[test]
    fn test_fn2_input() {
        scaffold_test(YEAR, DAY, "input.txt", "input-spec.2.txt", fn2);
    }
}
