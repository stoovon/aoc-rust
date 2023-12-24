extern crate core;

use std::collections::HashMap;

fn tilt_north(grid: &mut Vec<Vec<char>>) {
    for row in 0..grid.len() {
        for col in 0..grid[row].len() {
            if grid[row][col] == 'O' {
                let mut new_resting_row = row;
                while new_resting_row > 0 && grid[new_resting_row - 1][col] == '.' {
                    // Simple swap
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
    // These are simple simulations. Could refine.
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

    // Originally I called these repetitions but "spin cycle" was a bit of a hint that cycle was a better name,
    // and of course that's the mathematical term for the strategy we're doing: find cycles, then modulo the
    // balance (since we won't improve on that even if we keep going until the heat death of the universe)
    let mut cycle_found_at = 0;

    // Cycle length
    // It's <a low number> in example
    // It's only <about 10x more, and significantly under 1,000> in solution
    // (Numbers obfuscated so I don't give the game away)

    // Then we reduce our work significantly.
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
                cache.insert(grid_fingerprint, j); // <about half of low number> in example, <big number less about twice this> in input (numbers obfuscated)

                // Could probably be a lot smarter than this, but makes sense to burn a few cycles and collect 
                // grid fingerprints again on the assumption that the cycle repeats. Debugging revealed this 
                // was always a reasonable number but we do lose a little performance here.
                continue;
            }

            // This bit is then one-shot
            let mut remaining = 1000000000 - j - 1;

            // ALL HAIL THE MIGHTY MODULUS OPERATOR. This time warps us through the simulation by collapsing all the repeats.
            remaining %= j - cycle_found_at;

            for _ in 0..remaining {
                // This is a bit pants; I would have hoped the problem would ask for the max north load seen but we only want the nth load.
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
    use svutils::load_spec;

    #[test]
    fn fn1_test_example() {
        assert_eq!(fn1(include_str!("example.txt")), load_spec(include_str!("example-spec.1.txt")));
    }

    #[test]
    fn test_fn1_test_case_name() {
        assert_eq!(fn1(include_str!("input.txt")), load_spec(include_str!("input-spec.1.txt")));
    }

    #[test]
    fn fn2_test_example() {
        assert_eq!(fn2(include_str!("example.txt")), load_spec(include_str!("example-spec.2.txt")));
    }

    #[test]
    fn test_fn2_test_case_name() {
        assert_eq!(fn2(include_str!("input.txt")), load_spec(include_str!("input-spec.2.txt")));
    }
}