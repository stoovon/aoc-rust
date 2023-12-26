extern crate core;

use std::collections::HashSet;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
enum Direction {
    Left,
    Right,
    Up,
    Down,
}

#[derive(Debug)]
struct Grid {
    device: Vec<Vec<char>>,
    components: HashSet<(usize, usize, Direction)>,
    active: HashSet<(usize, usize)>,
}

impl Grid {
    fn new(grid: Vec<Vec<char>>) -> Grid {
        Grid {
            device: grid,
            components: HashSet::new(),
            active: HashSet::new(),
        }
    }
    fn beam(&mut self, mut direction: Direction, mut col: usize, mut row: usize) {
        loop {
            if !self.components.insert((row, col, direction.clone())) {
                break;
            }
            self.active.insert((row, col));
            
            match self.device[row][col] {
                '/' => direction = match direction {
                    Direction::Up => Direction::Right,
                    Direction::Right => Direction::Up,
                    Direction::Down => Direction::Left,
                    Direction::Left => Direction::Down,
                },
                '\\' => direction = match direction {
                    Direction::Up => Direction::Left,
                    Direction::Right => Direction::Down,
                    Direction::Down => Direction::Right,
                    Direction::Left => Direction::Up,
                },
                '-' => if direction == Direction::Up || direction == Direction::Down {
                    self.beam(Direction::Right, col, row);
                    direction = Direction::Left;
                },
                '|' => if direction == Direction::Left || direction == Direction::Right {
                    self.beam(Direction::Up, col, row);
                    direction = Direction::Down;
                },
                _ => {},
            }
    
            match direction {
                Direction::Up if row > 0 => row -= 1,
                Direction::Right if col < self.device[0].len() - 1 => col += 1,
                Direction::Down if row < self.device.len() - 1 => row += 1,
                Direction::Left if col > 0 => col -= 1,
                _ => break,
            }
        }
    }    
}

fn activation_strength(grid: &mut Grid, direction: Direction, x: usize, y: usize, max: &mut i64) {
    grid.beam(direction, x, y);
    *max = (grid.active.len() as i64).max(*max);
    grid.components.clear();
    grid.active.clear();
}

pub fn fn1(input: &str) -> i64 {
    let data: Vec<Vec<char>> = input.lines().map(|line| {
        line.chars().collect()
    }).collect();
    let mut grid = Grid::new(data);
    grid.beam(Direction::Right, 0, 0);
    grid.active.len() as i64
}

pub fn fn2(input: &str) -> i64 {
    // Performance here is slow (2 seconds). Feels like we could fairly easily cache (set and deque?).
    // Could possibly also model as vectors and conditionally flip vectors (but I think the matches are nice).
    // One to come back to.

    let data: Vec<Vec<char>> = input.lines().map(|line| line.chars().collect()).collect();
    let cols = data[0].len() - 1;
    let rows = data.len() - 1;
    let mut grid = Grid::new(data);
    let mut max = 0;

    for col in 0..cols+1 {
        activation_strength(&mut grid, Direction::Down, col, 0, &mut max);
        activation_strength(&mut grid, Direction::Up, col, rows, &mut max);
    }
    for row in 0..rows+1 {
        activation_strength(&mut grid, Direction::Right, 0, row, &mut max);
        activation_strength(&mut grid, Direction::Left, cols, row, &mut max);
    }

    max
}

#[cfg(test)]
mod tests {
    use super::*;
    use svutils::scaffold_test;

    const YEAR: i16 = 2023;
    const DAY: i16 = 16;

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
