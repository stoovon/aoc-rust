use std::collections::{HashSet, VecDeque};
use svutils::SolveError;

type Point = (isize, isize);

#[derive(Clone, Copy)]
enum Boundary {
    Finite,
    Repeating,
}

struct Map {
    width: usize,
    height: usize,
    cells: Vec<u8>,
    start: Point,
}

impl Map {
    fn parse(input: &str) -> Result<Self, SolveError> {
        let mut cells = Vec::new();
        let mut width = None;
        let mut height = 0;
        let mut start = None;
        for (y, line) in input.lines().enumerate() {
            if line.is_empty() || width.is_some_and(|width| width != line.len()) {
                return Err(SolveError::InvalidInput(
                    "expected a nonempty rectangular garden".into(),
                )
                .at_line(y + 1));
            }
            width = Some(line.len());
            let y_pos = isize::try_from(y).map_err(|_| SolveError::ArithmeticOverflow)?;
            for (x, cell) in line.bytes().enumerate() {
                match cell {
                    b'.' | b'#' => {}
                    b'S' => {
                        let x_pos =
                            isize::try_from(x).map_err(|_| SolveError::ArithmeticOverflow)?;
                        if start.replace((x_pos, y_pos)).is_some() {
                            return Err(SolveError::InvalidInput(
                                "expected exactly one start".into(),
                            )
                            .at_line(y + 1));
                        }
                    }
                    _ => {
                        return Err(SolveError::InvalidInput(
                            "expected only '.', '#', or 'S'".into(),
                        )
                        .at_line(y + 1))
                    }
                }
                cells.push(cell);
            }
            height = y + 1;
        }
        let width = width.ok_or(SolveError::EmptyInput)?;
        isize::try_from(width).map_err(|_| SolveError::ArithmeticOverflow)?;
        isize::try_from(height).map_err(|_| SolveError::ArithmeticOverflow)?;
        let start =
            start.ok_or_else(|| SolveError::InvalidInput("expected exactly one start".into()))?;
        Ok(Self {
            width,
            height,
            cells,
            start,
        })
    }

    fn open(&self, (x, y): Point, boundary: Boundary) -> bool {
        let width = self.width as isize;
        let height = self.height as isize;
        let (x, y) = match boundary {
            Boundary::Finite if x < 0 || y < 0 || x >= width || y >= height => return false,
            Boundary::Finite => (x, y),
            Boundary::Repeating => (x.rem_euclid(width), y.rem_euclid(height)),
        };
        self.cells[y as usize * self.width + x as usize] != b'#'
    }

    fn get_moves(&self, (x, y): Point, boundary: Boundary) -> impl Iterator<Item = Point> + '_ {
        [(x - 1, y), (x + 1, y), (x, y - 1), (x, y + 1)]
            .into_iter()
            .filter(move |&point| self.open(point, boundary))
    }

    fn reachable(&self, steps: usize, boundary: Boundary) -> Result<i64, SolveError> {
        let reach = isize::try_from(steps).map_err(|_| SolveError::ArithmeticOverflow)?;
        for coordinate in [self.start.0, self.start.1] {
            coordinate
                .checked_add(reach)
                .and_then(|n| n.checked_add(1))
                .ok_or(SolveError::ArithmeticOverflow)?;
            coordinate
                .checked_sub(reach)
                .and_then(|n| n.checked_sub(1))
                .ok_or(SolveError::ArithmeticOverflow)?;
        }
        if self.get_moves(self.start, boundary).next().is_none() {
            return Ok(i64::from(steps == 0));
        }
        let mut visited = HashSet::from([self.start]);
        let mut distance_layer = vec![self.start];
        let mut parity_totals = [1i64, 0];
        // In an undirected grid, an edge and its reverse spend any spare pair of steps.
        for distance in 1..=steps {
            let mut next_layer = Vec::new();
            for point in distance_layer {
                for next in self.get_moves(point, boundary) {
                    if visited.insert(next) {
                        next_layer.push(next);
                    }
                }
            }
            parity_totals[distance % 2] = parity_totals[distance % 2]
                .checked_add(
                    i64::try_from(next_layer.len()).map_err(|_| SolveError::ArithmeticOverflow)?,
                )
                .ok_or(SolveError::ArithmeticOverflow)?;
            if next_layer.is_empty() {
                break;
            }
            distance_layer = next_layer;
        }
        Ok(parity_totals[steps % 2])
    }

    fn certify(&self) -> Result<CertifiedGarden<'_>, SolveError> {
        if self.width != self.height {
            return Err(SolveError::InvalidInput(
                "large repeating counts require a square garden".into(),
            ));
        }
        for radius in [1, 2] {
            let model = DistanceModel::build(self, radius)?;
            if model.validates(self) {
                return Ok(CertifiedGarden { map: self, model });
            }
        }
        Err(SolveError::InvalidInput(
            "repeating garden has no certified distance model within the fixed budget".into(),
        ))
    }
}

struct DistanceModel {
    radius: isize,
    side: usize,
    distances: Vec<usize>,
}

impl DistanceModel {
    fn build(map: &Map, radius: isize) -> Result<Self, SolveError> {
        let copies = (2 * radius + 1) as usize;
        let side = map
            .width
            .checked_mul(copies)
            .ok_or(SolveError::ArithmeticOverflow)?;
        let area = side
            .checked_mul(side)
            .ok_or(SolveError::ArithmeticOverflow)?;
        // The surrounding validation ring and its neighbours must also fit Point.
        map.width
            .checked_mul(copies + 2)
            .and_then(|n| n.checked_add(1))
            .filter(|&n| isize::try_from(n).is_ok())
            .ok_or(SolveError::ArithmeticOverflow)?;
        let mut distances = Vec::new();
        distances
            .try_reserve_exact(area)
            .map_err(|_| SolveError::InvalidInput("garden distance patch is too large".into()))?;
        distances.resize(area, usize::MAX);
        let shift = radius as usize * map.width;
        let start = (map.start.1 as usize + shift) * side + map.start.0 as usize + shift;
        distances[start] = 0;
        let mut queue = VecDeque::from([start]);
        while let Some(index) = queue.pop_front() {
            let x = index % side;
            let y = index / side;
            let neighbours = [
                x.checked_sub(1).map(|nx| y * side + nx),
                (x + 1 < side).then_some(index + 1),
                y.checked_sub(1).map(|ny| ny * side + x),
                (y + 1 < side).then_some(index + side),
            ];
            for next in neighbours.into_iter().flatten() {
                let local = (next / side % map.height) * map.width + next % side % map.width;
                if distances[next] == usize::MAX && map.cells[local] != b'#' {
                    distances[next] = distances[index]
                        .checked_add(1)
                        .ok_or(SolveError::ArithmeticOverflow)?;
                    queue.push_back(next);
                }
            }
        }
        Ok(Self {
            radius,
            side,
            distances,
        })
    }

    fn distance(&self, map: &Map, (x, y): Point) -> Option<i128> {
        let width = map.width as isize;
        let (tx, ty) = (x.div_euclid(width), y.div_euclid(width));
        let core_x = tx.clamp(-self.radius, self.radius);
        let core_y = ty.clamp(-self.radius, self.radius);
        let col = ((core_x + self.radius) * width + x.rem_euclid(width)) as usize;
        let row = ((core_y + self.radius) * width + y.rem_euclid(width)) as usize;
        let base = self.distances[row * self.side + col];
        if base == usize::MAX {
            return None;
        }
        let translations =
            (tx as i128 - core_x as i128).abs() + (ty as i128 - core_y as i128).abs();
        Some(base as i128 + translations * width as i128)
    }

    fn validates(&self, map: &Map) -> bool {
        let width = map.width as isize;
        let low = -(self.radius + 1) * width;
        let high = (self.radius + 2) * width;
        // Outside the core, translating outward adds width to both endpoints of
        // an edge. This ring represents every possible neighbour-distance difference.
        // Edge inequalities bound every path from below; a decreasing neighbour
        // at each positive distance supplies a path to the unique zero at S.
        for y in low..high {
            for x in low..high {
                let point = (x, y);
                if !map.open(point, Boundary::Repeating) {
                    if self.distance(map, point).is_some() {
                        return false;
                    }
                    continue;
                }
                let distance = self.distance(map, point);
                if distance == Some(0) && point != map.start
                    || point == map.start && distance != Some(0)
                {
                    return false;
                }
                let mut descending = point == map.start;
                for next in map.get_moves(point, Boundary::Repeating) {
                    match (distance, self.distance(map, next)) {
                        (None, None) => {}
                        (Some(d), Some(n)) if (d - n).abs() <= 1 => descending |= n + 1 == d,
                        _ => return false,
                    }
                }
                if distance.is_some() && !descending {
                    return false;
                }
            }
        }
        true
    }
}

struct CertifiedGarden<'a> {
    map: &'a Map,
    model: DistanceModel,
}

fn translated_count(remaining: i128, width: i128, axes: u8) -> Result<i128, SolveError> {
    if remaining < 0 {
        return Ok(0);
    }
    let copies = remaining / width;
    let parity = remaining % 2;
    match axes {
        0 => Ok(i128::from(parity == 0)),
        1 if width % 2 == 0 => Ok(if parity == 0 { copies } else { 0 }),
        1 => Ok(if parity == 0 {
            copies / 2
        } else {
            copies / 2 + copies % 2
        }),
        // Two positive translations with total s have s-1 ordered decompositions.
        2 if width % 2 == 0 => {
            if parity != 0 || copies < 2 {
                return Ok(0);
            }
            copies
                .checked_mul(copies - 1)
                .map(|n| n / 2)
                .ok_or(SolveError::ArithmeticOverflow)
        }
        2 if parity == 0 => (copies / 2)
            .checked_mul(copies / 2)
            .ok_or(SolveError::ArithmeticOverflow),
        2 => {
            let pairs = (copies - 1).max(0) / 2;
            pairs
                .checked_mul(pairs + 1)
                .ok_or(SolveError::ArithmeticOverflow)
        }
        _ => unreachable!("only core, rays, and quadrants are counted"),
    }
}

impl CertifiedGarden<'_> {
    fn count(&self, steps: i64) -> Result<i64, SolveError> {
        if steps < 0 {
            return Err(SolveError::InvalidInput(
                "step count must be nonnegative".into(),
            ));
        }
        if self
            .map
            .get_moves(self.map.start, Boundary::Repeating)
            .next()
            .is_none()
        {
            return Ok(i64::from(steps == 0));
        }
        let width = self.map.width as isize;
        let radius = self.model.radius;
        let mut total = 0i128;
        for (index, &distance) in self.model.distances.iter().enumerate() {
            if distance == usize::MAX {
                continue;
            }
            let tile_x = (index % self.model.side / self.map.width) as isize - radius;
            let tile_y = (index / self.model.side / self.map.width) as isize - radius;
            let remaining = steps as i128 - distance as i128;
            let mut add = |axes| -> Result<(), SolveError> {
                total = total
                    .checked_add(translated_count(remaining, width as i128, axes)?)
                    .ok_or(SolveError::ArithmeticOverflow)?;
                Ok(())
            };
            add(0)?;
            if tile_x.abs() == radius {
                add(1)?;
            }
            if tile_y.abs() == radius {
                add(1)?;
            }
            if tile_x.abs() == radius && tile_y.abs() == radius {
                add(2)?;
            }
        }
        i64::try_from(total).map_err(|_| SolveError::ArithmeticOverflow)
    }
}

pub fn part1(input: &str) -> Result<String, SolveError> {
    Map::parse(input)?
        .reachable(64, Boundary::Finite)
        .map(|n| n.to_string())
}

pub fn part2(input: &str) -> Result<String, SolveError> {
    Map::parse(input)?
        .certify()?
        .count(26_501_365)
        .map(|n| n.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use svutils::scaffold_test_result;

    fn direct_counts(map: &Map, boundary: Boundary, steps: usize) -> Vec<i64> {
        let mut frontier = HashSet::from([map.start]);
        let mut counts = vec![1];
        for _ in 0..steps {
            // Deliberately retain exact-step frontiers as an independent small oracle.
            frontier = frontier
                .into_iter()
                .flat_map(|(x, y)| [(x - 1, y), (x + 1, y), (x, y - 1), (x, y + 1)])
                .filter(|&point| map.open(point, boundary))
                .collect();
            counts.push(frontier.len() as i64);
        }
        counts
    }

    fn small_map(width: usize, height: usize, start: usize, mut rocks: usize) -> Map {
        let mut text = String::new();
        for index in 0..width * height {
            text.push(if index == start {
                'S'
            } else {
                let rock = rocks & 1 != 0;
                rocks >>= 1;
                if rock {
                    '#'
                } else {
                    '.'
                }
            });
            if index % width == width - 1 {
                text.push('\n');
            }
        }
        Map::parse(&text).unwrap()
    }

    #[test]
    fn parser_validates_shape_symbols_and_start() {
        assert!(matches!(Map::parse(""), Err(SolveError::EmptyInput)));
        for text in [".\n", "SS", "S?", "Sé", "S\n\n", "S.\n."] {
            assert!(Map::parse(text).is_err(), "{text:?}");
        }
        let error = Map::parse("S.\n.x").err().unwrap();
        assert!(error.to_string().starts_with("line 2:"));
        let map = Map::parse(".S\r\n#.\r\n").unwrap();
        assert_eq!((map.width, map.height, map.start), (2, 2, (1, 0)));
    }

    #[test]
    fn boundaries_wrap_each_axis_independently() {
        let map = Map::parse("S.#\n.#.").unwrap();
        assert!(!map.open((-1, 0), Boundary::Finite));
        assert!(!map.open((-1, 0), Boundary::Repeating));
        assert!(map.open((-1, -1), Boundary::Repeating));
        assert!(!map.open((1, -1), Boundary::Repeating));
        assert_eq!(map.reachable(1, Boundary::Finite).unwrap(), 2);
        assert_eq!(map.reachable(1, Boundary::Repeating).unwrap(), 3);
    }

    #[test]
    fn zero_steps_isolation_and_backtracking() {
        let isolated = Map::parse("###\n#S#\n###").unwrap();
        for boundary in [Boundary::Finite, Boundary::Repeating] {
            assert_eq!(isolated.reachable(0, boundary).unwrap(), 1);
            assert_eq!(isolated.reachable(2, boundary).unwrap(), 0);
        }
        assert_eq!(isolated.certify().unwrap().count(2).unwrap(), 0);
        let corridor = Map::parse("S..").unwrap();
        assert_eq!(corridor.reachable(2, Boundary::Finite).unwrap(), 2);
        assert_eq!(corridor.reachable(3, Boundary::Finite).unwrap(), 1);
        assert_eq!(
            Map::parse("S")
                .unwrap()
                .reachable(64, Boundary::Finite)
                .unwrap(),
            0
        );
    }

    #[test]
    fn distance_parity_matches_direct_steps_on_small_rectangles() {
        let mut cases = 0;
        for width in 1..=3 {
            for height in 1..=3 {
                for rocks in 0..1 << (width * height - 1) {
                    let map = small_map(width, height, 0, rocks);
                    for boundary in [Boundary::Finite, Boundary::Repeating] {
                        let expected = direct_counts(&map, boundary, 8);
                        for (steps, &count) in expected.iter().enumerate() {
                            assert_eq!(map.reachable(steps, boundary).unwrap(), count);
                        }
                        cases += 1;
                    }
                }
            }
        }
        assert_eq!(cases, 682);
    }

    #[test]
    fn certified_counts_match_direct_steps_for_every_small_square_and_start() {
        let mut cases = 0;
        let mut accepted = 0;
        for width in 1..=3 {
            for start in 0..width * width {
                for rocks in 0..1 << (width * width - 1) {
                    let map = small_map(width, width, start, rocks);
                    if let Ok(garden) = map.certify() {
                        for (steps, count) in direct_counts(&map, Boundary::Repeating, 20)
                            .into_iter()
                            .enumerate()
                        {
                            assert_eq!(
                                garden.count(steps as i64).unwrap(),
                                count,
                                "width={width}, start={start}, rocks={rocks}, steps={steps}"
                            );
                        }
                        accepted += 1;
                    }
                    cases += 1;
                }
            }
        }
        assert_eq!(cases, 2337);
        assert!(accepted > 100);
    }

    #[test]
    fn certificate_rejects_corrupted_distances_and_unreachable_regions() {
        let map = Map::parse("S.\n..").unwrap();
        let mut model = DistanceModel::build(&map, 1).unwrap();
        assert!(model.validates(&map));
        let index = model.distances.iter().position(|&n| n == 1).unwrap();
        model.distances[index] = 0;
        assert!(!model.validates(&map));
        model.distances[index] = usize::MAX;
        assert!(!model.validates(&map));
        model.distances[index] = 4;
        assert!(!model.validates(&map));
        let start = model.distances.iter().position(|&n| n == 0).unwrap();
        model.distances[start] = 2;
        assert!(!model.validates(&map));
    }

    #[test]
    fn translation_formulas_match_enumeration_for_both_parities() {
        for width in 1..=6i128 {
            for remaining in -1..=60 {
                for axes in 0..=2 {
                    let expected = match axes {
                        0 => i128::from(remaining >= 0 && remaining % 2 == 0),
                        1 => (1..=60)
                            .filter(|&x| remaining >= x * width && (remaining - x * width) % 2 == 0)
                            .count() as i128,
                        2 => (1..=60)
                            .flat_map(|x| (1..=60).map(move |y| x + y))
                            .filter(|&sum| {
                                remaining >= sum * width && (remaining - sum * width) % 2 == 0
                            })
                            .count() as i128,
                        _ => unreachable!(),
                    };
                    assert_eq!(translated_count(remaining, width, axes).unwrap(), expected);
                }
            }
        }
    }

    #[test]
    fn exact_counting_preserves_large_integers_and_reports_overflow() {
        let map = Map::parse("S").unwrap();
        let garden = map.certify().unwrap();
        let steps = 100_000_000i64;
        assert_eq!(garden.count(steps).unwrap(), (steps + 1) * (steps + 1));
        assert!(matches!(
            garden.count(i64::MAX),
            Err(SolveError::ArithmeticOverflow)
        ));
        assert!(matches!(
            translated_count(i128::MAX - 1, 2, 2),
            Err(SolveError::ArithmeticOverflow)
        ));
        assert!(garden.count(-1).is_err());
        assert!(matches!(
            map.reachable(usize::MAX, Boundary::Finite),
            Err(SolveError::ArithmeticOverflow)
        ));
    }

    #[test]
    fn larger_core_can_certify_but_unsupported_geometry_stays_bounded() {
        let map = Map::parse("#...#\n....#\n#.S.#\n.###.\n...##").unwrap();
        assert!(!DistanceModel::build(&map, 1).unwrap().validates(&map));
        let garden = map.certify().unwrap();
        assert_eq!(garden.model.radius, 2);
        for (steps, expected) in direct_counts(&map, Boundary::Repeating, 60)
            .into_iter()
            .enumerate()
        {
            assert_eq!(garden.count(steps as i64).unwrap(), expected);
        }
        let unsupported = Map::parse(".#..#\n.#...\n..S##\n.#.#.\n.....").unwrap();
        assert!(unsupported.certify().is_err());
    }

    #[test]
    fn nonsquare_large_model_is_rejected() {
        let map = Map::parse("S.").unwrap();
        assert!(map.certify().is_err());
        assert_eq!(map.reachable(5, Boundary::Repeating).unwrap(), 36);
    }

    #[test]
    fn test_part1_input() {
        scaffold_test_result(2023, 21, "input.txt", "input-spec.1.txt", part1);
    }

    #[test]
    fn test_part2_input() {
        scaffold_test_result(2023, 21, "input.txt", "input-spec.2.txt", part2);
    }
}
