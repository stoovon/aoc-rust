use svutils::SolveError;

use std::ops::{Add, AddAssign};

const UP: Point = Point::new(0, -1);
const DOWN: Point = Point::new(0, 1);
const LEFT: Point = Point::new(-1, 0);
const RIGHT: Point = Point::new(1, 0);
const ORTHOGONAL: [Point; 4] = [UP, DOWN, LEFT, RIGHT];

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
struct Point {
    x: i32,
    y: i32,
}

impl Point {
    #[inline]
    #[must_use]
    const fn new(x: i32, y: i32) -> Self {
        Point { x, y }
    }
}

impl Add for Point {
    type Output = Self;

    #[inline]
    fn add(self, rhs: Self) -> Self {
        Point::new(self.x + rhs.x, self.y + rhs.y)
    }
}

impl AddAssign for Point {
    #[inline]
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}

struct Grid {
    width: i32,
    height: i32,
    bytes: Vec<u8>,
}

impl Grid {
    fn parse(input: &str) -> Result<Self, SolveError> {
        if input.trim().is_empty() {
            return Err(SolveError::EmptyInput);
        }
        let rows: Vec<_> = input.lines().map(str::as_bytes).collect();
        let width = rows[0].len();
        if width == 0 || rows.iter().any(|row| row.len() != width) {
            return Err(SolveError::InvalidInput(
                "grid must have nonempty rows of equal width".into(),
            ));
        }
        let height = rows.len();
        let tile_count = width
            .checked_mul(height)
            .ok_or(SolveError::ArithmeticOverflow)?;
        let width = i32::try_from(width).map_err(|_| SolveError::ArithmeticOverflow)?;
        let height = i32::try_from(height).map_err(|_| SolveError::ArithmeticOverflow)?;
        let mut bytes = Vec::with_capacity(tile_count);
        let mut starts = 0;
        for row in rows {
            for &tile in row {
                if !matches!(tile, b'.' | b'S' | b'|' | b'-' | b'L' | b'J' | b'F' | b'7') {
                    return Err(SolveError::InvalidInput(
                        "grid contains an invalid tile".into(),
                    ));
                }
                if tile == b'S' {
                    starts += 1;
                    if starts > 1 {
                        return Err(SolveError::InvalidInput(
                            "grid has multiple starting tiles".into(),
                        ));
                    }
                }
                bytes.push(tile);
            }
        }
        if starts == 0 {
            return Err(SolveError::InvalidInput("missing starting tile".into()));
        }
        Ok(Grid {
            width,
            height,
            bytes,
        })
    }

    fn find(&self, needle: u8) -> Option<Point> {
        let to_point = |index| {
            let x = (index % self.width as usize) as i32;
            let y = (index / self.width as usize) as i32;
            Point::new(x, y)
        };
        self.bytes.iter().position(|&h| h == needle).map(to_point)
    }

    fn get(&self, point: Point) -> Option<&u8> {
        if point.x < 0 || point.y < 0 || point.x >= self.width || point.y >= self.height {
            return None;
        }
        let index = (point.y as usize)
            .checked_mul(self.width as usize)?
            .checked_add(point.x as usize)?;
        self.bytes.get(index)
    }
}

fn pipe_connections(tile: u8) -> Option<[Point; 2]> {
    match tile {
        b'|' => Some([UP, DOWN]),
        b'-' => Some([LEFT, RIGHT]),
        b'L' => Some([UP, RIGHT]),
        b'J' => Some([UP, LEFT]),
        b'F' => Some([DOWN, RIGHT]),
        b'7' => Some([DOWN, LEFT]),
        _ => None,
    }
}

fn start_connections(grid: &Grid, start: Point) -> Result<[Point; 2], SolveError> {
    let connections: Vec<_> = ORTHOGONAL
        .into_iter()
        .filter(|&direction| {
            let back = Point::new(-direction.x, -direction.y);
            grid.get(start + direction)
                .and_then(|&tile| pipe_connections(tile))
                .is_some_and(|openings| openings.contains(&back))
        })
        .collect();
    connections.try_into().map_err(|_| {
        SolveError::InvalidInput("starting tile must have exactly two connecting neighbours".into())
    })
}

fn walk_loop(
    grid: &Grid,
    mut visit_edge: impl FnMut(Point, Point) -> Result<(), SolveError>,
) -> Result<i64, SolveError> {
    let start = grid
        .find(b'S')
        .ok_or_else(|| SolveError::InvalidInput("missing starting tile".into()))?;
    let openings = start_connections(grid, start)?;
    let mut direction = openings[0];
    let mut previous = start;
    let mut position = start + direction;
    let mut perimeter = 1_i64;
    let tile_limit = i64::try_from(grid.bytes.len()).map_err(|_| SolveError::ArithmeticOverflow)?;

    loop {
        let tile = grid
            .get(position)
            .copied()
            .ok_or_else(|| SolveError::InvalidInput("pipe leaves the grid".into()))?;
        visit_edge(previous, position)?;
        let back = Point::new(-direction.x, -direction.y);
        if position == start {
            if back != openings[1] {
                return Err(SolveError::InvalidInput(
                    "loop must return through the other starting connection".into(),
                ));
            }
            break;
        }
        if perimeter >= tile_limit {
            return Err(SolveError::InvalidInput(
                "pipe walk does not return to the starting tile".into(),
            ));
        }
        let connections = pipe_connections(tile)
            .ok_or_else(|| SolveError::InvalidInput("loop enters a non-pipe tile".into()))?;
        direction = match connections {
            [first, second] if first == back => second,
            [first, second] if second == back => first,
            _ => {
                return Err(SolveError::InvalidInput(
                    "pipe does not connect back to the previous tile".into(),
                ));
            }
        };
        previous = position;
        position += direction;
        perimeter = perimeter
            .checked_add(1)
            .ok_or(SolveError::ArithmeticOverflow)?;
    }

    Ok(perimeter)
}

fn shoelace_term(a: Point, b: Point) -> i128 {
    // i32 coordinates make each product and their difference fit safely in i128.
    i128::from(a.x) * i128::from(b.y) - i128::from(a.y) * i128::from(b.x)
}

fn enclosed_tiles(twice_area: i128, boundary: i64) -> Result<i64, SolveError> {
    // Pick: I = A - B/2 + 1, keeping doubled area integral throughout.
    let twice_interior = twice_area
        .checked_abs()
        .and_then(|area| area.checked_sub(i128::from(boundary)))
        .and_then(|difference| difference.checked_add(2))
        .ok_or(SolveError::ArithmeticOverflow)?;
    if twice_interior < 0 || twice_interior % 2 != 0 {
        return Err(SolveError::InvalidInput("invalid loop geometry".into()));
    }
    i64::try_from(twice_interior / 2).map_err(|_| SolveError::ArithmeticOverflow)
}

pub fn part1(input: &str) -> Result<String, SolveError> {
    let grid = Grid::parse(input)?;
    let length = walk_loop(&grid, |_, _| Ok(()))?;
    Ok((length / 2).to_string())
}

pub fn part2(input: &str) -> Result<String, SolveError> {
    let grid = Grid::parse(input)?;
    let mut twice_area = 0_i128;
    let boundary = walk_loop(&grid, |a, b| {
        twice_area = twice_area
            .checked_add(shoelace_term(a, b))
            .ok_or(SolveError::ArithmeticOverflow)?;
        Ok(())
    })?;
    Ok(enclosed_tiles(twice_area, boundary)?.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use svutils::scaffold_test_result;

    const YEAR: i16 = 2023;
    const DAY: i16 = 10;

    #[test]
    fn test_part1_input() {
        scaffold_test_result(YEAR, DAY, "input.txt", "input-spec.1.txt", part1);
    }

    #[test]
    fn test_part2_input() {
        scaffold_test_result(YEAR, DAY, "input.txt", "input-spec.2.txt", part2);
    }

    #[test]
    fn lookup_respects_both_coordinate_bounds() {
        let grid = Grid::parse("S-\nLJ").unwrap();
        assert_eq!(grid.get(Point::new(0, 0)), Some(&b'S'));
        assert_eq!(grid.get(Point::new(1, 1)), Some(&b'J'));
        for point in [
            Point::new(-1, 0),
            Point::new(0, -1),
            Point::new(2, 0),
            Point::new(0, 2),
        ] {
            assert_eq!(grid.get(point), None);
        }
    }

    #[test]
    fn start_can_be_on_the_top_edge() {
        let input = "S--7\n|..|\nL--J";
        assert_eq!(part1(input).unwrap(), "5");
        assert_eq!(part2(input).unwrap(), "2");
    }

    #[test]
    fn pipe_leaving_grid_returns_an_error() {
        let input = "S--7\n|..|\n|..|";
        assert!(matches!(part1(input), Err(SolveError::InvalidInput(_))));
        assert!(matches!(part2(input), Err(SolveError::InvalidInput(_))));
    }

    #[test]
    fn start_can_have_any_of_the_six_pipe_shapes() {
        for input in [
            "S--7\n|..|\nL--J",
            "F-S7\n|..|\nL--J",
            "F--S\n|..|\nL--J",
            "F--7\nS..|\nL--J",
            "F--7\n|..|\nS--J",
            "F--7\n|..|\nL--S",
        ] {
            assert_eq!(part1(input).unwrap(), "5");
            assert_eq!(part2(input).unwrap(), "2");
        }
        let input = "F-S7\n|..|\nL--J";
        assert_eq!(part1(input).unwrap(), "5");
        assert_eq!(part2(input).unwrap(), "2");
    }

    #[test]
    fn start_requires_exactly_two_neighbours_that_open_toward_it() {
        for input in ["S", "S-", "-|.\n-S-\n.|.", ".|.\n-S-\n.|.", "-S-\n.|."] {
            assert!(matches!(part1(input), Err(SolveError::InvalidInput(_))));
            assert!(matches!(part2(input), Err(SolveError::InvalidInput(_))));
        }
        // Adjacent pipes that face away from S do not count as connections.
        let grid = Grid::parse(".|.\n|S|\n.|.").unwrap();
        assert_eq!(
            start_connections(&grid, Point::new(1, 1)).unwrap(),
            [UP, DOWN]
        );
    }

    #[test]
    fn traversal_rejects_broken_connections_and_false_finishes() {
        for input in [
            "S--7\n|..-\nL--J", // horizontal pipe entered vertically
            "S--7\n|..7\nL--J", // corner has no opening toward its predecessor
            "S--7\n|...\nL--J", // ground is not the starting tile
            "S--7\n|..S\nL--J", // a different S is not loop closure
        ] {
            assert!(matches!(part1(input), Err(SolveError::InvalidInput(_))));
            assert!(matches!(part2(input), Err(SolveError::InvalidInput(_))));
        }
    }

    #[test]
    fn unconnected_pipes_do_not_change_the_loop() {
        let input = "S--7..\n|.F|..\nL--JF7\n....LJ";
        assert_eq!(part1(input).unwrap(), "5");
        assert_eq!(part2(input).unwrap(), "2");
    }

    #[test]
    fn parser_rejects_empty_malformed_and_ambiguous_grids() {
        for input in ["", "\n", " \t\r\n"] {
            assert!(matches!(Grid::parse(input), Err(SolveError::EmptyInput)));
        }
        for input in [
            "\nS--7",
            "S--7\n|.\nL--J",
            "S--7\n\nL--J",
            "S--7\n|..|\nL--J\n\n",
            "S--7\n|.x|\nL--J",
            "S--7\n|.é\nL--J",
            "S--7\n|. |\nL--J",
            "F--7\n|..|\nL--J",
            "S-S",
        ] {
            assert!(
                matches!(Grid::parse(input), Err(SolveError::InvalidInput(_))),
                "{input:?}"
            );
        }
    }

    #[test]
    fn parser_accepts_crlf_and_a_normal_trailing_newline() {
        for input in ["S--7\n|..|\nL--J\n", "S--7\r\n|..|\r\nL--J\r\n"] {
            assert_eq!(part1(input).unwrap(), "5");
            assert_eq!(part2(input).unwrap(), "2");
        }
    }

    #[test]
    fn geometry_handles_both_orientations_and_wide_coordinates() {
        let corners = [
            Point::new(0, 0),
            Point::new(i32::MAX, 0),
            Point::new(i32::MAX, i32::MAX),
            Point::new(0, i32::MAX),
        ];
        let area: i128 = (0..4)
            .map(|i| shoelace_term(corners[i], corners[(i + 1) % 4]))
            .sum();
        let side = i64::from(i32::MAX);
        let expected = (side - 1) * (side - 1);
        assert_eq!(enclosed_tiles(area, 4 * side).unwrap(), expected);
        assert_eq!(enclosed_tiles(-area, 4 * side).unwrap(), expected);
        assert_eq!(enclosed_tiles(2, 4).unwrap(), 0);
        assert!(matches!(
            enclosed_tiles(i128::MIN, 4),
            Err(SolveError::ArithmeticOverflow)
        ));
        assert!(matches!(
            enclosed_tiles(i128::MAX - 1, 4),
            Err(SolveError::ArithmeticOverflow)
        ));
    }

    #[test]
    fn parts_handle_a_loop_with_no_interior_tiles() {
        assert_eq!(part1("S7\nLJ").unwrap(), "2");
        assert_eq!(part2("S7\nLJ").unwrap(), "0");
    }

    #[test]
    fn concave_loop_has_the_same_area_in_either_direction() {
        // A 4x4 square with a 2x2 corner removed: area 12, boundary 16,
        // hence five interior lattice points by Pick's theorem.
        for input in [
            "S---7\n|...|\n|.F-J\n|.|..\nL-J..",
            "F---7\n|...|\n|.F-J\n|.|..\nL-S..",
        ] {
            assert_eq!(part1(input).unwrap(), "8");
            assert_eq!(part2(input).unwrap(), "5");
        }
    }
}
