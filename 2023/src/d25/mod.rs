extern crate core;

use std::collections::{BTreeMap, BTreeSet, VecDeque};

fn parse(contents: &str) -> BTreeMap<String, Vec<String>> {
    let mut graph: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for line in contents.lines() {
        if let Some((from, to)) = line.split_once(": ") {
            let source = from.to_string();
            for target in to.split(' ') {
                graph
                    .entry(source.clone())
                    .and_modify(|e| {
                        e.push(target.to_string());
                    })
                    .or_insert(vec![target.to_string()]);
                graph
                    .entry(target.to_string())
                    .and_modify(|e| {
                        e.push(source.clone());
                    })
                    .or_insert(vec![source.clone()]);
            }
        }
    }
    graph
}

fn split(graph: &BTreeMap<String, Vec<String>>) -> (u32, u32) {
    let mut first_group: u32 = 1;
    let mut second_group: u32 = 0;
    let mut components = graph.keys();
    let first_component = components.next().unwrap();
    for component in components {
        let mut connections: u32 = 0;
        let mut tested: BTreeSet<String> = BTreeSet::from([first_component.clone()]);
        for starting_component in graph.get(first_component).unwrap() {
            if *starting_component == *component {
                connections += 1;
                continue;
            }
            let mut seen: BTreeSet<String> = BTreeSet::new();
            let mut q: VecDeque<(String, Vec<String>)> = VecDeque::new();
            let mut found = false;
            q.push_back((starting_component.clone(), vec![starting_component.clone()]));
            while !q.is_empty() && !found && connections < 4 {
                let (other_component, path) = q.pop_front().unwrap();
                for c in graph.get(&other_component).unwrap() {
                    if *component == *c {
                        connections += 1;
                        for p in path.iter() {
                            tested.insert(p.clone());
                        }
                        found = true;
                        break;
                    }
                    else if !seen.contains(c) && !path.contains(&c) && !tested.contains(c) {
                        let mut new_path = path.clone();
                        new_path.push(c.clone());
                        q.push_back((c.clone(), new_path));
                        seen.insert(c.clone());
                    }
                }
            }
        }
        if connections >= 4 {
            first_group += 1;
        } else {
            second_group += 1;
        }
    }
    (first_group, second_group)
}

pub fn fn1(input: &str) -> i64 {
    // Build a list of connections
    // Initial solve in Python/networkx. Will try to implement the solution in Rust soon.

    let graph = parse(input);
    let (first_group, second_group) = split(&graph);
    (first_group * second_group) as i64
}

#[cfg(test)]
mod tests {
    use super::*;
    use svutils::scaffold_test;

    const YEAR: i16 = 2023;
    const DAY: i16 = 25;

    #[test]
    fn test_fn1_input() {
        scaffold_test(YEAR, DAY, "input.txt", "input-spec.1.txt", fn1);
    }
}
