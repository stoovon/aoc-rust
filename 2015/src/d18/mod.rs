use std::str::FromStr;
use svutils::SolveError;

const STEPS: usize = 100;
const MAX_CELLS: usize = 1_000_000;
const MAX_CELL_UPDATES: usize = 10_000_000;

struct Grid {
    cells: Vec<bool>,
    rows: usize,
    cols: usize,
}

impl FromStr for Grid {
    type Err = SolveError;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        if input.trim().is_empty() {
            return Err(SolveError::EmptyInput);
        }
        let rows = input.lines().count();
        let cols = input.lines().next().map_or(0, str::len);
        if cols == 0 {
            return Err(SolveError::InvalidInput("grid rows cannot be empty".into()).at_line(1));
        }
        let area = rows
            .checked_mul(cols)
            .filter(|&area| area <= MAX_CELLS)
            .ok_or(SolveError::ResourceLimit("light grid size budget exceeded"))?;
        let mut cells = Vec::new();
        cells
            .try_reserve_exact(area)
            .map_err(|_| SolveError::ResourceLimit("could not allocate light grid"))?;
        for (index, line) in input.lines().enumerate() {
            if line.len() != cols {
                return Err(
                    SolveError::InvalidInput("expected a rectangular light grid".into())
                        .at_line(index + 1),
                );
            }
            for byte in line.bytes() {
                cells.push(match byte {
                    b'#' => true,
                    b'.' => false,
                    _ => {
                        return Err(SolveError::InvalidInput(
                            "expected only '#' and '.' cells".into(),
                        )
                        .at_line(index + 1))
                    }
                });
            }
        }
        Ok(Self { cells, rows, cols })
    }
}

impl Grid {
    fn corners_on(&mut self) {
        for corner in [
            0,
            self.cols - 1,
            (self.rows - 1) * self.cols,
            self.cells.len() - 1,
        ] {
            self.cells[corner] = true;
        }
    }

    fn step_into(&self, next: &mut [bool]) {
        for row in 0..self.rows {
            for col in 0..self.cols {
                let mut neighbours = 0u8;
                for other_row in row.saturating_sub(1)..=(row + 1).min(self.rows - 1) {
                    for other_col in col.saturating_sub(1)..=(col + 1).min(self.cols - 1) {
                        if (other_row, other_col) != (row, col) {
                            neighbours += u8::from(self.cells[other_row * self.cols + other_col]);
                        }
                    }
                }
                let index = row * self.cols + col;
                next[index] = neighbours == 3 || (self.cells[index] && neighbours == 2);
            }
        }
    }

    fn advance(&mut self, steps: usize, stuck_corners: bool) -> Result<(), SolveError> {
        let work = self
            .cells
            .len()
            .checked_mul(steps)
            .filter(|&work| work <= MAX_CELL_UPDATES)
            .ok_or(SolveError::ResourceLimit(
                "light simulation work budget exceeded",
            ))?;
        if stuck_corners {
            self.corners_on();
        }
        if work == 0 {
            return Ok(());
        }
        let mut next = Vec::new();
        next.try_reserve_exact(self.cells.len())
            .map_err(|_| SolveError::ResourceLimit("could not allocate next light grid"))?;
        next.resize(self.cells.len(), false);
        for _ in 0..steps {
            self.step_into(&mut next);
            std::mem::swap(&mut self.cells, &mut next);
            if stuck_corners {
                self.corners_on();
            }
        }
        Ok(())
    }

    fn count(&self) -> usize {
        self.cells.iter().filter(|&&on| on).count()
    }
}

fn solve(input: &str, stuck_corners: bool) -> Result<String, SolveError> {
    let mut grid: Grid = input.parse()?;
    grid.advance(STEPS, stuck_corners)?;
    Ok(grid.count().to_string())
}

pub fn part1(input: &str) -> Result<String, SolveError> {
    solve(input, false)
}
pub fn part2(input: &str) -> Result<String, SolveError> {
    solve(input, true)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::{HashMap, HashSet};
    use svutils::scaffold_test_result;

    fn sparse_reference(
        mut on: HashSet<(i32, i32)>,
        rows: i32,
        cols: i32,
        steps: usize,
        stuck: bool,
    ) -> HashSet<(i32, i32)> {
        let corners = [(0, 0), (0, cols - 1), (rows - 1, 0), (rows - 1, cols - 1)];
        if stuck {
            on.extend(corners);
        }
        for _ in 0..steps {
            let mut counts = HashMap::new();
            for &(row, col) in &on {
                for dy in -1..=1 {
                    for dx in -1..=1 {
                        let cell = (row + dy, col + dx);
                        if (dy, dx) != (0, 0)
                            && cell.0 >= 0
                            && cell.0 < rows
                            && cell.1 >= 0
                            && cell.1 < cols
                        {
                            *counts.entry(cell).or_insert(0usize) += 1;
                        }
                    }
                }
            }
            on = counts
                .into_iter()
                .filter_map(|(cell, count)| {
                    (count == 3 || (count == 2 && on.contains(&cell))).then_some(cell)
                })
                .collect();
            if stuck {
                on.extend(corners);
            }
        }
        on
    }

    #[test]
    fn all_small_rectangles_match_sparse_simulation() {
        let mut cases = 0;
        for rows in 1..=3 {
            for cols in 1..=3 {
                for bits in 0..1usize << (rows * cols) {
                    let mut input = String::new();
                    let mut on = HashSet::new();
                    for row in 0..rows {
                        for col in 0..cols {
                            let lit = bits & (1 << (row * cols + col)) != 0;
                            input.push(if lit { '#' } else { '.' });
                            if lit {
                                on.insert((row as i32, col as i32));
                            }
                        }
                        input.push('\n');
                    }
                    for steps in 0..=4 {
                        for stuck in [false, true] {
                            let expected = sparse_reference(
                                on.clone(),
                                rows as i32,
                                cols as i32,
                                steps,
                                stuck,
                            );
                            let mut actual: Grid = input.parse().unwrap();
                            actual.advance(steps, stuck).unwrap();
                            for row in 0..rows {
                                for col in 0..cols {
                                    assert_eq!(
                                        actual.cells[row * cols + col],
                                        expected.contains(&(row as i32, col as i32))
                                    );
                                }
                            }
                            assert_eq!(actual.count(), expected.len());
                            cases += 1;
                        }
                    }
                }
            }
        }
        assert_eq!(cases, 6820);
    }

    #[test]
    fn corner_policy_applies_at_zero_steps_and_collapsed_corners() {
        for (input, expected) in [
            (".", 1),
            ("..", 2),
            (".\n.", 2),
            ("...", 2),
            ("..\n..", 4),
            ("...\n...\n...", 4),
        ] {
            let mut grid: Grid = input.parse().unwrap();
            grid.advance(0, true).unwrap();
            assert_eq!(grid.count(), expected);
        }
        assert_eq!(part1(".").unwrap(), "0");
        assert_eq!(part2(".").unwrap(), "1");
    }

    #[test]
    fn parser_rejects_nonrectangular_and_invalid_cells() {
        for solver in [part1, part2] {
            for empty in ["", " \r\n"] {
                assert!(matches!(solver(empty), Err(SolveError::EmptyInput)));
            }
            for bad in ["#", "###", "", "#x", "# ", "é"] {
                let input = format!("..\n{bad}\n");
                assert!(
                    solver(&input)
                        .unwrap_err()
                        .to_string()
                        .starts_with("line 2:"),
                    "{bad:?}"
                );
            }
            assert!(solver("\n..")
                .unwrap_err()
                .to_string()
                .starts_with("line 1:"));
        }
        let grid: Grid = ".#.\r\n#..\r\n".parse().unwrap();
        assert_eq!((grid.rows, grid.cols, grid.count()), (2, 3, 2));
    }

    #[test]
    fn size_and_work_budgets_return_errors_before_simulation() {
        let oversized = ".".repeat(MAX_CELLS + 1);
        assert!(matches!(
            oversized.parse::<Grid>(),
            Err(SolveError::ResourceLimit(_))
        ));
        let mut grid: Grid = "..".parse().unwrap();
        assert!(matches!(
            grid.advance(MAX_CELL_UPDATES, false),
            Err(SolveError::ResourceLimit(_))
        ));
        assert!(matches!(
            grid.advance(usize::MAX, false),
            Err(SolveError::ResourceLimit(_))
        ));
        assert_eq!(grid.count(), 0);
    }

    #[test]
    fn test_part1_input() {
        scaffold_test_result(2015, 18, "input.txt", "input-spec.1.txt", part1);
    }
    #[test]
    fn test_part2_input() {
        scaffold_test_result(2015, 18, "input.txt", "input-spec.2.txt", part2);
    }
}
