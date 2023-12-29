extern crate core;

use std::collections::HashMap;
use std::str::FromStr;

#[derive(Debug)]
struct Grid {
    grid: HashMap<(isize, isize), bool>,
    rows: isize,
    cols: isize,
}

impl FromStr for Grid {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut lights: Grid =Self {
            grid: HashMap::new(),
            rows: s.lines().count() as isize,
            cols: s.lines().next().unwrap().chars().count() as isize,
        };
        
        for (row, line) in s.lines().enumerate() {
            for (col, ch) in line.chars().enumerate() {
                let value = match ch {
                    '#' => true,
                    '.' => false,
                    _ => panic!("Invalid character: {}", ch),
                };
                lights.grid.insert((row as isize, col as isize), value);
            }
        }

        Ok(lights)
    }
}

impl Grid {
    fn next_perfect(&mut self) {
        let mut next_grid = HashMap::new();
        for row in 0..self.rows {
            for col in 0..self.cols {
                let mut neighbours = 0;
                for x in -1..=1 {
                    for y in -1..=1 {
                        if x == 0 && y == 0 {
                            // Don't count ourselves as a neighbour.
                            continue;
                        }
                        let value = self.grid.get(&(row + x, col + y)).unwrap_or(&false);
                        if *value {
                            neighbours += 1;
                        }
                    }
                }
                let value = self.grid.get(&(row, col)).unwrap_or(&false);
                let next_value = match (value, neighbours) {
                    (true, 2) | (true, 3) | (false, 3) => true,
                    _ => false,
                };
                next_grid.insert((row, col), next_value);
            }
        }
        self.grid = next_grid;
    }

    fn corners_on(&mut self) {
        self.grid.insert((0 as isize, 0 as isize), true);
        self.grid.insert((0 as isize, self.cols - 1 as isize), true);
        self.grid.insert((self.rows - 1 as isize, 0 as isize), true);
        self.grid.insert((self.rows - 1 as isize, self.cols - 1 as isize), true);
    }

    fn next_flawed(&mut self) {
        let mut next_grid = HashMap::new();
        
        for row in 0..self.rows {
            for col in 0..self.cols {
                let mut count = 0;
                for x in -1..=1 {
                    for y in -1..=1 {
                        if x == 0 && y == 0 {
                            continue;
                        }
                        let value = self.grid.get(&(row + x, col + y)).unwrap_or(&false);
                        if *value {
                            count += 1;
                        }
                    }
                }
                let value = self.grid.get(&(row, col)).unwrap_or(&false);
                let next_value = match (value, count) {
                    (true, 2) | (true, 3) | (false, 3) => true,
                    _ => false,
                };
                next_grid.insert((row, col), next_value);
                next_grid.insert((0 as isize, 0 as isize), true);
                next_grid.insert((0 as isize, self.cols - 1 as isize), true);
                next_grid.insert((self.rows - 1 as isize, 0 as isize), true);
                next_grid.insert((self.rows - 1 as isize, self.cols - 1 as isize), true);
            }
        }

        self.grid = next_grid;
        self.corners_on()
    }

    fn count(&self) -> usize {
        self.grid.values().filter(|&&v| v).count()
    }
}

pub fn fn1(input: &str, steps: i8) -> i64 {
    let mut grid: Grid = input.parse().unwrap();

    for _ in 0..steps {
        grid.next_perfect();
    }

    grid.count() as i64
}

pub fn fn2(input: &str, steps: i8) -> i64 {
    let mut grid: Grid = input.parse().unwrap();

    grid.corners_on();

    for _ in 0..steps {
        grid.next_flawed();
    }

    grid.count() as i64
}

#[cfg(test)]
mod tests {
    use super::*;
    use svutils::scaffold_test;

    const YEAR: i16 = 2015;
    const DAY: i16 = 18;

    #[test]
    fn test_fn1_example() {
        scaffold_test(YEAR, DAY, "example.txt", "example-spec.1.txt", |input| { fn1(input, 4) });
    }

    #[test]
    fn test_fn1_input() {
        scaffold_test(YEAR, DAY, "input.txt", "input-spec.1.txt", |input| { fn1(input, 100) });
    }

    #[test]
    fn test_fn2_example() {
        scaffold_test(YEAR, DAY, "example.txt", "example-spec.2.txt", |input| { fn2(input, 5) });
    }

    #[test]
    fn test_fn2_input() {
        scaffold_test(YEAR, DAY, "input.txt", "input-spec.2.txt", |input| {fn2(input, 100) });
    }
}
