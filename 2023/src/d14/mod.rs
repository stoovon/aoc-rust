use std::collections::HashMap;
use svutils::SolveError;

#[derive(Clone, Debug, PartialEq, Eq)]
struct Platform {
    width: usize,
    height: usize,
    cells: Vec<u8>,
}

impl Platform {
    fn parse(input: &str) -> Result<Self, SolveError> {
        if input.trim().is_empty() {
            return Err(SolveError::EmptyInput);
        }
        let mut cells = Vec::new();
        let mut width = None;
        let mut height = 0;
        let mut has_round_rock = false;
        for (index, line) in input.lines().enumerate() {
            let invalid =
                |message: &str| SolveError::InvalidInput(message.into()).at_line(index + 1);
            if line.is_empty() {
                return Err(invalid("platform row is empty"));
            }
            if line.len() != *width.get_or_insert(line.len()) {
                return Err(invalid("platform rows must have equal width"));
            }
            if !line.bytes().all(|tile| matches!(tile, b'.' | b'#' | b'O')) {
                return Err(invalid("platform accepts only '.', '#' and 'O'"));
            }
            has_round_rock |= line.contains('O');
            cells.extend_from_slice(line.as_bytes());
            height += 1;
        }
        if !has_round_rock {
            return Err(SolveError::InvalidInput(
                "platform must contain at least one round rock".into(),
            ));
        }
        // Nonempty input and at least one rock guarantee a nonzero width.
        Ok(Self {
            width: cells.len() / height,
            height,
            cells,
        })
    }

    fn tilt_north(&mut self) {
        for col in 0..self.width {
            let mut destination = 0;
            for row in 0..self.height {
                let index = row * self.width + col;
                match self.cells[index] {
                    b'#' => destination = row + 1,
                    b'O' => {
                        if row != destination {
                            self.cells[destination * self.width + col] = b'O';
                            self.cells[index] = b'.';
                        }
                        destination += 1;
                    }
                    _ => {}
                }
            }
        }
    }

    fn tilt_west(&mut self) {
        for row in self.cells.chunks_exact_mut(self.width) {
            let mut destination = 0;
            for col in 0..row.len() {
                match row[col] {
                    b'#' => destination = col + 1,
                    b'O' => {
                        if col != destination {
                            row[destination] = b'O';
                            row[col] = b'.';
                        }
                        destination += 1;
                    }
                    _ => {}
                }
            }
        }
    }

    fn tilt_south(&mut self) {
        for col in 0..self.width {
            // Exclusive destination avoids subtracting below zero at the top edge.
            let mut destination = self.height;
            for row in (0..self.height).rev() {
                let index = row * self.width + col;
                match self.cells[index] {
                    b'#' => destination = row,
                    b'O' => {
                        destination -= 1;
                        if row != destination {
                            self.cells[destination * self.width + col] = b'O';
                            self.cells[index] = b'.';
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    fn tilt_east(&mut self) {
        for row in self.cells.chunks_exact_mut(self.width) {
            let mut destination = row.len();
            for col in (0..row.len()).rev() {
                match row[col] {
                    b'#' => destination = col,
                    b'O' => {
                        destination -= 1;
                        if col != destination {
                            row[destination] = b'O';
                            row[col] = b'.';
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    fn spin_cycle(&mut self) {
        self.tilt_north();
        self.tilt_west();
        self.tilt_south();
        self.tilt_east();
    }

    fn run_cycles(&mut self, target: usize) {
        let mut first_seen: HashMap<Vec<u8>, usize> = HashMap::new();
        // Dimensions stay fixed; the row-major bytes identify the complete state.
        first_seen.insert(self.cells.clone(), 0);
        for completed in 1..=target {
            self.spin_cycle();
            // Borrow the current bytes for lookup; clone only for a new snapshot.
            if let Some(&previous) = first_seen.get(self.cells.as_slice()) {
                let period = completed - previous;
                let remaining = (target - completed) % period;
                for _ in 0..remaining {
                    self.spin_cycle();
                }
                return;
            }
            first_seen.insert(self.cells.clone(), completed);
        }
    }

    fn north_load(&self) -> Result<i64, SolveError> {
        let mut load = 0_i64;
        for (index, row) in self.cells.chunks_exact(self.width).enumerate() {
            let weight =
                i64::try_from(self.height - index).map_err(|_| SolveError::ArithmeticOverflow)?;
            let rocks = i64::try_from(row.iter().filter(|&&tile| tile == b'O').count())
                .map_err(|_| SolveError::ArithmeticOverflow)?;
            let row_load = weight
                .checked_mul(rocks)
                .ok_or(SolveError::ArithmeticOverflow)?;
            load = load
                .checked_add(row_load)
                .ok_or(SolveError::ArithmeticOverflow)?;
        }
        Ok(load)
    }
}

pub fn part1(input: &str) -> Result<String, SolveError> {
    let mut platform = Platform::parse(input)?;
    platform.tilt_north();
    Ok(platform.north_load()?.to_string())
}

pub fn part2(input: &str) -> Result<String, SolveError> {
    let mut platform = Platform::parse(input)?;
    platform.run_cycles(1_000_000_000);
    Ok(platform.north_load()?.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use svutils::scaffold_test_result;

    fn platform_from_grid(grid: &[Vec<char>]) -> Platform {
        Platform {
            width: grid[0].len(),
            height: grid.len(),
            cells: grid.iter().flatten().map(|&tile| tile as u8).collect(),
        }
    }

    const YEAR: i16 = 2023;
    const DAY: i16 = 14;

    #[test]
    fn test_part1_input() {
        scaffold_test_result(YEAR, DAY, "input.txt", "input-spec.1.txt", part1);
    }

    #[test]
    fn test_part2_input() {
        scaffold_test_result(YEAR, DAY, "input.txt", "input-spec.2.txt", part2);
    }

    #[test]
    fn skipped_cycles_match_direct_simulation_on_small_grids() {
        for mut encoding in 0..3_usize.pow(6) {
            let mut initial = vec![vec!['.'; 3]; 2];
            for row in &mut initial {
                for tile in row {
                    *tile = ['.', '#', 'O'][encoding % 3];
                    encoding /= 3;
                }
            }
            let initial = platform_from_grid(&initial);
            let mut direct = initial.clone();
            for cycles in 0..=20 {
                let mut skipped = initial.clone();
                skipped.run_cycles(cycles);
                assert_eq!(skipped, direct, "target={cycles}, initial={initial:?}");
                direct.spin_cycle();
            }
        }
    }

    #[test]
    fn a_fixed_point_can_skip_a_billion_cycles() {
        let mut platform = Platform::parse("O").unwrap();
        platform.run_cycles(1_000_000_000);
        assert_eq!(platform.cells, b"O");
    }

    #[test]
    fn malformed_or_rock_free_platforms_are_rejected() {
        for input in ["", " \t\r\n"] {
            assert!(matches!(part1(input), Err(SolveError::EmptyInput)));
            assert!(matches!(part2(input), Err(SolveError::EmptyInput)));
        }
        for input in [
            ".", "###\n...", "O.\n.", "\nO", "O\n\nO", "O\n\n", "Ox", "O ", "Oé",
        ] {
            assert!(part1(input).is_err(), "{input:?}");
            assert!(part2(input).is_err(), "{input:?}");
        }
    }

    #[test]
    fn narrow_platforms_and_normal_line_endings_are_supported() {
        for input in ["O", "O\n", "O\r\n"] {
            assert_eq!(part1(input).unwrap(), "1");
            assert_eq!(part2(input).unwrap(), "1");
        }
        assert_eq!(part1("O.O").unwrap(), "2");
        assert_eq!(part2("O.O").unwrap(), "2");
        assert_eq!(part1("O\n.\nO").unwrap(), "5");
        assert_eq!(part2("O\n.\nO").unwrap(), "3");
    }

    fn direct_tilt(grid: &mut [Vec<char>], dx: isize, dy: isize) {
        // Independent reference: move every rock one cell, repeating until stable.
        loop {
            let mut moved = false;
            for y in 0..grid.len() {
                for x in 0..grid[0].len() {
                    if grid[y][x] != 'O' {
                        continue;
                    }
                    let nx = x as isize + dx;
                    let ny = y as isize + dy;
                    if nx < 0 || ny < 0 || nx >= grid[0].len() as isize || ny >= grid.len() as isize
                    {
                        continue;
                    }
                    let (nx, ny) = (nx as usize, ny as usize);
                    if grid[ny][nx] == '.' {
                        grid[ny][nx] = 'O';
                        grid[y][x] = '.';
                        moved = true;
                    }
                }
            }
            if !moved {
                break;
            }
        }
    }

    #[test]
    fn compact_tilts_match_direct_movement_on_small_rectangles() {
        type Tilt = fn(&mut Platform);
        let directions: [(Tilt, isize, isize); 4] = [
            (Platform::tilt_north, 0, -1),
            (Platform::tilt_west, -1, 0),
            (Platform::tilt_south, 0, 1),
            (Platform::tilt_east, 1, 0),
        ];
        for (height, width) in [(1, 1), (1, 4), (4, 1), (2, 3), (3, 2)] {
            for mut encoding in 0..3_usize.pow((height * width) as u32) {
                let mut initial = vec![vec!['.'; width]; height];
                for row in &mut initial {
                    for tile in row {
                        *tile = ['.', '#', 'O'][encoding % 3];
                        encoding /= 3;
                    }
                }
                for &(tilt, dx, dy) in &directions {
                    let mut expected = initial.clone();
                    direct_tilt(&mut expected, dx, dy);
                    let expected = platform_from_grid(&expected);
                    let mut actual = platform_from_grid(&initial);
                    tilt(&mut actual);
                    assert_eq!(
                        actual, expected,
                        "initial={initial:?}, direction=({dx},{dy})"
                    );
                    tilt(&mut actual);
                    assert_eq!(actual, expected, "tilting twice must be idempotent");
                }
            }
        }
    }

    #[test]
    fn snapshots_are_independent_exact_row_major_bytes() {
        let mut platform = Platform::parse(".O#\nO..").unwrap();
        let snapshot = platform.cells.clone();
        assert_eq!(snapshot, b".O#O..");
        platform.tilt_north();
        assert_eq!(platform.cells, b"OO#...");
        assert_eq!(snapshot, b".O#O..");
    }

    #[test]
    fn skipping_preserves_a_transient_then_a_two_cycle_period() {
        let initial = Platform::parse("...\n..#\nOOO").unwrap();
        let mut direct = initial.clone();
        let mut states = vec![direct.clone()];
        for _ in 0..4 {
            direct.spin_cycle();
            states.push(direct.clone());
        }
        assert_ne!(states[0], states[1]);
        assert_ne!(states[1], states[2]);
        assert_eq!(states[1], states[3]);
        for target in [0, 1, 2, 3, 4, 99, 100, 1_000_000_000] {
            let expected = if target == 0 {
                &states[0]
            } else {
                &states[1 + (target - 1) % 2]
            };
            let mut actual = initial.clone();
            actual.run_cycles(target);
            assert_eq!(&actual, expected);
        }
    }
}
