use svutils::SolveError;

#[derive(Clone, Copy, Debug)]
struct Point {
    x: i64,
    y: i64,
}

impl Point {
    const fn new(x: i64, y: i64) -> Self {
        Self { x, y }
    }
}

struct GalaxyMap {
    galaxies: Vec<Point>,
    empty_rows: Vec<i64>,
    empty_columns: Vec<i64>,
}

fn empty_prefix_counts(empty: Vec<bool>) -> Vec<i64> {
    let mut count = 0;
    empty
        .into_iter()
        .map(|is_empty| {
            count += i64::from(is_empty);
            count
        })
        .collect()
}

fn parse(input: &str) -> Result<GalaxyMap, SolveError> {
    if input.trim().is_empty() {
        return Err(SolveError::EmptyInput);
    }
    let rows: Vec<_> = input.lines().map(str::as_bytes).collect();
    let width = rows[0].len();
    if width == 0 {
        return Err(SolveError::InvalidInput("grid row is empty".into()).at_line(1));
    }
    // Coordinates and prefix counts fit i64 once the dimensions do.
    i64::try_from(width).map_err(|_| SolveError::ArithmeticOverflow)?;
    i64::try_from(rows.len()).map_err(|_| SolveError::ArithmeticOverflow)?;
    let mut galaxies = Vec::new();
    let mut empty_rows = vec![true; rows.len()];
    let mut empty_columns = vec![true; width];
    for (y, row) in rows.into_iter().enumerate() {
        let invalid = |message: &str| SolveError::InvalidInput(message.into()).at_line(y + 1);
        if row.len() != width {
            return Err(invalid("grid rows must have equal width"));
        }
        for (x, tile) in row.iter().enumerate() {
            match tile {
                b'#' => {
                    galaxies.push(Point::new(x as i64, y as i64));
                    empty_rows[y] = false;
                    empty_columns[x] = false;
                }
                b'.' => {}
                _ => return Err(invalid("grid accepts only '.' and '#'")),
            }
        }
    }
    if galaxies.len() < 2 {
        return Err(SolveError::InvalidInput(
            "grid must contain at least two galaxies".into(),
        ));
    }
    Ok(GalaxyMap {
        galaxies,
        empty_rows: empty_prefix_counts(empty_rows),
        empty_columns: empty_prefix_counts(empty_columns),
    })
}

fn expand(input: &GalaxyMap, factor: i64) -> Result<Vec<Point>, SolveError> {
    if factor < 1 {
        return Err(SolveError::InvalidInput(
            "expansion factor must be positive".into(),
        ));
    }
    let extra_width = factor - 1;
    input
        .galaxies
        .iter()
        .map(|point| {
            let x = extra_width
                .checked_mul(input.empty_columns[point.x as usize])
                .and_then(|offset| point.x.checked_add(offset))
                .ok_or(SolveError::ArithmeticOverflow)?;
            let y = extra_width
                .checked_mul(input.empty_rows[point.y as usize])
                .and_then(|offset| point.y.checked_add(offset))
                .ok_or(SolveError::ArithmeticOverflow)?;
            Ok(Point::new(x, y))
        })
        .collect()
}

fn axis_distance(mut coordinates: Vec<i64>) -> Result<i128, SolveError> {
    coordinates.sort_unstable();
    let mut prefix_sum = 0_i128;
    let mut total = 0_i128;
    for (index, coordinate) in coordinates.into_iter().enumerate() {
        let coordinate = i128::from(coordinate);
        let count = i128::try_from(index).map_err(|_| SolveError::ArithmeticOverflow)?;
        // Sorted earlier coordinates are <= this one, so absolute differences
        // sum to count * coordinate - sum(earlier coordinates).
        let contribution = count
            .checked_mul(coordinate)
            .and_then(|product| product.checked_sub(prefix_sum))
            .ok_or(SolveError::ArithmeticOverflow)?;
        total = total
            .checked_add(contribution)
            .ok_or(SolveError::ArithmeticOverflow)?;
        prefix_sum = prefix_sum
            .checked_add(coordinate)
            .ok_or(SolveError::ArithmeticOverflow)?;
    }
    Ok(total)
}

fn total_distance(points: &[Point]) -> Result<i64, SolveError> {
    let horizontal = axis_distance(points.iter().map(|point| point.x).collect())?;
    let vertical = axis_distance(points.iter().map(|point| point.y).collect())?;
    let total = horizontal
        .checked_add(vertical)
        .ok_or(SolveError::ArithmeticOverflow)?;
    i64::try_from(total).map_err(|_| SolveError::ArithmeticOverflow)
}

fn solve(input: &str, factor: i64) -> Result<String, SolveError> {
    let galaxies = parse(input)?;
    let points = expand(&galaxies, factor)?;
    Ok(total_distance(&points)?.to_string())
}

pub fn part1(input: &str) -> Result<String, SolveError> {
    solve(input, 2)
}

pub fn part2(input: &str) -> Result<String, SolveError> {
    solve(input, 1_000_000)
}

#[cfg(test)]
mod tests {
    use super::*;
    use svutils::scaffold_test_result;

    const YEAR: i16 = 2023;
    const DAY: i16 = 11;

    #[test]
    fn test_part1_input() {
        scaffold_test_result(YEAR, DAY, "input.txt", "input-spec.1.txt", part1);
    }

    #[test]
    fn test_part2_input() {
        scaffold_test_result(YEAR, DAY, "input.txt", "input-spec.2.txt", part2);
    }

    #[test]
    fn tall_grid_counts_empty_rows_using_its_height() {
        let input = "#.\n..\n.#";
        assert_eq!(solve(input, 1).unwrap(), "3");
        assert_eq!(solve(input, 2).unwrap(), "4");
        assert_eq!(solve(input, 10).unwrap(), "12");
    }

    #[test]
    fn wide_grid_counts_empty_columns_using_its_width() {
        let input = "#..\n..#";
        assert_eq!(solve(input, 1).unwrap(), "3");
        assert_eq!(solve(input, 2).unwrap(), "4");
        assert_eq!(solve(input, 10).unwrap(), "12");
    }

    #[test]
    fn malformed_grids_are_rejected() {
        for input in ["", " \t\r\n", "\n"] {
            assert!(matches!(part1(input), Err(SolveError::EmptyInput)));
            assert!(matches!(part2(input), Err(SolveError::EmptyInput)));
        }
        for input in [
            "\n##",
            "##\n.",
            "##\n\n..",
            "##\n..\n\n",
            "#x#",
            "# #",
            "#é#",
            ".",
            "#",
            "...\n.#.",
        ] {
            assert!(part1(input).is_err(), "{input:?}");
            assert!(part2(input).is_err(), "{input:?}");
        }
    }

    #[test]
    fn normal_trailing_newlines_and_crlf_are_accepted() {
        for input in ["#.\n..\n.#\n", "#.\r\n..\r\n.#\r\n"] {
            assert_eq!(part1(input).unwrap(), "4");
            assert_eq!(part2(input).unwrap(), "1000002");
        }
    }

    #[test]
    fn expansion_factor_must_be_positive() {
        for factor in [i64::MIN, -1, 0] {
            assert!(matches!(
                solve("#.#", factor),
                Err(SolveError::InvalidInput(_))
            ));
        }
    }

    #[test]
    fn large_expansion_is_exact_and_overflow_is_reported() {
        assert_eq!(solve("#.#", i64::MAX - 1).unwrap(), i64::MAX.to_string());
        for (input, factor) in [
            ("#..#", i64::MAX),              // expansion multiplication
            ("#.#", i64::MAX),               // expanded coordinate addition
            ("#..\n...\n..#", i64::MAX / 2), // individual Manhattan distance
            ("#.#.#", i64::MAX / 4),         // total of representable pair distances
        ] {
            assert!(matches!(
                solve(input, factor),
                Err(SolveError::ArithmeticOverflow)
            ));
        }
    }

    #[test]
    fn sorted_axis_sum_matches_direct_pair_distances() {
        // Exhaust all subsets of a small grid, including shared coordinates.
        for mask in 0_u32..(1 << 9) {
            let points: Vec<_> = (0..9)
                .filter(|index| mask & (1 << index) != 0)
                .map(|index| Point::new(index % 3, index / 3))
                .collect();
            let expected: i64 = points
                .iter()
                .enumerate()
                .map(|(index, point)| {
                    points[..index]
                        .iter()
                        .map(|other| (point.x - other.x).abs() + (point.y - other.y).abs())
                        .sum::<i64>()
                })
                .sum();
            assert_eq!(total_distance(&points).unwrap(), expected);
            let reversed: Vec<_> = points.into_iter().rev().collect();
            assert_eq!(total_distance(&reversed).unwrap(), expected);
        }
    }
}
