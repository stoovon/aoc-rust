use svutils::SolveError;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
enum Direction {
    Left,
    Right,
    Up,
    Down,
}

impl Direction {
    fn bit(self) -> u8 {
        match self {
            Self::Right => 0b0001, // entering from the left
            Self::Left => 0b0010,
            Self::Down => 0b0100,
            Self::Up => 0b1000,
        }
    }
}

struct Grid {
    width: usize,
    height: usize,
    cells: Vec<u8>,
}

impl Grid {
    fn parse(input: &str) -> Result<Self, SolveError> {
        if input.trim().is_empty() {
            return Err(SolveError::EmptyInput);
        }
        let mut cells = Vec::new();
        let mut width = None;
        let mut height = 0;
        for (index, line) in input.lines().enumerate() {
            let invalid =
                |message: &str| SolveError::InvalidInput(message.into()).at_line(index + 1);
            if line.is_empty() {
                return Err(invalid("beam grid row is empty"));
            }
            if line.len() != *width.get_or_insert(line.len()) {
                return Err(invalid("beam grid rows must have equal width"));
            }
            if !line
                .bytes()
                .all(|tile| matches!(tile, b'.' | b'/' | b'\\' | b'-' | b'|'))
            {
                return Err(invalid("beam grid contains an invalid tile"));
            }
            cells.extend_from_slice(line.as_bytes());
            height += 1;
        }
        i64::try_from(cells.len()).map_err(|_| SolveError::ArithmeticOverflow)?;
        Ok(Self {
            width: cells.len() / height,
            height,
            cells,
        })
    }
}

struct BeamTrace {
    seen: Vec<u8>,
    pending: Vec<(Direction, usize, usize)>,
}

impl BeamTrace {
    fn new(grid: &Grid) -> Self {
        Self {
            seen: vec![0; grid.cells.len()],
            pending: Vec::new(),
        }
    }
    fn energized(&mut self, grid: &Grid, direction: Direction, col: usize, row: usize) -> usize {
        self.seen.fill(0);
        self.pending.clear();
        self.pending.push((direction, col, row));
        let mut energized = 0;
        while let Some((mut direction, mut col, mut row)) = self.pending.pop() {
            loop {
                let index = row * grid.width + col;
                let bit = direction.bit();
                if self.seen[index] & bit != 0 {
                    break;
                }
                if self.seen[index] == 0 {
                    energized += 1;
                }
                self.seen[index] |= bit;

                match grid.cells[index] {
                    b'/' => {
                        direction = match direction {
                            Direction::Up => Direction::Right,
                            Direction::Right => Direction::Up,
                            Direction::Down => Direction::Left,
                            Direction::Left => Direction::Down,
                        }
                    }
                    b'\\' => {
                        direction = match direction {
                            Direction::Up => Direction::Left,
                            Direction::Right => Direction::Down,
                            Direction::Down => Direction::Right,
                            Direction::Left => Direction::Up,
                        }
                    }
                    b'-' => {
                        if direction == Direction::Up || direction == Direction::Down {
                            self.pending.push((Direction::Right, col, row));
                            direction = Direction::Left;
                        }
                    }
                    b'|' => {
                        if direction == Direction::Left || direction == Direction::Right {
                            self.pending.push((Direction::Up, col, row));
                            direction = Direction::Down;
                        }
                    }
                    _ => {}
                }

                match direction {
                    Direction::Up if row > 0 => row -= 1,
                    Direction::Right if col < grid.width - 1 => col += 1,
                    Direction::Down if row < grid.height - 1 => row += 1,
                    Direction::Left if col > 0 => col -= 1,
                    _ => break,
                }
            }
        }
        energized
    }
}

pub fn part1(input: &str) -> Result<String, SolveError> {
    let grid = Grid::parse(input)?;
    let mut trace = BeamTrace::new(&grid);
    Ok(trace.energized(&grid, Direction::Right, 0, 0).to_string())
}

pub fn part2(input: &str) -> Result<String, SolveError> {
    let grid = Grid::parse(input)?;
    let mut trace = BeamTrace::new(&grid);
    let mut maximum = 0;
    for col in 0..grid.width {
        maximum = maximum.max(trace.energized(&grid, Direction::Down, col, 0));
        maximum = maximum.max(trace.energized(&grid, Direction::Up, col, grid.height - 1));
    }
    for row in 0..grid.height {
        maximum = maximum.max(trace.energized(&grid, Direction::Right, 0, row));
        maximum = maximum.max(trace.energized(&grid, Direction::Left, grid.width - 1, row));
    }
    Ok(maximum.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use svutils::scaffold_test_result;

    use std::collections::HashSet;

    struct ReferenceGrid {
        device: Vec<Vec<char>>,
        components: HashSet<(usize, usize, Direction)>,
        active: HashSet<(usize, usize)>,
    }
    impl ReferenceGrid {
        fn new(device: Vec<Vec<char>>) -> Self {
            Self {
                device,
                components: HashSet::new(),
                active: HashSet::new(),
            }
        }
        fn reference_beam(&mut self, mut direction: Direction, mut col: usize, mut row: usize) {
            loop {
                if !self.components.insert((row, col, direction)) {
                    break;
                }
                self.active.insert((row, col));

                match self.device[row][col] {
                    '/' => {
                        direction = match direction {
                            Direction::Up => Direction::Right,
                            Direction::Right => Direction::Up,
                            Direction::Down => Direction::Left,
                            Direction::Left => Direction::Down,
                        }
                    }
                    '\\' => {
                        direction = match direction {
                            Direction::Up => Direction::Left,
                            Direction::Right => Direction::Down,
                            Direction::Down => Direction::Right,
                            Direction::Left => Direction::Up,
                        }
                    }
                    '-' => {
                        if direction == Direction::Up || direction == Direction::Down {
                            self.reference_beam(Direction::Right, col, row);
                            direction = Direction::Left;
                        }
                    }
                    '|' => {
                        if direction == Direction::Left || direction == Direction::Right {
                            self.reference_beam(Direction::Up, col, row);
                            direction = Direction::Down;
                        }
                    }
                    _ => {}
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

    const YEAR: i16 = 2023;
    const DAY: i16 = 16;

    #[test]
    fn test_part1_input() {
        scaffold_test_result(YEAR, DAY, "input.txt", "input-spec.1.txt", part1);
    }

    #[test]
    fn test_part2_input() {
        scaffold_test_result(YEAR, DAY, "input.txt", "input-spec.2.txt", part2);
    }

    #[test]
    fn worklist_matches_recursive_traversal_on_small_grids() {
        let tiles = ['.', '/', '\\', '-', '|'];
        for (height, width) in [(1, 1), (1, 3), (3, 1), (2, 2)] {
            for mut encoding in 0..tiles.len().pow((height * width) as u32) {
                let mut device = vec![vec!['.'; width]; height];
                for row in &mut device {
                    for tile in row {
                        *tile = tiles[encoding % tiles.len()];
                        encoding /= tiles.len();
                    }
                }
                for row in 0..height {
                    for col in 0..width {
                        for direction in [
                            Direction::Up,
                            Direction::Down,
                            Direction::Left,
                            Direction::Right,
                        ] {
                            let mut expected = ReferenceGrid::new(device.clone());
                            expected.reference_beam(direction, col, row);
                            let input = device
                                .iter()
                                .map(|row| row.iter().collect::<String>())
                                .collect::<Vec<_>>()
                                .join("\n");
                            let grid = Grid::parse(&input).unwrap();
                            let mut trace = BeamTrace::new(&grid);
                            assert_eq!(
                                trace.energized(&grid, direction, col, row),
                                expected.active.len()
                            );
                            let active: HashSet<_> = trace
                                .seen
                                .iter()
                                .enumerate()
                                .filter(|(_, bits)| **bits != 0)
                                .map(|(index, _)| (index / width, index % width))
                                .collect();
                            assert_eq!(active, expected.active);
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn invalid_grids_are_rejected_and_normal_line_endings_are_accepted() {
        for input in ["", " \t\r\n"] {
            assert!(matches!(part1(input), Err(SolveError::EmptyInput)));
            assert!(matches!(part2(input), Err(SolveError::EmptyInput)));
        }
        for input in ["\n.", "..\n.", ".\n\n.", ".\n\n", "x", "é", ". "] {
            assert!(part1(input).is_err());
            assert!(part2(input).is_err());
        }
        for input in [".", ".\n", ".\r\n"] {
            assert_eq!(part1(input).unwrap(), "1");
            assert_eq!(part2(input).unwrap(), "1");
        }
        assert_eq!(part1("...\n...").unwrap(), "3");
        assert_eq!(part2(".\n.\n.").unwrap(), "3");
    }

    #[test]
    fn reused_trace_clears_previous_directions_and_tiles() {
        let grid = Grid::parse("...\n...").unwrap();
        let mut trace = BeamTrace::new(&grid);
        assert_eq!(trace.energized(&grid, Direction::Right, 0, 0), 3);
        assert_eq!(trace.energized(&grid, Direction::Down, 0, 0), 2);
        assert_eq!(trace.energized(&grid, Direction::Right, 0, 0), 3);
    }
}
