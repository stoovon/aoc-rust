extern crate core;

use std::collections::HashMap;

fn tilt_north(grid: &mut Vec<Vec<char>>) {
    for row in 0..grid.len() {
        for col in 0..grid[row].len() {
            if grid[row][col] == 'O' {
                let mut new_resting_row = row;
                while new_resting_row > 0 && grid[new_resting_row - 1][col] == '.' {
                    grid[new_resting_row][col] = '.';
                    grid[new_resting_row - 1][col] = 'O';
                    new_resting_row -= 1;
                }
            }
        }
    }
}

fn tilt_west(grid: &mut Vec<Vec<char>>) {
    for row in 0..grid.len() {
        for col in 0..grid[row].len() {
            if grid[row][col] == 'O' {
                let mut new_resting_col = col;
                while new_resting_col > 0 && grid[row][new_resting_col - 1] == '.' {
                    grid[row][new_resting_col] = '.';
                    grid[row][new_resting_col - 1] = 'O';
                    new_resting_col -= 1;
                }
            }
        }
    }
}

fn tilt_south(grid: &mut Vec<Vec<char>>) {
    for row in (0..grid.len()).rev() {
        for col in 0..grid[row].len() {
            if grid[row][col] == 'O' {
                let mut new_resting_row = row;
                while new_resting_row < grid.len() - 1 && grid[new_resting_row + 1][col] == '.' {
                    grid[new_resting_row][col] = '.';
                    grid[new_resting_row + 1][col] = 'O';
                    new_resting_row += 1;
                }
            }
        }
    }
}

fn tilt_east(grid: &mut Vec<Vec<char>>) {
    for row in 0..grid.len() {
        for col in (0..grid[row].len()).rev() {
            if grid[row][col] == 'O' {
                let mut new_resting_col = col;
                while new_resting_col < grid[row].len() - 1 && grid[row][new_resting_col + 1] == '.' {
                    grid[row][new_resting_col] = '.';
                    grid[row][new_resting_col + 1] = 'O';
                    new_resting_col += 1;
                }
            }
        }
    }
}

fn spin_cycle(grid: &mut Vec<Vec<char>>) {
    tilt_north(grid);
    tilt_west(grid);
    tilt_south(grid);
    tilt_east(grid);
}

fn north_load(grid: &Vec<Vec<char>>) -> usize {
    let mut load = 0;
    for row in 0..grid.len() {
        for col in 0..grid[row].len() {
            if grid[row][col] == 'O' {
                // load is delta to south edge of grid
                load += grid.len() - row;
            }
        }
    }
    load
}

pub fn fn1(input: &str) -> i64 {
    let mut grid: Vec<Vec<char>> = input.lines().map(|line| line.chars().collect()).collect();

    tilt_north(&mut grid);

    north_load(&grid) as i64
}

pub fn fn2(input: &str) -> i64 {
    let mut grid: Vec<Vec<char>> = input.lines().map(|line| line.chars().collect()).collect();
    let mut cache: HashMap<String, usize> = HashMap::new();

    let mut cycle_found_at = 0;

    for j in 0..1000 { 
        spin_cycle(&mut grid);

        let grid_fingerprint = grid
            .iter()
            .map(|row| row.iter().collect::<String>())
            .collect::<Vec<String>>()
            .join("");

        let seen_work_before = cache.contains_key(&grid_fingerprint);
        if seen_work_before {
            if cycle_found_at == 0 {
                cycle_found_at = j;
                cache.clear();
                cache.insert(grid_fingerprint, j);

                continue;
            }

            let mut remaining = 1000000000 - j - 1;

            remaining %= j - cycle_found_at;

            for _ in 0..remaining {
                spin_cycle(&mut grid);
            }

            return north_load(&grid) as i64;
        }

        cache.insert(grid_fingerprint, j);
    }
    unreachable!()
}

#[cfg(test)]
mod tests {
    use super::*;
    use svutils::scaffold_test;

    const YEAR: i16 = 2023;
    const DAY: i16 = 14;

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
