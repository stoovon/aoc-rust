mod linear;

use self::linear::LinearSystem;
use num_bigint::BigInt;
use num_rational::BigRational;
use num_traits::{Signed, ToPrimitive, Zero};
use svutils::SolveError;

type Vector = [BigInt; 3];

#[derive(Debug, Clone)]
struct Hailstone {
    position: Vector,
    velocity: Vector,
}

fn parse_vector(text: &str) -> Result<Vector, SolveError> {
    let coordinates = text
        .split(',')
        .map(|field| {
            let field = field.trim();
            let digits = field
                .strip_prefix('-')
                .or_else(|| field.strip_prefix('+'))
                .unwrap_or(field);
            if digits.is_empty() || !digits.bytes().all(|c| c.is_ascii_digit()) {
                return Err(SolveError::InvalidInput(
                    "expected a signed decimal integer coordinate".into(),
                ));
            }
            field
                .parse::<BigInt>()
                .map_err(|_| SolveError::InvalidInput("invalid integer coordinate".into()))
        })
        .collect::<Result<Vec<_>, _>>()?;
    coordinates
        .try_into()
        .map_err(|_| SolveError::InvalidInput("expected three coordinates".into()))
}

fn parse(input: &str) -> Result<Vec<Hailstone>, SolveError> {
    if input.trim().is_empty() {
        return Err(SolveError::EmptyInput);
    }
    input
        .lines()
        .enumerate()
        .map(|(index, line)| {
            let parse_line = || {
                let (position, velocity) = line.split_once('@').ok_or_else(|| {
                    SolveError::InvalidInput("expected position @ velocity".into())
                })?;
                Ok(Hailstone {
                    position: parse_vector(position)?,
                    velocity: parse_vector(velocity)?,
                })
            };
            parse_line().map_err(|error: SolveError| error.at_line(index + 1))
        })
        .collect()
}

fn cross_xy(a: &Vector, b: &Vector) -> BigInt {
    &a[0] * &b[1] - &a[1] * &b[0]
}

fn stationary_crossing(point: &Hailstone, moving: &Hailstone, min: &BigInt, max: &BigInt) -> bool {
    if point.position[..2]
        .iter()
        .any(|value| value < min || value > max)
    {
        return false;
    }
    let delta: Vector = std::array::from_fn(|axis| &point.position[axis] - &moving.position[axis]);
    if !cross_xy(&delta, &moving.velocity).is_zero() {
        return false;
    }
    if let Some(axis) = (0..2).find(|&axis| !moving.velocity[axis].is_zero()) {
        delta[axis].is_zero() || delta[axis].sign() == moving.velocity[axis].sign()
    } else {
        delta[0].is_zero() && delta[1].is_zero()
    }
}

fn crosses_in_window(
    a: &Hailstone,
    b: &Hailstone,
    min: &BigInt,
    max: &BigInt,
) -> Result<bool, SolveError> {
    if a.velocity[0].is_zero() && a.velocity[1].is_zero() {
        return Ok(stationary_crossing(a, b, min, max));
    }
    if b.velocity[0].is_zero() && b.velocity[1].is_zero() {
        return Ok(stationary_crossing(b, a, min, max));
    }
    let delta: Vector = std::array::from_fn(|axis| &b.position[axis] - &a.position[axis]);
    let mut determinant = cross_xy(&a.velocity, &b.velocity);
    if determinant.is_zero() {
        if cross_xy(&delta, &a.velocity).is_zero() {
            return Err(SolveError::InvalidInput(
                "collinear projected paths do not have a unique line intersection".into(),
            ));
        }
        return Ok(false);
    }
    let mut a_time = cross_xy(&delta, &b.velocity);
    let mut b_time = cross_xy(&delta, &a.velocity);
    if determinant.is_negative() {
        determinant = -determinant;
        a_time = -a_time;
        b_time = -b_time;
    }
    if a_time.is_negative() || b_time.is_negative() {
        return Ok(false);
    }
    // Compare scaled integer coordinates instead of allocating fractions per pair.
    let lower = min * &determinant;
    let upper = max * &determinant;
    Ok((0..2).all(|axis| {
        let coordinate = &a.position[axis] * &determinant + &a.velocity[axis] * &a_time;
        coordinate >= lower && coordinate <= upper
    }))
}

#[derive(Clone, Copy)]
struct SmallProjection {
    position: [i128; 2],
    velocity: [i128; 2],
}

impl SmallProjection {
    fn from_stone(stone: &Hailstone) -> Option<Self> {
        Some(Self {
            position: [stone.position[0].to_i128()?, stone.position[1].to_i128()?],
            velocity: [stone.velocity[0].to_i128()?, stone.velocity[1].to_i128()?],
        })
    }
}

fn checked_cross(a: [i128; 2], b: [i128; 2]) -> Option<i128> {
    a[0].checked_mul(b[1])?.checked_sub(a[1].checked_mul(b[0])?)
}

// Any unrepresentable intermediate or degenerate geometry uses the BigInt path.
fn small_crossing(a: SmallProjection, b: SmallProjection, min: i128, max: i128) -> Option<bool> {
    if a.velocity == [0, 0] || b.velocity == [0, 0] {
        return None;
    }
    let delta = [
        b.position[0].checked_sub(a.position[0])?,
        b.position[1].checked_sub(a.position[1])?,
    ];
    let mut determinant = checked_cross(a.velocity, b.velocity)?;
    if determinant == 0 {
        return None;
    }
    let mut a_time = checked_cross(delta, b.velocity)?;
    let mut b_time = checked_cross(delta, a.velocity)?;
    if determinant < 0 {
        determinant = determinant.checked_neg()?;
        a_time = a_time.checked_neg()?;
        b_time = b_time.checked_neg()?;
    }
    if a_time < 0 || b_time < 0 {
        return Some(false);
    }
    let lower = min.checked_mul(determinant)?;
    let upper = max.checked_mul(determinant)?;
    for axis in 0..2 {
        let coordinate = a.position[axis]
            .checked_mul(determinant)?
            .checked_add(a.velocity[axis].checked_mul(a_time)?)?;
        if coordinate < lower || coordinate > upper {
            return Some(false);
        }
    }
    Some(true)
}

fn count_crossings(input: &str, min: &BigInt, max: &BigInt) -> Result<usize, SolveError> {
    if min > max {
        return Err(SolveError::InvalidInput(
            "intersection window minimum exceeds maximum".into(),
        ));
    }
    let hailstones = parse(input)?;
    let bounds = min.to_i128().zip(max.to_i128());
    let small: Vec<_> = hailstones.iter().map(SmallProjection::from_stone).collect();
    let mut count = 0usize;
    for (index, a) in hailstones.iter().enumerate() {
        for (other, b) in hailstones.iter().enumerate().skip(index + 1) {
            let fast = bounds.and_then(|(min, max)| {
                small[index]
                    .zip(small[other])
                    .and_then(|(a, b)| small_crossing(a, b, min, max))
            });
            let crosses = match fast {
                Some(value) => value,
                None => crosses_in_window(a, b, min, max)?,
            };
            if crosses {
                count = count.checked_add(1).ok_or(SolveError::ArithmeticOverflow)?;
            }
        }
    }
    Ok(count)
}

pub fn part1(input: &str) -> Result<String, SolveError> {
    count_crossings(
        input,
        &BigInt::from(200_000_000_000_000i64),
        &BigInt::from(400_000_000_000_000i64),
    )
    .map(|count| count.to_string())
}

fn add_pair(system: &mut LinearSystem, a: &Hailstone, b: &Hailstone) -> Result<(), SolveError> {
    // Subtracting two collision equations cancels the common position × velocity term.
    for (i, j) in [(0, 1), (0, 2), (1, 2)] {
        let mut row = std::array::from_fn(|_| BigInt::zero());
        row[i] = &a.velocity[j] - &b.velocity[j];
        row[j] = &b.velocity[i] - &a.velocity[i];
        row[3 + i] = &b.position[j] - &a.position[j];
        row[3 + j] = &a.position[i] - &b.position[i];
        row[6] = &a.position[i] * &a.velocity[j]
            - &a.position[j] * &a.velocity[i]
            - &b.position[i] * &b.velocity[j]
            + &b.position[j] * &b.velocity[i];
        system.add_row(row.map(BigRational::from_integer))?;
    }
    Ok(())
}

fn collision_time(rock: &[BigRational; 6], stone: &Hailstone) -> Result<BigRational, SolveError> {
    let mut time = None;
    for axis in 0..3 {
        let separation = BigRational::from_integer(stone.position[axis].clone()) - &rock[axis];
        let relative_velocity =
            &rock[axis + 3] - BigRational::from_integer(stone.velocity[axis].clone());
        if relative_velocity.is_zero() {
            if !separation.is_zero() {
                return Err(SolveError::InvalidInput(
                    "rock does not collide with hailstone".into(),
                ));
            }
        } else {
            let axis_time = separation / relative_velocity;
            if axis_time.is_negative() || time.as_ref().is_some_and(|t| t != &axis_time) {
                return Err(SolveError::InvalidInput(
                    "rock must collide at one nonnegative time in all three coordinates".into(),
                ));
            }
            time = Some(axis_time);
        }
    }
    Ok(time.unwrap_or_else(BigRational::zero))
}

fn solve(hailstones: &[Hailstone]) -> Result<[BigRational; 6], SolveError> {
    let mut system = LinearSystem::new();
    for stone in &hailstones[1..] {
        add_pair(&mut system, &hailstones[0], stone)?;
        if system.is_complete() {
            break;
        }
    }
    let rock = system.solution()?;
    for (index, stone) in hailstones.iter().enumerate() {
        collision_time(&rock, stone).map_err(|error| error.at_line(index + 1))?;
    }
    Ok(rock)
}

pub fn part2(input: &str) -> Result<String, SolveError> {
    let rock = solve(&parse(input)?)?;
    if !rock.iter().all(BigRational::is_integer) {
        return Err(SolveError::InvalidInput(
            "rock position and velocity must be integral".into(),
        ));
    }
    Ok((&rock[0] + &rock[1] + &rock[2]).to_integer().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scene(position: [i64; 3], velocity: [i64; 3], fractional: bool) -> String {
        let directions = [
            [2, 4, -6],
            [-4, 6, 2],
            [8, -2, 4],
            [6, 8, -4],
            [-6, -4, 8],
            [4, -8, -2],
        ];
        let mut input = String::new();
        for (index, direction) in directions.iter().enumerate() {
            let numerator = (2 * index + 1) as i64;
            let denominator = if fractional { 2 } else { 1 };
            let p: [i64; 3] = std::array::from_fn(|axis| {
                position[axis] + direction[axis] * numerator / denominator
            });
            let u: [i64; 3] = std::array::from_fn(|axis| velocity[axis] - direction[axis]);
            input.push_str(&format!(
                "{}, {}, {} @ {}, {}, {}\n",
                p[0], p[1], p[2], u[0], u[1], u[2]
            ));
        }
        input
    }

    #[test]
    fn recovers_generated_rocks_with_integer_and_fractional_times() {
        for seed in 0..64 {
            let position = [17 + seed, -23 + 2 * seed, 41 - seed];
            let velocity = [701 + seed, -907 - seed, 1201 + seed];
            for fractional in [false, true] {
                let input = scene(position, velocity, fractional);
                let hailstones = parse(&input).unwrap();
                let rock = solve(&hailstones).unwrap();
                assert_eq!(
                    rock,
                    [
                        position[0],
                        position[1],
                        position[2],
                        velocity[0],
                        velocity[1],
                        velocity[2]
                    ]
                    .map(|v| BigRational::from_integer(v.into()))
                );
                assert_eq!(
                    part2(&input).unwrap(),
                    position.iter().sum::<i64>().to_string()
                );
                assert_eq!(
                    collision_time(&rock, &hailstones[0]).unwrap().is_integer(),
                    !fractional
                );
            }
        }
    }

    #[test]
    fn handles_dependent_first_stones_and_checks_every_later_stone() {
        let input = scene([17, -23, 41], [701, -907, 1201], true);
        let first = input.lines().next().unwrap();
        let redundant = format!("{first}\n{first}\n{input}");
        assert_eq!(part2(&redundant).unwrap(), "35");
        let mut stones = parse(&input).unwrap();
        stones.last_mut().unwrap().position[0] += 1;
        assert!(solve(&stones).is_err());
        assert!(part2(first)
            .unwrap_err()
            .to_string()
            .contains("unique rock"));
    }

    #[test]
    fn verifies_zero_relative_velocity_shared_paths_and_past_collisions() {
        let rock = [17, -23, 41, 7, -9, 12].map(|v| BigRational::from_integer(v.into()));
        let mut stone = Hailstone {
            position: [17, -23, 41].map(BigInt::from),
            velocity: [7, -9, 12].map(BigInt::from),
        };
        assert_eq!(collision_time(&rock, &stone).unwrap(), BigRational::zero());
        stone.position[0] += 1;
        assert!(collision_time(&rock, &stone).is_err());
        stone.velocity[0] -= 2;
        assert_eq!(
            collision_time(&rock, &stone).unwrap(),
            BigRational::new(1.into(), 2.into())
        );
        stone.position[0] -= 2;
        assert!(collision_time(&rock, &stone).is_err());
    }

    #[test]
    fn projected_crossings_handle_vertical_paths_time_and_boundaries() {
        for (input, min, max, expected) in [
            ("0,0,9 @ 0,2,-1\n-3,4,7 @ 1,0,4", 0, 4, 1),
            ("0,0,9 @ 0,-2,-1\n-3,4,7 @ 1,0,4", 0, 4, 0),
            ("4,0,9 @ 0,2,0\n0,4,7 @ 1,0,0", 4, 4, 1),
            ("0,0,0 @ 1,1,0\n1,0,0 @ -1,1,0", 0, 1, 1),
            ("0,0,0 @ 1,1,0\n0,1,0 @ 1,1,0", 0, 4, 0),
            ("4,4,0 @ 0,0,1\n0,4,0 @ 1,0,1", 0, 4, 1),
            ("4,4,0 @ 0,0,1\n4,4,0 @ 0,0,-1", 4, 4, 1),
            ("4,4,0 @ 1,0,1\n4,4,0 @ 0,1,-1", 4, 4, 1),
        ] {
            assert_eq!(
                count_crossings(input, &min.into(), &max.into()).unwrap(),
                expected
            );
        }
        assert!(count_crossings("0,0,0 @ 1,1,0\n1,1,0 @ 1,1,0", &0.into(), &4.into()).is_err());
        assert!(count_crossings("0,0,0 @ 1,1,0", &4.into(), &0.into()).is_err());
    }

    #[test]
    fn projected_boundaries_remain_exact_above_float_precision() {
        let boundary: BigInt = BigInt::from(10u8).pow(40);
        let previous = &boundary - 1;
        let input = format!("{boundary},{boundary},0 @ 0,1,0\n{previous},{boundary},0 @ 3,1,0");
        assert_eq!(count_crossings(&input, &boundary, &boundary).unwrap(), 0);
    }

    #[test]
    fn projected_crossings_match_constructed_intersections() {
        let mut checked = 0;
        for seed in 0..512i64 {
            let (x, y) = (seed % 17 - 8, (seed / 17) % 17 - 8);
            let a_velocity = [seed % 7 - 3, (seed / 7) % 7 - 3, 0];
            let b_velocity = [(seed / 11) % 7 - 3, (seed / 13) % 7 - 3, 0];
            if a_velocity[0] * b_velocity[1] == a_velocity[1] * b_velocity[0] {
                continue;
            }
            let a_time = (seed / 3) % 7 - 3;
            let b_time = (seed / 5) % 7 - 3;
            let a = Hailstone {
                position: [x - a_velocity[0] * a_time, y - a_velocity[1] * a_time, 0]
                    .map(BigInt::from),
                velocity: a_velocity.map(BigInt::from),
            };
            let b = Hailstone {
                position: [x - b_velocity[0] * b_time, y - b_velocity[1] * b_time, 0]
                    .map(BigInt::from),
                velocity: b_velocity.map(BigInt::from),
            };
            let expected =
                a_time >= 0 && b_time >= 0 && (-5..=5).contains(&x) && (-5..=5).contains(&y);
            assert_eq!(
                crosses_in_window(&a, &b, &(-5).into(), &5.into()).unwrap(),
                expected
            );
            assert_eq!(
                crosses_in_window(&b, &a, &(-5).into(), &5.into()).unwrap(),
                expected
            );
            let small_a = SmallProjection::from_stone(&a).unwrap();
            let small_b = SmallProjection::from_stone(&b).unwrap();
            assert_eq!(small_crossing(small_a, small_b, -5, 5), Some(expected));
            assert_eq!(small_crossing(small_b, small_a, -5, 5), Some(expected));
            checked += 1;
        }
        assert!(checked > 300);
    }

    #[test]
    fn test_part1_input() {
        svutils::scaffold_test_result(2023, 24, "input.txt", "input-spec.1.txt", part1);
    }
    #[test]
    fn test_part2_input() {
        svutils::scaffold_test_result(2023, 24, "input.txt", "input-spec.2.txt", part2);
    }
    #[test]
    fn checked_projection_falls_back_on_overflow_and_degeneracy() {
        let cases = [
            ([i128::MIN, 0, 1, 0], [i128::MAX, 1, 0, -1], -5, 5),
            ([0, 0, i128::MAX, 0], [1, 1, 0, 2], -5, 5),
            ([0, 0, i128::MIN, 0], [0, 0, 0, 1], -5, 5),
            ([0, 0, 1, 0], [1, 1, 0, -2], i128::MAX, i128::MAX),
            ([i128::MAX, 0, 1, 0], [i128::MAX, 1, 0, -2], -5, 5),
            ([0, 0, 1, 1], [1, 1, 1, 1], -5, 5),
            ([0, 0, 0, 0], [1, 1, -1, -1], -5, 5),
        ];
        for (a, b, min, max) in cases {
            let projection = |values: [i128; 4]| SmallProjection {
                position: [values[0], values[1]],
                velocity: [values[2], values[3]],
            };
            assert!(small_crossing(projection(a), projection(b), min, max).is_none());
            let stone = |values: [i128; 4]| Hailstone {
                position: [
                    BigInt::from(values[0]),
                    BigInt::from(values[1]),
                    BigInt::zero(),
                ],
                velocity: [
                    BigInt::from(values[2]),
                    BigInt::from(values[3]),
                    BigInt::zero(),
                ],
            };
            let (a, b) = (stone(a), stone(b));
            let input = format!(
                "{},{},0 @ {},{},0\n{},{},0 @ {},{},0",
                a.position[0],
                a.position[1],
                a.velocity[0],
                a.velocity[1],
                b.position[0],
                b.position[1],
                b.velocity[0],
                b.velocity[1]
            );
            let expected = crosses_in_window(&a, &b, &min.into(), &max.into());
            let actual = count_crossings(&input, &min.into(), &max.into());
            match expected {
                Ok(crossing) => assert_eq!(actual.unwrap(), usize::from(crossing)),
                Err(_) => assert!(actual.is_err()),
            }
        }
    }
    #[test]
    fn verifies_original_equations_after_solving_the_differences() {
        let mut input = String::new();
        for (a, b, c) in [
            (1, 2, 3),
            (2, 3, 5),
            (3, 5, 7),
            (5, 7, 11),
            (7, 11, 13),
            (11, 13, 17),
        ] {
            let p = [1 + a * b, b, 0];
            let u = [c * p[0] + a, c * p[1] + 1, 0];
            input.push_str(&format!("{},{},0 @ {},{},0\n", p[0], p[1], u[0], u[1]));
        }
        let stones = parse(&input).unwrap();
        let mut system = LinearSystem::new();
        for stone in &stones[1..] {
            add_pair(&mut system, &stones[0], stone).unwrap();
        }
        assert_eq!(
            system.solution().unwrap(),
            std::array::from_fn(|_| BigRational::zero())
        );
        assert!(part2(&input).is_err());
    }

    #[test]
    fn part2_rejects_fractional_rocks_and_past_collisions() {
        let position_twice = [35, -45, 83];
        let velocity_twice = [1403, -1813, 2403];
        let mut fractional_rock = String::new();
        for (i, u) in [
            [-3, 2, 4],
            [5, -7, 2],
            [8, 9, -5],
            [-11, 13, 17],
            [19, -23, 29],
            [-31, 37, 41],
        ]
        .iter()
        .enumerate()
        {
            let time = (2 * i + 1) as i64;
            let p: [i64; 3] = std::array::from_fn(|axis| {
                (position_twice[axis] + velocity_twice[axis] * time) / 2 - u[axis] * time
            });
            fractional_rock.push_str(&format!(
                "{},{},{} @ {},{},{}\n",
                p[0], p[1], p[2], u[0], u[1], u[2]
            ));
        }
        assert!(part2(&fractional_rock)
            .unwrap_err()
            .to_string()
            .contains("must be integral"));
        let mut past = scene([17, -23, 41], [701, -907, 1201], true);
        let p = [17 - 701 + 5, -23 + 907 + 7, 41 - 1201 + 11];
        past.push_str(&format!("{},{},{} @ 5,7,11\n", p[0], p[1], p[2]));
        assert!(part2(&past)
            .unwrap_err()
            .to_string()
            .contains("nonnegative time"));
    }

    #[test]
    fn part2_is_order_independent_and_supports_large_integer_answers() {
        let input = scene([17, -23, 41], [701, -907, 1201], true);
        let lines: Vec<_> = input.lines().collect();
        for rotation in 0..lines.len() {
            let mut ordered = lines.clone();
            ordered.rotate_left(rotation);
            for reverse in [false, true] {
                if reverse {
                    ordered.reverse();
                }
                assert_eq!(part2(&ordered.join("\n")).unwrap(), "35");
            }
        }
        let shift = BigInt::from(10).pow(80);
        let stones = parse(&input).unwrap();
        let shifted = stones
            .iter()
            .map(|stone| {
                format!(
                    "{},{},{} @ {},{},{}",
                    &stone.position[0] + &shift,
                    &stone.position[1] + &shift,
                    &stone.position[2] + &shift,
                    stone.velocity[0],
                    stone.velocity[1],
                    stone.velocity[2]
                )
            })
            .collect::<Vec<_>>()
            .join("\n");
        assert_eq!(
            part2(&shifted).unwrap(),
            (shift * BigInt::from(3) + BigInt::from(35)).to_string()
        );
    }

    #[test]
    fn varied_generated_geometries_recover_the_known_rock() {
        for seed in 0..32u64 {
            let position = [17 + seed as i64, -23 + seed as i64, 41 - seed as i64];
            let velocity = [701, -907, 1201];
            let mut state = seed + 1;
            let mut input = String::new();
            for i in 0..8 {
                let mut relative = [0i64; 3];
                for value in &mut relative {
                    state = state.wrapping_mul(1664525).wrapping_add(1013904223);
                    *value = 2 * ((state % 19) as i64 - 9);
                }
                let numerator = 2 * i + 1;
                let p: [i64; 3] =
                    std::array::from_fn(|axis| position[axis] + relative[axis] * numerator / 2);
                let u: [i64; 3] = std::array::from_fn(|axis| velocity[axis] - relative[axis]);
                input.push_str(&format!(
                    "{},{},{} @ {},{},{}\n",
                    p[0], p[1], p[2], u[0], u[1], u[2]
                ));
            }
            let rock = solve(&parse(&input).unwrap()).unwrap();
            for axis in 0..3 {
                assert_eq!(rock[axis], BigRational::from_integer(position[axis].into()));
                assert_eq!(
                    rock[axis + 3],
                    BigRational::from_integer(velocity[axis].into())
                );
            }
        }
    }

    #[test]
    fn common_parser_validates_both_parts_and_crlf() {
        for input in [
            "",
            " \n",
            "1,2 @ 3,4,5",
            "1,2,3,4 @ 5,6,7",
            "1,2,3 @ 4,5,6 @ 7,8,9",
            "NaN,2,3 @ 4,5,6",
            "1.5,2,3 @ 4,5,6",
            "1,2,3 @ 4,5,6\n\n",
            "--1,2,3 @ 4,5,6",
            "-+1,2,3 @ 4,5,6",
        ] {
            for solve in [part1, part2] {
                assert!(solve(input).is_err());
            }
        }
        for solve in [part1, part2] {
            assert!(solve("1,2,3 @ 4,5,6\nbad")
                .unwrap_err()
                .to_string()
                .starts_with("line 2:"));
        }
        let input = scene([17, -23, 41], [701, -907, 1201], true);
        assert_eq!(part2(&input.replace('\n', "\r\n")).unwrap(), "35");
        let stone = parse(" +1, -2, 0003 @ -4, +5, 0\r\n").unwrap();
        assert_eq!(stone[0].position, [1, -2, 3].map(BigInt::from));
    }
}
