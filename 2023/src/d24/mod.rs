extern crate core;

use itertools::Itertools;
use std::{fmt::Display, str::FromStr};

const EPSILON: f64 = 0.0001;

#[derive(Debug, Clone, Copy, PartialEq)]
struct Vec3 {
    x: f64,
    y: f64,
    z: f64,
}

impl FromStr for Vec3 {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut it = s.trim().split(", ");
        let x = it.next().ok_or("no x".to_string())?.trim();
        let y = it.next().ok_or("no y".to_string())?.trim();
        let z = it.next().ok_or("no z".to_string())?.trim();

        let x = x
            .parse::<f64>()
            .map_err(|_| format!("could not parse x: {}", x))?;
        let y = y
            .parse::<f64>()
            .map_err(|_| "could not parse y".to_string())?;
        let z = z
            .parse::<f64>()
            .map_err(|_| "could not parse y".to_string())?;

        Ok(Self { x, y, z })
    }
}

impl Display for Vec3 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}, {}, {}", self.x, self.y, self.z)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct Hailstone {
    pos: Vec3,
    vel: Vec3,
}

impl FromStr for Hailstone {
    type Err = String;

    fn from_str(line: &str) -> Result<Self, Self::Err> {
        let mut it = line.trim().split('@');
        let start = it.next().ok_or("no start".to_string())?;
        let velocity = it.next().ok_or("no velocity".to_string())?;

        let start = start.parse::<Vec3>()?;
        let velocity = velocity.parse::<Vec3>()?;

        Ok(Self { pos: start, vel: velocity })
    }
}

impl Hailstone {
    fn intersect_2d(&self, other: &Self) -> (f64, f64, (f64, f64)) {
        let Hailstone {
            pos:
                Vec3 {
                    x: p1x,
                    y: p1y,
                    z: _,
                },
            vel:
                Vec3 {
                    x: v1x,
                    y: v1y,
                    z: _,
                },
        } = *self;

        let Hailstone {
            pos:
                Vec3 {
                    x: p2x,
                    y: p2y,
                    z: _,
                },
            vel:
                Vec3 {
                    x: v2x,
                    y: v2y,
                    z: _,
                },
        } = *other;

        // solving the equation system
        let t2 = ((p2y - p1y) * v1x - (p2x - p1x) * v1y) / (v2x * v1y - v2y * v1x);
        let t1 = (p2x - p1x + t2 * v2x) / v1x;

        let x = p1x + t1 * v1x;
        let y = p1y + t1 * v1y;

        (t1, t2, (x, y))
    }
}

impl Display for Hailstone {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} @ {}", self.pos, self.vel)
    }
}

fn parse_input(input: &str) -> Vec<Hailstone> {
    let line_count = input.trim().lines().count();
    let mut result = Vec::with_capacity(line_count);

    for line in input.trim().lines() {
        result.push(line.parse::<Hailstone>().unwrap())
    }

    result
}

pub fn fn1(input: &str, min: f64, max: f64) -> i64 {
    let hailstones = parse_input(input);

    let mut result = 0;

    for (a, b) in hailstones.iter().tuple_combinations() {
        let (t1, t2, (x, y)) = a.intersect_2d(b);
        if t1.is_sign_positive()
            && t2.is_sign_positive()
            && x >= min
            && y >= min
            && x <= max
            && y <= max
        {
            result += 1;
            // println!(
            //     "A: {}\nB: {}\nintersection within relevant area at x: {}, y: {} at time t1: {}, t2: {}",
            //     a, b, x, y, t1, t2
            // );
        }
    }

    result
}

fn solve_part2(a: Hailstone, b: Hailstone, vx: f64, vy: f64, vz: f64) -> Option<(f64, f64, Vec3)> {
    let Hailstone {
        pos: Vec3 {
            x: pax,
            y: pay,
            z: paz,
        },
        vel: Vec3 {
            x: vax,
            y: vay,
            z: vaz,
        },
    } = a;

    let Hailstone {
        pos: Vec3 {
            x: pbx,
            y: pby,
            z: pbz,
        },
        vel: Vec3 {
            x: vbx,
            y: vby,
            z: vbz,
        },
    } = b;

    let t2_numerator = pby - pay - (((vay - vy) * (pbx - pax)) / (vax - vx));
    let t2_denominator = vy - vby - (((vay - vy) * (vx - vbx)) / (vax - vx));

    let t2 = t2_numerator / t2_denominator;

    let t1 = (pbx - pax - t2 * (vx - vbx)) / (vax - vx);

    let px = pax - t1 * (vx - vax);
    let py = pay - t1 * (vy - vay);
    let pz = paz - t1 * (vz - vaz);

    if (pz + t2 * (vz - vbz) - pbz).abs() > EPSILON {
        None
    } else {
        Some((
            t1,
            t2,
            Vec3 {
                x: px,
                y: py,
                z: pz,
            },
        ))
    }
}
pub fn fn2(input: &str) -> i64 {
    // Works well but takes about 30 seconds.
    // It's possible to solve more quickly using Sympy or Z3.

    let hailstones = parse_input(input);

    let a = hailstones[0];
    let b = hailstones[1];

    let is_int = |f: f64| (f.round() - f).abs() < EPSILON;

    for vx in -500..500 {
        for vy in -500..500 {
            'outer: for vz in -500..500 {
                let vx = vx as f64;
                let vy = vy as f64;
                let vz = vz as f64;

                if let Some((
                    t1,
                    t2,
                    Vec3 {
                        x: px,
                        y: py,
                        z: pz,
                    },
                )) = solve_part2(a, b, vx, vy, vz)
                {
                    if !(t1.is_finite()
                        && t2.is_finite()
                        && px.is_finite()
                        && py.is_finite()
                        && pz.is_finite())
                    {
                        // No +/-Inf or NaN
                        continue;
                    }

                    if t1.is_sign_negative() || t2.is_sign_negative() {
                        // No time travel
                        continue;
                    }

                    if !(is_int(t1) && is_int(t2) && is_int(px) && is_int(py) && is_int(pz)) {
                        // Only integers
                        continue;
                    }

                    for i in 2..hailstones.len() {
                        let c = hailstones[i];

                        let Hailstone {
                            pos:
                                Vec3 {
                                    x: pcx,
                                    y: pcy,
                                    z: pcz,
                                },
                            vel:
                                Vec3 {
                                    x: vcx,
                                    y: vcy,
                                    z: vcz,
                                },
                        } = c;

                        let t3 = (pcx - px) / (vx - vcx);

                        if (py + t3 * vy - (pcy + t3 * vcy)).abs() > EPSILON
                            || (pz + t3 * vz - (pcz + t3 * vcz)).abs() > EPSILON
                        {
                            // I still want to improve on this, used it too many times now.
                            continue 'outer;
                        }
                    }

                    return px as i64 + py as i64 + pz as i64;
                }
            }
        }
    }

    panic!("found no solution");
}

#[cfg(test)]
mod tests {
    use super::*;
    use svutils::load_spec;

    #[test]
    fn test_fn1_example() {
        assert_eq!(fn1(include_str!("../../../input/2023/d24/example.txt"), 7., 27.), load_spec(include_str!("../../../input/2023/d24/example-spec.1.txt")));
    }

    #[test]
    fn test_fn1_input() {
        assert_eq!(fn1(include_str!("../../../input/2023/d24/input.txt"), 200000000000000., 400000000000000.), load_spec(include_str!("../../../input/2023/d24/input-spec.1.txt")));
    }

    #[test]
    fn test_fn2_example() {
        assert_eq!(fn2(include_str!("../../../input/2023/d24/example.txt")), load_spec(include_str!("../../../input/2023/d24/example-spec.2.txt")));
    }

    #[test]
    fn test_fn2_input() {
        assert_eq!(fn2(include_str!("../../../input/2023/d24/input.txt")), load_spec(include_str!("../../../input/2023/d24/input-spec.2.txt")));
    }

}