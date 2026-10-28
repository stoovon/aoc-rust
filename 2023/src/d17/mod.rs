use svutils::SolveError;

use std::{cmp::Reverse, collections::BinaryHeap};

struct HeatMap {
    width: usize,
    height: usize,
    costs: Vec<u8>,
}

impl HeatMap {
    fn parse(input: &str) -> Result<Self, SolveError> {
        if input.trim().is_empty() {
            return Err(SolveError::EmptyInput);
        }
        let mut costs = Vec::new();
        let mut width = None;
        let mut height = 0;
        for (index, line) in input.lines().enumerate() {
            let invalid =
                |message: &str| SolveError::InvalidInput(message.into()).at_line(index + 1);
            if line.is_empty() {
                return Err(invalid("heat-map row is empty"));
            }
            if line.len() != *width.get_or_insert(line.len()) {
                return Err(invalid("heat-map rows must have equal width"));
            }
            if !line.bytes().all(|byte| byte.is_ascii_digit()) {
                return Err(invalid("heat map accepts only ASCII digits"));
            }
            costs.extend(line.bytes().map(|byte| byte - b'0'));
            height += 1;
        }
        Ok(Self {
            width: costs.len() / height,
            height,
            costs,
        })
    }

    fn step(&self, position: usize, direction: Direction) -> Option<usize> {
        match direction {
            North if position >= self.width => Some(position - self.width),
            East if position % self.width + 1 < self.width => Some(position + 1),
            South if position / self.width + 1 < self.height => Some(position + self.width),
            West if position % self.width > 0 => Some(position - 1),
            _ => None,
        }
    }
}

#[derive(Clone, Copy)]
struct MovementRules {
    min_straight: usize,
    max_straight: usize,
}

const STANDARD: MovementRules = MovementRules {
    min_straight: 0,
    max_straight: 3,
};
const ULTRA: MovementRules = MovementRules {
    min_straight: 4,
    max_straight: 10,
};

#[derive(Hash, Eq, PartialEq, Ord, PartialOrd, Clone, Copy, Debug)]
enum Direction {
    North,
    East,
    South,
    West,
}

use Direction::*;

impl Direction {
    fn opposite(self) -> Self {
        match self {
            North => South,
            East => West,
            South => North,
            West => East,
        }
    }

    fn index(self) -> usize {
        match self {
            North => 0,
            East => 1,
            South => 2,
            West => 3,
        }
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Node {
    // Keep cost first so the heap prioritizes the cheapest routes.
    cost: i64,
    pos: usize,
    dir: Option<Direction>,
    distance: usize,
}

fn state_key(position: usize, direction: Direction, run_length: usize, max_run: usize) -> usize {
    // Moving states have run lengths 1..=max_run; their slots start at zero.
    (position * 4 + direction.index()) * max_run + (run_length - 1)
}

fn minimum_heat(grid: &HeatMap, rules: MovementRules) -> Result<i64, SolveError> {
    let state_count = grid
        .costs
        .len()
        .checked_mul(4)
        .and_then(|count| count.checked_mul(rules.max_straight))
        .ok_or(SolveError::ArithmeticOverflow)?;
    let mut best_cost = vec![None; state_count];
    let mut pending = BinaryHeap::new();
    pending.push(Reverse(Node {
        pos: 0,
        dir: None,
        distance: 0,
        cost: 0,
    }));
    while let Some(Reverse(Node {
        pos,
        dir,
        distance,
        cost,
    })) = pending.pop()
    {
        if let Some(direction) = dir {
            let key = state_key(pos, direction, distance, rules.max_straight);
            if best_cost[key] != Some(cost) {
                continue;
            }
        }
        if pos == grid.costs.len() - 1 && distance >= rules.min_straight {
            return Ok(cost);
        }
        for direction in [North, East, South, West] {
            let same_direction = dir.is_none_or(|previous| previous == direction);
            if let Some(previous) = dir {
                if previous.opposite() == direction
                    || (same_direction && distance == rules.max_straight)
                    || (!same_direction && distance < rules.min_straight)
                {
                    continue;
                }
            }
            let Some(next) = grid.step(pos, direction) else {
                continue;
            };
            let run_length = if same_direction { distance + 1 } else { 1 };
            let key = state_key(next, direction, run_length, rules.max_straight);
            let next_cost = cost
                .checked_add(i64::from(grid.costs[next]))
                .ok_or(SolveError::ArithmeticOverflow)?;
            if best_cost[key].is_some_and(|previous| previous <= next_cost) {
                continue;
            }
            best_cost[key] = Some(next_cost);
            pending.push(Reverse(Node {
                pos: next,
                dir: Some(direction),
                distance: run_length,
                cost: next_cost,
            }));
        }
    }
    Err(SolveError::InvalidInput(
        "destination has no legal route".into(),
    ))
}

pub fn part1(input: &str) -> Result<String, SolveError> {
    let grid = HeatMap::parse(input)?;
    Ok(minimum_heat(&grid, STANDARD)?.to_string())
}

pub fn part2(input: &str) -> Result<String, SolveError> {
    let grid = HeatMap::parse(input)?;
    Ok(minimum_heat(&grid, ULTRA)?.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use svutils::scaffold_test_result;

    const YEAR: i16 = 2023;
    const DAY: i16 = 17;

    #[test]
    fn test_part1_input() {
        scaffold_test_result(YEAR, DAY, "input.txt", "input-spec.1.txt", part1);
    }

    #[test]
    fn test_part2_input() {
        scaffold_test_result(YEAR, DAY, "input.txt", "input-spec.2.txt", part2);
    }

    #[test]
    fn ultra_crucible_cannot_stop_after_a_short_final_run() {
        let input = "11111\n11111";
        assert_eq!(part1(input).unwrap(), "5");
        assert!(matches!(part2(input), Err(SolveError::InvalidInput(_))));
    }

    #[test]
    fn ultra_crucible_can_stop_after_four_straight_moves() {
        let input = "11111\n11111\n11111\n11111\n11111";
        assert_eq!(part2(input).unwrap(), "8");
    }

    #[test]
    fn routes_can_use_the_maximum_run_on_either_axis() {
        assert_eq!(part1("1111\n1111\n1111\n1111").unwrap(), "6");
        for (height, width) in [(5, 11), (11, 5)] {
            let input = vec!["1".repeat(width); height].join("\n");
            assert_eq!(part2(&input).unwrap(), "14");
        }
    }

    #[test]
    fn heap_pops_lowest_cost_and_orders_equal_cost_states_consistently() {
        let first = Node {
            cost: 2,
            pos: 1,
            dir: Some(East),
            distance: 1,
        };
        let second = Node {
            cost: 2,
            pos: 2,
            dir: Some(East),
            distance: 1,
        };
        assert_ne!(first, second);
        assert_ne!(first.cmp(&second), std::cmp::Ordering::Equal);
        let mut heap = BinaryHeap::from([
            Reverse(Node {
                cost: 9,
                pos: 0,
                dir: None,
                distance: 0,
            }),
            Reverse(second),
            Reverse(first),
        ]);
        assert_eq!(heap.pop().unwrap().0.pos, 1);
        assert_eq!(heap.pop().unwrap().0.pos, 2);
        assert_eq!(heap.pop().unwrap().0.cost, 9);
    }

    #[test]
    fn an_invalid_cheap_arrival_does_not_beat_a_legal_goal() {
        let input = "111119\n999919\n999919\n999919\n999911";
        // Cost 9 ends with one eastward move; a legal final run costs at least 41.
        assert_eq!(part2(input).unwrap(), "41");
    }

    #[test]
    fn zero_cost_tiles_preserve_dijkstra_and_cycle_termination() {
        let input = "00000\n00000\n00000\n00000\n00000";
        assert_eq!(part1(input).unwrap(), "0");
        assert_eq!(part2(input).unwrap(), "0");
    }

    #[test]
    fn malformed_heat_maps_are_rejected() {
        for input in ["", " \t\r\n"] {
            assert!(matches!(part1(input), Err(SolveError::EmptyInput)));
            assert!(matches!(part2(input), Err(SolveError::EmptyInput)));
        }
        for input in ["\n1", "11\n1", "1\n\n1", "1\n\n", "1 ", "-1", "é", "1x"] {
            assert!(part1(input).is_err());
            assert!(part2(input).is_err());
        }
    }

    #[test]
    fn narrow_maps_crlf_and_starting_cost_are_handled() {
        for input in ["9123", "9123\n", "9\r\n1\r\n2\r\n3\r\n"] {
            assert_eq!(part1(input).unwrap(), "6");
        }
        assert_eq!(part2("91111").unwrap(), "4");
        assert_eq!(part2("9\n1\n1\n1\n1").unwrap(), "4");
        assert!(part1("11111").is_err());
        assert!(part2("1111").is_err());
        assert_eq!(part1("7").unwrap(), "0");
        assert!(part2("7").is_err());
    }

    fn segment_reference(grid: &HeatMap, rules: MovementRules) -> Option<i64> {
        use std::collections::VecDeque;
        if grid.costs.len() == 1 {
            return (rules.min_straight == 0).then_some(0);
        }
        // Independent model: move whole straight segments, alternating axes.
        // Queue-based repeated relaxation uses no heap or run-length state keys.
        let mut best: Vec<[Option<i64>; 2]> = vec![[None; 2]; grid.costs.len()];
        let mut pending: VecDeque<(usize, Option<usize>)> = VecDeque::from([(0, None)]);
        while let Some((position, previous_axis)) = pending.pop_front() {
            let cost = previous_axis.map_or(0, |axis| best[position][axis].unwrap());
            for (dx, dy, axis) in [(1, 0, 0), (-1, 0, 0), (0, 1, 1), (0, -1, 1)] {
                if previous_axis == Some(axis) {
                    continue;
                }
                let mut x = (position % grid.width) as isize;
                let mut y = (position / grid.width) as isize;
                let mut next_cost = cost;
                for length in 1..=rules.max_straight {
                    x += dx;
                    y += dy;
                    if x < 0 || y < 0 || x >= grid.width as isize || y >= grid.height as isize {
                        break;
                    }
                    let next = y as usize * grid.width + x as usize;
                    next_cost += i64::from(grid.costs[next]);
                    if length < rules.min_straight {
                        continue;
                    }
                    if best[next][axis].is_none_or(|old| next_cost < old) {
                        best[next][axis] = Some(next_cost);
                        pending.push_back((next, Some(axis)));
                    }
                }
            }
        }
        best.last().unwrap().iter().flatten().copied().min()
    }

    #[test]
    fn dijkstra_matches_independent_segment_relaxation() {
        let mut seed = 17_u64;
        for (height, width) in [(1, 1), (1, 5), (5, 1), (2, 5), (3, 3), (5, 6), (6, 5)] {
            for _ in 0..12 {
                let mut input = String::new();
                for row in 0..height {
                    if row > 0 {
                        input.push('\n');
                    }
                    for _ in 0..width {
                        seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
                        input.push(char::from(b'0' + ((seed >> 32) % 10) as u8));
                    }
                }
                let grid = HeatMap::parse(&input).unwrap();
                for rules in [STANDARD, ULTRA] {
                    assert_eq!(
                        minimum_heat(&grid, rules).ok(),
                        segment_reference(&grid, rules),
                        "{input}"
                    );
                }
            }
        }
    }
}
