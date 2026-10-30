use itertools::Itertools;
use svutils::SolveError;

#[derive(Debug, Clone)]
struct Point3D {
    x: usize,
    y: usize,
    z: usize,
}

#[derive(Debug, Clone)]
struct Brick {
    start: Point3D,
    end: Point3D,
    supports: Vec<usize>,
    supported_by: Vec<usize>,
}

fn parse_point_3d(s: &str) -> Result<Point3D, SolveError> {
    let mut fields = s.split(',');
    let mut coordinates = [0; 3];
    for coordinate in &mut coordinates {
        let field = fields
            .next()
            .ok_or_else(|| SolveError::InvalidInput("expected three coordinates".into()))?;
        if field.is_empty() || !field.bytes().all(|c| c.is_ascii_digit()) {
            return Err(SolveError::InvalidInput(
                "coordinates must be nonnegative decimal integers".into(),
            ));
        }
        *coordinate = field.parse()?;
    }
    if fields.next().is_some() {
        return Err(SolveError::InvalidInput(
            "expected three coordinates".into(),
        ));
    }
    let [x, y, z] = coordinates;
    Ok(Point3D { x, y, z })
}

fn overlaps_xy(a: &Brick, b: &Brick) -> bool {
    a.start.x <= b.end.x && b.start.x <= a.end.x && a.start.y <= b.end.y && b.start.y <= a.end.y
}

fn parse_bricks(input: &str) -> Result<Vec<Brick>, SolveError> {
    if input.trim().is_empty() {
        return Err(SolveError::EmptyInput);
    }
    let bricks = input
        .lines()
        .enumerate()
        .map(|(line_index, line)| {
            let parse = || -> Result<Brick, SolveError> {
                let (start, end) = line.split_once("~").ok_or_else(|| {
                    SolveError::InvalidInput("expected two endpoints separated by ~".into())
                })?;

                let a = parse_point_3d(start)?;
                let b = parse_point_3d(end)?;
                if a.z == 0 || b.z == 0 {
                    return Err(SolveError::InvalidInput(
                        "brick must be above ground (z >= 1)".into(),
                    ));
                }
                if usize::from(a.x != b.x) + usize::from(a.y != b.y) + usize::from(a.z != b.z) > 1 {
                    return Err(SolveError::InvalidInput(
                        "brick must be a straight axis-aligned line or a single cube".into(),
                    ));
                }
                Ok(Brick {
                    start: Point3D {
                        x: a.x.min(b.x),
                        y: a.y.min(b.y),
                        z: a.z.min(b.z),
                    },
                    end: Point3D {
                        x: a.x.max(b.x),
                        y: a.y.max(b.y),
                        z: a.z.max(b.z),
                    },
                    supports: vec![],
                    supported_by: vec![],
                })
            };
            parse().map_err(|error| error.at_line(line_index + 1))
        })
        .collect::<Result<Vec<_>, _>>()?;
    for (index, brick) in bricks.iter().enumerate() {
        for (previous_index, previous) in bricks[..index].iter().enumerate() {
            if brick.start.z <= previous.end.z
                && previous.start.z <= brick.end.z
                && overlaps_xy(brick, previous)
            {
                return Err(SolveError::InvalidInput(format!(
                    "brick overlaps the brick on line {} before falling",
                    previous_index + 1
                ))
                .at_line(index + 1));
            }
        }
    }
    bricks
        .into_iter()
        .sorted_by_key(|brick| brick.start.z)
        .try_fold(Vec::<Brick>::new(), |mut settled, mut brick| {
            let height = brick.end.z - brick.start.z;
            let mut highest_top = 0;
            for (index, support) in settled.iter().enumerate().rev() {
                if support.end.z < highest_top || !overlaps_xy(&brick, support) {
                    continue;
                }
                if support.end.z > highest_top {
                    highest_top = support.end.z;
                    brick.supported_by.clear();
                }
                brick.supported_by.push(index);
            }
            brick.start.z = highest_top
                .checked_add(1)
                .ok_or(SolveError::ArithmeticOverflow)?;
            brick.end.z = brick
                .start
                .z
                .checked_add(height)
                .ok_or(SolveError::ArithmeticOverflow)?;
            let index = settled.len();
            for &support in &brick.supported_by {
                settled[support].supports.push(index);
            }
            settled.push(brick);
            Ok(settled)
        })
}

pub fn part1(input: &str) -> Result<String, SolveError> {
    let bricks = parse_bricks(input)?;

    Ok(bricks
        .iter()
        .filter(|b| {
            b.supports
                .iter()
                .all(|&idx| bricks[idx].supported_by.len() > 1)
        })
        .count()
        .to_string())
}

// Each support edge is removed once, and each child reaches zero only once.
fn cascade_size(
    bricks: &[Brick],
    removed: usize,
    remaining: &mut [usize],
    queue: &mut Vec<usize>,
) -> usize {
    for (count, brick) in remaining.iter_mut().zip(bricks) {
        *count = brick.supported_by.len();
    }
    queue.clear();
    queue.push(removed);
    let mut cursor = 0;
    while cursor < queue.len() {
        let fallen = queue[cursor];
        cursor += 1;
        for &child in &bricks[fallen].supports {
            remaining[child] -= 1;
            if remaining[child] == 0 {
                queue.push(child);
            }
        }
    }
    queue.len() - 1
}

pub fn part2(input: &str) -> Result<String, SolveError> {
    let bricks = parse_bricks(input)?;
    let mut sum = 0usize;
    let mut remaining = vec![0; bricks.len()];
    let mut queue = Vec::with_capacity(bricks.len());
    for removed in 0..bricks.len() {
        if !bricks[removed].supports.is_empty() {
            sum = sum
                .checked_add(cascade_size(&bricks, removed, &mut remaining, &mut queue))
                .ok_or(SolveError::ArithmeticOverflow)?;
        }
    }
    Ok(sum.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use svutils::scaffold_test_result;

    fn cascade_size(bricks: &[Brick], removed: usize) -> usize {
        super::cascade_size(bricks, removed, &mut vec![0; bricks.len()], &mut Vec::new())
    }

    #[test]
    fn parser_boundaries_and_large_coordinates() {
        for input in [
            "",
            " \n\t",
            "0,0,1",
            "0,0,1~0,0,1~0,0,2",
            "0,0~0,0,1",
            "0,0,1,2~0,0,1",
            "-1,0,1~0,0,1",
            "+0,0,1~0,0,1",
            "0, 0,1~0,0,1",
            "0,0,1~0,0,1\n\n",
            "0,0,1~0,0,1\njunk",
        ] {
            assert!(part1(input).is_err(), "{input:?}");
            assert!(part2(input).is_err(), "{input:?}");
        }
        let max = usize::MAX;
        for input in [
            format!("{max},{max},1~{max},{max},{max}"),
            format!("0,0,{max}~0,0,{max}"),
        ] {
            assert_eq!(part1(&input).unwrap(), "1");
            assert_eq!(part2(&input).unwrap(), "0");
        }
        let overflow = format!("{}0,0,1~0,0,1", max);
        assert!(part1(&overflow).is_err());
        assert_eq!(part1("0,0,1~0,0,1\r\n").unwrap(), "1");
    }

    #[test]
    fn reversed_endpoints_describe_the_same_bricks() {
        let input = "0,0,1~0,0,3\n0,0,8~2,0,8\n2,0,12~2,2,12\n2,2,16~2,2,16";
        let expected = parse_bricks(input).unwrap();
        for mask in 0..16 {
            let reversed = input
                .lines()
                .enumerate()
                .map(|(i, line)| {
                    let (a, b) = line.split_once('~').unwrap();
                    if mask & (1 << i) != 0 {
                        format!("{b}~{a}")
                    } else {
                        line.to_owned()
                    }
                })
                .join("\n");
            let actual = parse_bricks(&reversed).unwrap();
            assert_eq!(actual.len(), expected.len());
            for (a, b) in actual.iter().zip(&expected) {
                assert_eq!(
                    (a.start.x, a.start.y, a.start.z),
                    (b.start.x, b.start.y, b.start.z)
                );
                assert_eq!((a.end.x, a.end.y, a.end.z), (b.end.x, b.end.y, b.end.z));
                assert_eq!(a.supports, b.supports);
                assert_eq!(a.supported_by, b.supported_by);
            }
            assert_eq!(part1(&reversed).unwrap(), part1(input).unwrap());
            assert_eq!(part2(&reversed).unwrap(), part2(input).unwrap());
        }
    }

    #[test]
    fn rejects_non_line_bricks_with_line_context() {
        for invalid in ["0,0,1~1,1,1", "0,0,1~1,0,2", "0,0,1~0,1,2", "0,0,1~1,1,2"] {
            let input = format!("4,4,1~4,4,1\n{invalid}");
            for solve in [part1, part2] {
                let error = solve(&input).unwrap_err().to_string();
                assert!(error.contains("line 2:"));
                assert!(error.contains("straight axis-aligned line"));
            }
        }
    }

    #[test]
    fn rejects_ground_and_initial_overlaps() {
        for input in ["0,0,0~0,0,0", "0,0,3~0,0,0"] {
            for solve in [part1, part2] {
                let error = solve(input).unwrap_err().to_string();
                assert!(error.contains("line 1:"));
                assert!(error.contains("above ground"));
            }
        }
        for input in [
            "0,0,2~2,0,2\n1,0,1~1,0,3",
            "0,0,2~2,0,2\n2,0,2~2,0,2",
            "0,0,2~0,0,2\n0,0,2~0,0,2",
            "0,0,1~0,0,5\n0,0,4~0,0,2",
        ] {
            for solve in [part1, part2] {
                let error = solve(input).unwrap_err().to_string();
                assert!(error.contains("line 2:"));
                assert!(error.contains("overlaps the brick on line 1"));
            }
        }
        for input in [
            "0,0,1~0,0,1\n1,0,1~1,0,1",
            "0,0,1~0,0,1\n0,0,2~0,0,2",
            "0,0,1~0,0,1\n0,1,1~0,1,1",
        ] {
            assert!(part1(input).is_ok());
            assert!(part2(input).is_ok());
        }
    }

    fn falling_reference(input: &str) -> Vec<Brick> {
        let mut bricks: Vec<_> = input
            .lines()
            .map(|line| {
                let (start, end) = line.split_once('~').unwrap();
                Brick {
                    start: parse_point_3d(start).unwrap(),
                    end: parse_point_3d(end).unwrap(),
                    supports: vec![],
                    supported_by: vec![],
                }
            })
            .collect();
        bricks.sort_by_key(|b| b.start.z);
        let mut settled: Vec<Brick> = vec![];
        for mut brick in bricks {
            while brick.start.z > 1 {
                let blocked = settled.iter().any(|b| {
                    (brick.start.x..=brick.end.x).any(|x| {
                        (brick.start.y..=brick.end.y).any(|y| {
                            x >= b.start.x
                                && x <= b.end.x
                                && y >= b.start.y
                                && y <= b.end.y
                                && brick.start.z - 1 == b.end.z
                        })
                    })
                });
                if blocked {
                    break;
                }
                brick.start.z -= 1;
                brick.end.z -= 1;
            }
            settled.push(brick);
        }
        for child in 0..settled.len() {
            for parent in 0..child {
                let a = &settled[parent];
                let b = &settled[child];
                let touching = a.end.z + 1 == b.start.z
                    && (a.start.x..=a.end.x).any(|x| {
                        (a.start.y..=a.end.y).any(|y| {
                            x >= b.start.x && x <= b.end.x && y >= b.start.y && y <= b.end.y
                        })
                    });
                if touching {
                    settled[parent].supports.push(child);
                    settled[child].supported_by.push(parent);
                }
            }
        }
        settled
    }

    fn physically_fallen_count(bricks: &[Brick], removed: usize) -> usize {
        let mut survivors: Vec<_> = bricks
            .iter()
            .enumerate()
            .filter(|(index, _)| *index != removed)
            .map(|(_, brick)| brick.clone())
            .collect();
        survivors.sort_by_key(|brick| brick.start.z);
        let mut occupied = std::collections::HashSet::new();
        let mut fallen = 0;
        for mut brick in survivors {
            let initial_z = brick.start.z;
            while brick.start.z > 1
                && !(brick.start.x..=brick.end.x).any(|x| {
                    (brick.start.y..=brick.end.y)
                        .any(|y| occupied.contains(&(x, y, brick.start.z - 1)))
                })
            {
                brick.start.z -= 1;
                brick.end.z -= 1;
            }
            fallen += usize::from(brick.start.z != initial_z);
            for x in brick.start.x..=brick.end.x {
                for y in brick.start.y..=brick.end.y {
                    for z in brick.start.z..=brick.end.z {
                        assert!(occupied.insert((x, y, z)));
                    }
                }
            }
        }
        fallen
    }

    #[test]
    fn cascade_handles_shared_supports_and_rejoining_paths() {
        let bricks =
            parse_bricks("0,0,1~2,0,1\n0,0,2~0,0,2\n2,0,2~2,0,2\n0,0,3~2,0,3\n1,0,4~1,0,4")
                .unwrap();
        for removed in 0..bricks.len() {
            assert_eq!(
                cascade_size(&bricks, removed),
                physically_fallen_count(&bricks, removed)
            );
        }
        assert_eq!(cascade_size(&bricks, 0), 4);
        assert_eq!(cascade_size(&bricks, 1), 0);
        assert_eq!(cascade_size(&bricks, 3), 1);
    }

    #[test]
    fn direct_landing_matches_unit_falls() {
        for seed in 0..256usize {
            let mut state = seed + 1;
            let mut input = String::new();
            for i in 0..12 {
                state = state.wrapping_mul(1664525).wrapping_add(1013904223);
                let x = state % 5;
                let y = (state / 5) % 5;
                let z = 1 + i * 4;
                let (dx, dy, dz) = match state % 3 {
                    0 => (2, 0, 0),
                    1 => (0, 2, 0),
                    _ => (0, 0, 2),
                };
                input.push_str(&format!("{x},{y},{z}~{},{},{}\n", x + dx, y + dy, z + dz));
            }
            let actual = parse_bricks(&input).unwrap();
            let expected = falling_reference(&input);
            for removed in 0..actual.len() {
                assert_eq!(
                    cascade_size(&actual, removed),
                    physically_fallen_count(&actual, removed),
                    "seed {seed}, removed {removed}"
                );
            }
            for (a, b) in actual.iter().zip(&expected) {
                assert_eq!((a.start.z, a.end.z), (b.start.z, b.end.z));
                assert_eq!(a.supports, b.supports);
                let mut actual_supporters = a.supported_by.clone();
                actual_supporters.sort_unstable();
                assert_eq!(actual_supporters, b.supported_by);
            }
        }
    }

    #[test]
    fn bridge_touches_only_highest_supports() {
        let bricks = parse_bricks("0,0,1~0,0,3\n1,0,1~1,0,1\n2,0,1~2,0,3\n0,0,9~2,0,9").unwrap();
        assert_eq!(bricks[3].start.z, 4);
        let mut supporters = bricks[3].supported_by.clone();
        supporters.sort_unstable();
        assert_eq!(supporters, vec![0, 2]);
        assert!(bricks[1].supports.is_empty());
    }

    const YEAR: i16 = 2023;
    const DAY: i16 = 22;

    #[test]
    fn test_fn1_input() {
        scaffold_test_result(YEAR, DAY, "input.txt", "input-spec.1.txt", part1);
    }

    #[test]
    fn test_fn2_input() {
        scaffold_test_result(YEAR, DAY, "input.txt", "input-spec.2.txt", part2);
    }
}
