use svutils::SolveError;

const ARROWS: [u8; 4] = *b"^>v<";

pub struct Grid {
    data: Box<[u8]>,
    offset: usize,
    start: usize,
    end: usize,
}
impl Grid {
    fn from_str(s: &str) -> Result<Self, SolveError> {
        let rows: Vec<_> = s.lines().collect();
        let width = rows.first().map_or(0, |row| row.len());
        if width == 0 {
            return Err(SolveError::InvalidInput(
                "expected a nonempty hiking map".into(),
            ));
        }
        if rows
            .iter()
            .any(|row| row.len() != width || !row.bytes().all(|tile| b"#.^>v<".contains(&tile)))
        {
            return Err(SolveError::InvalidInput(
                "expected rectangular rows of hiking tiles".into(),
            ));
        }
        let opening = |row: &str| -> Result<usize, SolveError> {
            let mut positions = row.bytes().enumerate().filter(|(_, tile)| *tile != b'#');
            match (positions.next(), positions.next()) {
                (Some((position, _)), None) => Ok(position),
                _ => Err(SolveError::InvalidInput(
                    "expected exactly one opening in each boundary row".into(),
                )),
            }
        };
        let start = opening(rows[0])?;
        let end = (rows.len() - 1) * width + opening(rows[rows.len() - 1])?;
        Ok(Self {
            data: rows.iter().flat_map(|row| row.bytes()).collect::<Box<_>>(),
            offset: width,
            start,
            end,
        })
    }

    const fn next_pos(&self, p: usize, dir: u8) -> Option<usize> {
        Some(match dir {
            0 if p >= self.offset => p - self.offset,
            1 if (p + 1) % self.offset != 0 => p + 1,
            2 if p < self.data.len() - self.offset => p + self.offset,
            3 if p % self.offset != 0 => p - 1,
            _ => return None,
        })
    }
    fn can_leave(&self, position: usize, direction: u8, slopes: bool) -> bool {
        !slopes
            || ARROWS
                .iter()
                .position(|&arrow| self.data[position] == arrow)
                .is_none_or(|arrow| arrow == usize::from(direction))
    }

    fn intersections_and_distances(&self, slopes: bool) -> Vec<Vec<(usize, usize)>> {
        let mut vertices = vec![self.start, self.end];
        for (position, &tile) in self.data.iter().enumerate() {
            if tile == b'#' {
                continue;
            }
            let degree = (0..4)
                .filter_map(|direction| self.next_pos(position, direction))
                .filter(|&neighbor| self.data[neighbor] != b'#')
                .count();
            if degree > 2 {
                vertices.push(position);
            }
        }
        vertices.sort_unstable();
        vertices.dedup();
        let mut vertex_at = vec![None; self.data.len()];
        for (id, &position) in vertices.iter().enumerate() {
            vertex_at[position] = Some(id);
        }
        let mut edges: Vec<Vec<(usize, usize)>> = vec![vec![]; vertices.len()];
        for (source, &position) in vertices.iter().enumerate() {
            for direction in 0..4 {
                if !self.can_leave(position, direction, slopes) {
                    continue;
                }
                let Some(mut current) = self
                    .next_pos(position, direction)
                    .filter(|&neighbor| self.data[neighbor] != b'#')
                else {
                    continue;
                };
                let mut previous = position;
                let mut distance = 1;
                loop {
                    if let Some(target) = vertex_at[current] {
                        if target != source {
                            if let Some((_, length)) =
                                edges[source].iter_mut().find(|(id, _)| *id == target)
                            {
                                *length = (*length).max(distance);
                            } else {
                                edges[source].push((target, distance));
                            }
                        }
                        break;
                    }
                    let next = (0..4)
                        .filter(|&direction| self.can_leave(current, direction, slopes))
                        .filter_map(|direction| self.next_pos(current, direction))
                        .find(|&neighbor| neighbor != previous && self.data[neighbor] != b'#');
                    let Some(next) = next else {
                        break;
                    };
                    previous = current;
                    current = next;
                    distance += 1;
                }
            }
        }
        edges
    }
}

fn longest_path(
    position: usize,
    edges: &[Vec<(usize, usize)>],
    visited: &mut [bool],
    end: usize,
) -> Option<usize> {
    if position == end {
        return Some(0);
    }
    visited[position] = true;
    let mut best = None;
    for &(neighbor, distance) in &edges[position] {
        if !visited[neighbor] {
            if let Some(remaining) = longest_path(neighbor, edges, visited, end) {
                let length = distance + remaining;
                best = Some(best.map_or(length, |current: usize| current.max(length)));
            }
        }
    }
    visited[position] = false;
    best
}

fn longest_hike(edges: &[Vec<(usize, usize)>], start: usize, end: usize) -> Option<usize> {
    if start == end {
        return Some(0);
    }
    let mut visited = vec![false; edges.len()];
    let mut incoming = edges.iter().enumerate().flat_map(|(source, outgoing)| {
        outgoing
            .iter()
            .filter_map(move |&(target, distance)| (target == end).then_some((source, distance)))
    });
    match (incoming.next(), incoming.next()) {
        (Some((predecessor, distance)), None) => {
            visited[end] = true;
            longest_path(start, edges, &mut visited, predecessor).map(|length| length + distance)
        }
        _ => longest_path(start, edges, &mut visited, end),
    }
}

fn solve(input: &str, slopes: bool) -> Result<String, SolveError> {
    let grid = Grid::from_str(input)?;
    let edges = grid.intersections_and_distances(slopes);
    longest_hike(&edges, 0, edges.len() - 1)
        .map(|distance| distance.to_string())
        .ok_or_else(|| SolveError::InvalidInput("no route from entrance to exit".into()))
}

pub fn part1(input: &str) -> Result<String, SolveError> {
    solve(input, true)
}

pub fn part2(input: &str) -> Result<String, SolveError> {
    solve(input, false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use svutils::scaffold_test_result;

    fn tile_reference(input: &str, slopes: bool) -> Option<usize> {
        fn walk(
            rows: &[Vec<u8>],
            p: (usize, usize),
            end: (usize, usize),
            slopes: bool,
            seen: &mut [Vec<bool>],
        ) -> Option<usize> {
            if p == end {
                return Some(0);
            }
            seen[p.0][p.1] = true;
            let arrow = match rows[p.0][p.1] {
                b'^' => Some(0),
                b'>' => Some(1),
                b'v' => Some(2),
                b'<' => Some(3),
                _ => None,
            };
            let mut best = None;
            for (direction, (dr, dc)) in [(-1isize, 0isize), (0, 1), (1, 0), (0, -1)]
                .into_iter()
                .enumerate()
            {
                if slopes && arrow.is_some_and(|arrow| arrow != direction) {
                    continue;
                }
                let (Some(row), Some(column)) =
                    (p.0.checked_add_signed(dr), p.1.checked_add_signed(dc))
                else {
                    continue;
                };
                if row >= rows.len()
                    || column >= rows[0].len()
                    || rows[row][column] == b'#'
                    || seen[row][column]
                {
                    continue;
                }
                if let Some(distance) = walk(rows, (row, column), end, slopes, seen) {
                    best = Some(best.unwrap_or(0).max(distance + 1));
                }
            }
            seen[p.0][p.1] = false;
            best
        }
        let rows: Vec<_> = input.lines().map(|line| line.as_bytes().to_vec()).collect();
        let start = rows[0].iter().position(|&tile| tile != b'#').unwrap();
        let end = (
            rows.len() - 1,
            rows.last()
                .unwrap()
                .iter()
                .position(|&tile| tile != b'#')
                .unwrap(),
        );
        let mut seen = vec![vec![false; rows[0].len()]; rows.len()];
        walk(&rows, (0, start), end, slopes, &mut seen)
    }

    #[test]
    fn forced_exit_checks_incoming_corridors() {
        let directed = vec![
            vec![(1, 2), (2, 3)],
            vec![(2, 11), (3, 5)],
            vec![(1, 7)],
            vec![(0, 1), (2, 1)],
        ];
        let multiple = vec![vec![(1, 2), (2, 3)], vec![(3, 5)], vec![(3, 7)], vec![]];
        let disconnected = vec![vec![(1, 2)], vec![(0, 2)], vec![(3, 5)], vec![]];
        for (edges, expected) in [
            (&directed, Some(15)),
            (&multiple, Some(10)),
            (&disconnected, None),
        ] {
            assert_eq!(longest_hike(edges, 0, 3), expected);
            assert_eq!(
                longest_hike(edges, 0, 3),
                longest_path(0, edges, &mut vec![false; edges.len()], 3)
            );
        }
        assert_eq!(longest_hike(&[vec![]], 0, 0), Some(0));
        assert_eq!(longest_hike(&[vec![], vec![]], 0, 1), None);
        assert_eq!(longest_hike(&[vec![(1, 5)], vec![]], 0, 1), Some(5));
    }

    #[test]
    fn discovers_endpoints_and_validates_map() {
        let shifted = "###.#\n#...#\n#.###";
        assert_eq!(part1(shifted).unwrap(), "4");
        assert_eq!(part2(shifted).unwrap(), "4");
        assert_eq!(part2(&shifted.replace('\n', "\r\n")).unwrap(), "4");
        assert_eq!(part1(".").unwrap(), "0");
        assert_eq!(part2(".\n.\n.").unwrap(), "2");
        let arrow = Grid::from_str("v\n.").unwrap();
        assert_eq!((arrow.start, arrow.end), (0, 1));
        for invalid in [
            "", "\n", "##\n##", "..\n#.", ".#\n..", ".#\n.", ".#\n\n.#", ".#\n?海", ".#\n #",
        ] {
            assert!(Grid::from_str(invalid).is_err(), "{invalid:?}");
            assert!(part1(invalid).is_err());
            assert!(part2(invalid).is_err());
        }
    }

    #[test]
    fn distinguishes_no_route_from_zero_steps() {
        for part in [part1, part2] {
            assert_eq!(part(".").unwrap(), "0");
            assert!(part(".#\n##\n#.")
                .unwrap_err()
                .to_string()
                .contains("no route"));
        }
        assert!(part1("^\n.").is_err());
        assert_eq!(part2("^\n.").unwrap(), "1");
    }

    #[test]
    fn longer_parallel_corridors_are_retained_per_direction() {
        let plain = "#.####\n#....#\n#..#.#\n####.#\n####.#";
        assert_eq!(part1(plain).unwrap(), "9");
        assert_eq!(part2(plain).unwrap(), "9");
        let slopes = "#.####\n#....#\n#>^#.#\n####.#\n####.#";
        let grid = Grid::from_str(slopes).unwrap();
        let directed = grid.intersections_and_distances(true);
        assert!(directed[1].contains(&(2, 3)));
        assert!(directed[2].contains(&(1, 1)));
        let undirected = grid.intersections_and_distances(false);
        assert!(undirected[2].contains(&(1, 3)));
        for input in [plain, slopes] {
            assert_eq!(
                part1(input).unwrap(),
                tile_reference(input, true).unwrap().to_string()
            );
            assert_eq!(
                part2(input).unwrap(),
                tile_reference(input, false).unwrap().to_string()
            );
        }
    }

    #[test]
    fn compressed_paths_match_exhaustive_small_tile_searches() {
        for mask in 0..1usize << 12 {
            for arrows in [false, true] {
                let mut cells = vec![b'#'; 30];
                cells[1] = b'.';
                cells[28] = b'.';
                for tile in 0..12 {
                    if mask & (1 << tile) != 0 {
                        cells[(tile / 4 + 1) * 6 + tile % 4 + 1] = if arrows {
                            b".^>v<"[(tile + mask) % 5]
                        } else {
                            b'.'
                        };
                    }
                }
                let input = cells
                    .chunks(6)
                    .map(|row| std::str::from_utf8(row).unwrap())
                    .collect::<Vec<_>>()
                    .join("\n");
                for slopes in [false, true] {
                    let expected = tile_reference(&input, slopes);
                    let actual = if slopes { part1(&input) } else { part2(&input) };
                    match expected {
                        Some(distance) => assert_eq!(
                            actual.unwrap(),
                            distance.to_string(),
                            "slopes={slopes}\n{input}"
                        ),
                        None => assert!(
                            actual.unwrap_err().to_string().contains("no route"),
                            "slopes={slopes}\n{input}"
                        ),
                    }
                }
            }
        }
    }

    const YEAR: i16 = 2023;
    const DAY: i16 = 23;

    #[test]
    fn test_part1_input() {
        scaffold_test_result(YEAR, DAY, "input.txt", "input-spec.1.txt", part1);
    }

    #[test]
    fn test_part2_input() {
        scaffold_test_result(YEAR, DAY, "input.txt", "input-spec.2.txt", part2);
    }
}
