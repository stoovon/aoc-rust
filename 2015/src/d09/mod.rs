extern crate core;

use std::collections::{HashMap, HashSet};

use itertools::Itertools;

fn parse(input: &str) -> (HashSet<String>, HashMap<(String, String), i64>) {
    let mut routes: HashMap<(String, String), i64> = HashMap::new();
    let mut cities: HashSet<String> = HashSet::new();

    for line in input.lines() {
        let tokens = line.split_whitespace().collect_vec();
        let (from, to, dist) = (
            tokens[0].to_string(),
            tokens[2].to_string(),
            tokens[4].parse().unwrap(),
        );

        routes.insert((from.clone(), to.clone()), dist);
        routes.insert((to.clone(), from.clone()), dist);
        cities.insert(from);
        cities.insert(to);
    }

    (cities, routes)
}

pub fn fn1(input: &str) -> i64 {
    let (cities, routes) = parse(input);

    let mut min_dist = i64::MAX;
    for route in cities.iter().permutations(cities.len()) {
        let mut dist = 0;
        for i in 0..route.len() - 1 {
            dist += routes
                .get(&(route[i].clone(), route[i + 1].clone()))
                .unwrap();
        }
        min_dist = min_dist.min(dist);
    }

    min_dist
}

pub fn fn2(input: &str) -> i64 {
    let (cities, routes) = parse(input);

    let mut max_dist = i64::MIN;

    for route in cities.iter().permutations(cities.len()) {
        let mut dist = 0;
        for i in 0..route.len() - 1 {
            dist += routes
                .get(&(route[i].clone(), route[i + 1].clone()))
                .unwrap();
        }
        max_dist = max_dist.max(dist);
    }

    max_dist
}

#[cfg(test)]
mod tests {
    use super::*;
    use svutils::scaffold_test;

    const YEAR: i16 = 2015;
    const DAY: i16 = 9;

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
