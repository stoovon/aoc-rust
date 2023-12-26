extern crate core;

use itertools::Itertools;
use priority_queue::priority_queue;
use priority_queue::PriorityQueue;

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

fn parse_point_3d(s: &str) -> Point3D {
    let (x, y, z) = s
        .split(",")
        .map(|s| s.parse::<usize>().unwrap())
        .collect_tuple()
        .unwrap();

    Point3D { x, y, z }
}

fn point_between(p: usize, range: (usize, usize)) -> bool {
    p >= range.0 && p <= range.1
}

fn has_collision(brick_a: &Brick, brick_b: &Brick) -> bool {
    (point_between(brick_a.start.z, (brick_b.start.z, brick_b.end.z))
        || point_between(brick_a.end.z, (brick_b.start.z, brick_b.end.z))
        || point_between(brick_b.start.z, (brick_a.start.z, brick_a.end.z))
        || point_between(brick_b.end.z, (brick_a.start.z, brick_a.end.z)))
        && (point_between(brick_a.start.x, (brick_b.start.x, brick_b.end.x))
            || point_between(brick_a.end.x, (brick_b.start.x, brick_b.end.x))
            || point_between(brick_b.start.x, (brick_a.start.x, brick_a.end.x))
            || point_between(brick_b.end.x, (brick_a.start.x, brick_a.end.x)))
        && (point_between(brick_a.start.y, (brick_b.start.y, brick_b.end.y))
            || point_between(brick_a.end.y, (brick_b.start.y, brick_b.end.y))
            || point_between(brick_b.start.y, (brick_a.start.y, brick_a.end.y))
            || point_between(brick_b.end.y, (brick_a.start.y, brick_a.end.y)))
}

fn drop(brick: &mut Brick) {
    if brick.start.z > 1 {
        brick.start.z -= 1;
        brick.end.z -= 1;
    }
}

fn raise(brick: &mut Brick) {
    brick.start.z += 1;
    brick.end.z += 1;
}

fn parse_bricks(input: &str) -> Vec<Brick> {
    input
        .lines()
        .map(|line| {
            let (start, end) = line.split_once("~").unwrap();

            Brick {
                start: parse_point_3d(start),
                end: parse_point_3d(end),
                supports: vec![],
                supported_by: vec![],
            }
        })
        .sorted_by_key(|brick| brick.start.z) // TREMENDOUS PERFORMANCE BOOST
        .fold(
            (Vec::<Brick>::new(), 1),
            |(mut result, max_z_reached), mut brick| {
                // position just above max_z_reached
                let diff = brick.end.z - brick.start.z;
                brick.start.z = max_z_reached + 1;
                brick.end.z = brick.start.z + diff;

                loop {
                    let mut can_move_down = true;

                    drop(&mut brick);

                    for i in (0..result.len()).rev() {
                        if result[i].end.z < brick.start.z {
                            continue;
                        }

                        let collision = has_collision(&brick, &result[i]);

                        if collision {
                            can_move_down = false;
                            let len = result.len();
                            result[i].supports.push(len);
                            brick.supported_by.push(i);
                        }
                    }

                    if !can_move_down {
                        // cannot move any further down (collided with some brick)
                        // revert and break
                        raise(&mut brick);
                        break;
                    }

                    // reached bottom
                    if brick.start.z == 1 {
                        break;
                    }
                }

                let new_max_z_reached = max_z_reached.max(brick.end.z);

                result.push(brick);

                (result, new_max_z_reached)
            },
        )
        .0
}

pub fn fn1(input: &str) -> i64 {
    let bricks = parse_bricks(input);

    bricks
        .iter()
        .filter(|b| {
            b.supports
                .iter()
                .all(|&idx| bricks[idx].supported_by.len() > 1)
        })
        .count() as i64
}

pub fn fn2(input: &str) -> i64 {
    let bricks = parse_bricks(input);

    let mut sum = 0;

    // BFS
    for i in (0..bricks.len()).rev() {
        if bricks[i].supports.is_empty() {
            continue;
        }

        let mut unsupported_indexes: Vec<usize> = vec![];
        let mut pq = PriorityQueue::<usize, usize>::new();

        pq.push(i, bricks[i].end.z);

        while !pq.is_empty() {
            let (idx, _) = pq.pop().unwrap();

            unsupported_indexes.push(idx);

            let brick = &bricks[idx];

            for supported_brick_idx in &brick.supports {
                let supported_brick = &bricks[*supported_brick_idx];

                if supported_brick
                    .supported_by
                    .iter()
                    .all(|idx| unsupported_indexes.iter().rev().contains(idx))
                {
                    pq.push(*supported_brick_idx, supported_brick.end.z);
                }
            }
        }

        sum += unsupported_indexes.len() - 1;
    }

    sum as i64
}

#[cfg(test)]
mod tests {
    use super::*;
    use svutils::scaffold_test;

    const YEAR: i16 = 2023;
    const DAY: i16 = 22;

    #[test]
    fn test_fn1_example() {
        scaffold_test(YEAR, DAY, "example.txt", "example-spec.1.txt", fn1);
    }

    #[test]
    fn test_fn1_input() {
        scaffold_test(YEAR, DAY, "input.txt", "input-spec.1.txt", fn1);
    }

    #[test]
    fn test_fn2_example() {
        scaffold_test(YEAR, DAY, "example.txt", "example-spec.2.txt", fn2);
    }

    #[test]
    fn test_fn2_input() {
        scaffold_test(YEAR, DAY, "input.txt", "input-spec.2.txt", fn2);
    }
}
