extern crate core;

use std::collections::{HashMap, VecDeque};

enum Node<'a> {
  FlipFlop(bool),
  Conjunction(HashMap<&'a str, bool>),
  EntryPoint,
}

fn combined(input: &str) -> (usize, usize) {
  let mut graph = HashMap::new();
  let mut state = HashMap::new();

  for l in input.split('\n') {
    let (src, rest) = l.split_once(" -> ").unwrap();
    let connections = rest.split(", ").collect::<Vec<_>>();
    let (node, state_type) = match src.as_bytes()[0] as char {
      '%' => (&src[1..], Node::FlipFlop(false)),
      '&' => (&src[1..], Node::Conjunction(HashMap::new())),
      'b' => (src,       Node::EntryPoint),
      _ => unreachable!(),
    };
    graph.insert(node, connections);
    state.insert(node, state_type);
  }

  let mut rx_conjunction = "";
  for (&node, connections) in &graph {
    for &n in connections {
      match state.get_mut(n) {
        Some(Node::Conjunction(m)) => { m.insert(node, false); },
        Some(_) => {},
        None => rx_conjunction = node,
      }
    }
  }

  let mut cycles = match &state[rx_conjunction] {
    Node::Conjunction(m) => m.iter()
      .map(|(&node,_)| (node, None))
      .collect::<HashMap<_,_>>(),
    _ => unreachable!(),
  };

  let mut p1 = [0,0]; // Low, High
  let mut q = VecDeque::new();

  'outer: for t in 1.. {
    q.push_back(("broadcaster", "button", false));
    while let Some((node, prev, high)) = q.pop_front() {
      if t <= 1000 {
        p1[high as usize] += 1;
      }
      if high && node == rx_conjunction {
        let v = cycles.get_mut(prev).unwrap();
        if v.is_none() {
          *v = Some(t);
          if cycles.values().all(|o| o.is_some()) {
            break 'outer;
          }
        }
      }
      let pulse = match state.get_mut(node) {
        Some(Node::FlipFlop(_)) if high => continue,
        Some(Node::FlipFlop(on)) => {
          *on = !*on;
          *on
        },
        Some(Node::Conjunction(m)) => {
          m.insert(prev, high);
          m.values().any(|&b| !b)
        }
        Some(Node::EntryPoint) => false,
        None => continue,
      };
      q.extend(graph[node].iter().map(|&n| (n, node, pulse)));
    }
  }

  (p1[0] * p1[1], cycles.values().map(|o| o.unwrap()).product())
}

pub fn fn1(input: &str) -> i64 {
    combined(input).0 as i64
}

pub fn fn2(input: &str) -> i64 {
    combined(input).1 as i64
}

#[cfg(test)]
mod tests {
    use super::*;
    use svutils::scaffold_test;

    const YEAR: i16 = 2023;
    const DAY: i16 = 20;

    // #[test]
    // fn test_fn1_example() {
    //     scaffold_test(YEAR, DAY, "example.txt", "example-spec.1.txt", fn1);
    // }

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
