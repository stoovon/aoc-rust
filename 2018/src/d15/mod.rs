extern crate core;

use std::ops::Add;
use std::cmp::Ordering;
use std::collections::VecDeque;
use std::collections::HashMap;
use std::collections::HashSet;

// FIXME: Replace with the utility type
#[derive(Copy, Clone, Eq, PartialEq, Hash)]
struct Pos {
    row: i8,
    col: i8,
}

impl Pos {
    pub fn new(row: i8, col: i8) -> Pos {
        Pos { row, col }
    }

    pub fn get_index(self, width: usize) -> usize {
        self.row as usize * width + self.col as usize
    }

    pub fn adjacency_iter(self, width: usize, height: usize) -> AdjacencyIterator {
        AdjacencyIterator {
            pos: self,
            count: 0,
            width: width as i8,
            height: height as i8,
        }
    }
}

impl Add for Pos {
    type Output = Pos;

    fn add(self, other: Pos) -> Pos {
        Pos { row: self.row + other.row, col: self.col + other.col }
    }
}

impl PartialOrd for Pos {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if self.row < other.row {
            Some(Ordering::Less)
        } else if self.row > other.row {
            Some(Ordering::Greater)
        } else if self.col < other.col {
            Some(Ordering::Less)
        } else if self.col > other.col {
            Some(Ordering::Greater)
        } else {
            Some(Ordering::Equal)
        }
    }
}

struct AdjacencyIterator {
    pos: Pos,
    width: i8,
    height: i8,
    count: usize,
}

impl Iterator for AdjacencyIterator {
    type Item = Pos;

    fn next(&mut self) -> Option<Pos> {
        // NWES; reading order
        let offsets = [
            Pos::new(-1, 0),
            Pos::new(0, -1),
            Pos::new(0, 1),
            Pos::new(1, 0),
        ];

        let mut result: Option<Pos> = None;
        while result.is_none() && self.count < offsets.len() {
            let p = self.pos + offsets[self.count];
            if p.row >= 0 && p.row < self.height && p.col >= 0 && p.col < self.width {
                result = Some(p);
            }
            self.count += 1;
        }

        result
    }
}

// FIXME: Would be good to replace this for an entity in an entity system.
#[derive(Copy, Clone, Eq, PartialEq, Hash)]
struct Unit {
    id: u8,
    team_elf: bool,
    health: u8,
    turn: i64,
    pos: Pos,
}

impl Unit {
    pub fn get_key(self, width: usize, height: usize) -> usize {
        let r = self.pos.row as usize;
        let c = self.pos.col as usize;
        let t = self.turn as usize;

        (width * height * t) + r * width + c
    }
}

// Helper to performing breadth-first pathfind
#[derive(Copy, Clone, Eq, PartialEq)]
struct PathfindNode {
    pos: Pos,
    parent: Pos,
    cost: u16,
}

impl PathfindNode {
    fn new(pos: Pos, parent: Pos, cost: u16) -> Self {
        PathfindNode { pos, parent, cost }
    }
}

impl PartialOrd for PathfindNode {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if other.cost != self.cost {
            other.cost.partial_cmp(&self.cost)
        } else {
            other.pos.partial_cmp(&self.pos)
        }
    }
}

impl Ord for PathfindNode {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}

fn attack_adjacent_enemy_if_able(
    current_pos: Pos,
    playing_team_elf: bool,
    units: &mut HashMap<Pos, Unit>,
    unit_positions: &mut Vec<bool>,
    width: usize,
    height: usize,
    elf_power: u8,
    goblin_power: u8,
    dead_elves: &mut i64,
) -> bool {
    let mut best_enemy: Option<(Pos, u8)> = None;

    for adjacent_pos in current_pos.adjacency_iter(width, height) {
        let target = units.get_mut(&adjacent_pos);
        if target.is_none() {
            continue;
        }
        let target = target.unwrap();

        // No friendly fire
        if target.team_elf == playing_team_elf {
            continue;
        }

        if best_enemy.is_none() {
            best_enemy = Some((target.pos, target.health));
        } else {
            // If multiple, pick lowest health
            let (enemy_pos, enemy_health) = best_enemy.unwrap();
            if target.health < enemy_health
                || (target.health == enemy_health
                    && target.pos.get_index(width) < enemy_pos.get_index(width))
            {
                best_enemy = Some((target.pos, target.health));
            }
        }
    }

    if best_enemy.is_some() {
        let (enemy_pos, _) = best_enemy.unwrap();
        let enemy = units.get_mut(&enemy_pos).unwrap();
        let attack_damage = match playing_team_elf {
            true => elf_power,
            false => goblin_power,
        };

        if enemy.health > attack_damage {
            enemy.health -= attack_damage;
        } else {
            if enemy.team_elf {
                *dead_elves += 1;
            }
            units.remove(&enemy_pos);
            unit_positions[enemy_pos.get_index(width)] = false;
        }

        true
    } else {
        false
    }
}

fn game_over(units: HashMap<Pos, Unit>) -> bool {
    let mut elves: u8 = 0;
    let mut goblins: u8 = 0;
    for unit in units.values() {
        match unit.team_elf {
            true => elves += 1,
            false => goblins += 1,
        };
    }

    return elves == 0 || goblins == 0
}

fn move_towards_an_enemy(
    current_pos: &mut Pos,
    playing_team_elf: bool,
    units: &mut HashMap<Pos, Unit>, 
    unit_positions: &mut Vec<bool>,
    walls: Vec<bool>, 
    width: usize, 
    height: usize,
) {
    let valid_positions_adjacent_to_an_enemy: HashSet<Pos> = units
        .iter()
        .filter(|(_, unit)| unit.team_elf != playing_team_elf)
        .flat_map(|(_, unit)| unit.pos.adjacency_iter(width, height))
        .filter(|pos| {
            !walls[pos.get_index(width)] && !unit_positions[pos.get_index(width)]
        })
        .collect();

    let mut parents: Vec<Pos> = vec![Pos::new(-1, -1); width * height];
    let mut best_distance = std::u16::MAX;
    let mut closest_nodes: Vec<PathfindNode> = Vec::new();
    let mut visited: Vec<bool> = vec![false; width * height];

    // node structure to perform breadth-first search
    let mut nodes: VecDeque<PathfindNode> = current_pos
        .adjacency_iter(width, height)
        .filter(|pos| {
            !walls[pos.get_index(width)] && !unit_positions[pos.get_index(width)]
        })
        .map(|pos| {
            let idx = pos.get_index(width);
            parents[idx] = *current_pos;
            visited[idx] = true;
            PathfindNode::new(pos, *current_pos, 1)
        })
        .collect();

    // perform BFS
    while !nodes.is_empty() {
        let node = nodes.pop_front().unwrap();

        if node.cost > best_distance {
            break;
        }

        if valid_positions_adjacent_to_an_enemy.contains(&node.pos) {
            best_distance = node.cost;
            closest_nodes.push(node);
        } else {
            for pos in node.pos.adjacency_iter(width, height) {
                let idx = pos.get_index(width);

                if walls[idx] || visited[idx] || units.contains_key(&pos) {
                    continue;
                }

                visited[idx] = true;
                parents[idx] = node.pos;

                nodes.push_back(PathfindNode::new(pos, node.pos, node.cost + 1));
            }
        }
    }

    let mut path: Vec<Pos> = Vec::new();

    if !closest_nodes.is_empty() {
        let mut new_pos = closest_nodes
            .iter()
            .min_by_key(|node| node.pos.get_index(width))
            .unwrap()
            .pos;

        loop {
            path.push(new_pos);
            let parent_pos = parents[new_pos.get_index(width)];
            if parent_pos == *current_pos {
                break;
            } else {
                assert!(new_pos != parent_pos);
                new_pos = parent_pos;
            }
        }

        let mut unit = units.remove(&current_pos).unwrap();
        unit_positions[current_pos.get_index(width)] = false;
        unit.pos = new_pos;
        units.insert(new_pos, unit);
        *current_pos = new_pos;
        unit_positions[current_pos.get_index(width)] = true;
    }
}

fn run(input: &str) -> (i64, i64) {
    let mut part1: i64 = 0;
    let part2: i64;

    let width = input.lines().next().unwrap().len();
    let height = input.lines().count();

    let mut walls: Vec<bool> = Vec::with_capacity(width * height);
    let mut initial_units: HashMap<Pos, Unit> = HashMap::default();

    let default_health: u8 = 200;
    let goblin_power: u8 = 3;
    let mut elf_power: u8 = 3;

    let mut next_id = 0;

    for (row, line) in input.lines().enumerate() {
        for (col, c) in line.chars().enumerate() {
            match c {
                '.' => walls.push(false),
                '#' => walls.push(true),
                'E' | 'G' => {
                    walls.push(false);
                    let pos = Pos {
                        row: row as i8,
                        col: col as i8,
                    };
                    let new_unit = Unit {
                        id: next_id,
                        team_elf: c == 'E',
                        health: default_health,
                        turn: 0,
                        pos,
                    };
                    initial_units.insert(pos, new_unit);
                    next_id += 1;
                }
                _ => panic!("Unexpected character [{}]", c),
            };
        }
    }

    let mut initial_unit_positions: Vec<bool> = vec![false; width * height];
    initial_units
        .keys()
        .for_each(|p| initial_unit_positions[p.get_index(width)] = true);

    let mut game_attempts: i64 = 0;
    loop {
        let mut units = initial_units.clone();
        let mut unit_positions = initial_unit_positions.clone();

        let mut current_turn: i64 = 0;
        let mut dead_elves: i64 = 0;

        // Iterate until a team wins
        loop {
            // If there's no chance we can win this loop, punch out.
            // Pruning the search space gives us a big performance boost.
            // This is just like Not Enough Minerals; we move faster by doing less.
            if game_attempts > 0 && dead_elves > 0 {
                break;
            }

            // Roll for initiative (it's actually deterministic)
            let first_unit_pos = units
                .values()
                .min_by_key(|u| u.get_key(width, height))
                .unwrap()
                .pos;

            // Update turn counter
            if units[&first_unit_pos].turn > current_turn {
                current_turn += 1;

            }

            if game_over(units.clone()) {
                break;
            }

            let current_unit = units.get_mut(&first_unit_pos).unwrap();
            current_unit.turn += 1;
            let current_team_elf = current_unit.team_elf;
            let mut current_pos = current_unit.pos;

            // We can attack and done
            let performed_attack = attack_adjacent_enemy_if_able(
                current_pos,
                current_team_elf,
                &mut units,
                &mut unit_positions,
                width,
                height,
                elf_power,
                goblin_power,
                &mut dead_elves,
            );

            if performed_attack {
                continue;
            }

            // We can move
            move_towards_an_enemy(&mut current_pos, current_team_elf, &mut units, &mut unit_positions, walls.clone(), width, height);

            // We can try to attack after move
            attack_adjacent_enemy_if_able(
                current_pos,
                current_team_elf,
                &mut units,
                &mut unit_positions,
                width,
                height,
                elf_power,
                goblin_power,
                &mut dead_elves,
            );
        }

        let health_sum: i64 = units
            .iter()
            .map(|(_, unit)| unit.health as i64)
            .sum::<i64>();
        let final_score = health_sum * current_turn;

        if game_attempts == 0 {
            part1 = final_score.into();
        }

        if dead_elves == 0 {
            part2 = final_score.into();
            break;
        }

        elf_power += 1;
        game_attempts += 1;
    }

    (part1, part2)
}


pub fn fn1(input: &str) -> i64 {
    // FIXME: So we do this pattern a lot across solutions, but of course we've often inlined the part 2 solve for efficiency.
    // It would be good to refactor all fn1/fn2 to return tuples and have an fnSolve as that would halve work in those cases.
    let (part1, _) = run(input);
    part1
}

pub fn fn2(input: &str) -> i64 {
    let (_, part2) = run(input);
    part2
}

#[cfg(test)]
mod tests {
    use super::*;
    use svutils::scaffold_test;

    const YEAR: i16 = 2018;
    const DAY: i16 = 15;

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
