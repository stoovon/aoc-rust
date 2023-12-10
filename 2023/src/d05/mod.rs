extern crate core;

use std::fmt::Debug;
use std::ops::Range;
use std::str::FromStr;

// Again, building up ideas of parsing, I will harmonise this into a good solution.

pub fn tokens<T>(input: &str, sep: Option<&str>) -> Vec<T>
where
    T: FromStr + Debug,
    <T as FromStr>::Err: Debug,
{
    if let Some(sep) = sep {
        input
            .split(sep)
            .filter(|v| !v.is_empty())
            .flat_map(|v| v.parse().ok())
            .collect()
    } else {
        input
            .split_whitespace()
            .flat_map(|v| v.parse().ok())
            .collect()
    }
}

pub fn token_groups<T>(input: &str, sep: &str, inner_sep: Option<&str>) -> Vec<Vec<T>>
where
    T: FromStr + Debug,
    <T as FromStr>::Err: Debug,
{
    input
        .split(sep)
        .filter(|l| !l.is_empty())
        .map(|sub| tokens(sub, inner_sep))
        .collect()
}

fn convert(maps: &[Vec<(Range<i64>, i64)>], mut num: i64) -> i64 {
    for map in maps {
        for (range, dst) in map {
            if range.contains(&num) {
                num = dst + num - range.start;
                break;
            }
        }
    }
    num
}

fn convert_inv(maps: &[Vec<(Range<i64>, i64)>], num: i64, depth: i64) -> i64 {
    convert(&maps[(maps.len() - depth as usize)..], num)
}

pub fn solve(input: &str) -> (i64, i64) {
    let maps: Vec<Vec<(i64, i64, i64)>> = token_groups(input, "\n\n", None)
        .into_iter()
        .skip(1)
        .map(|map| {
            let map: Vec<i64> = map
                .into_iter()
                .skip(2)
                .map(|v: String| v.parse().unwrap())
                .collect();
            map.chunks(3).map(|v| (v[0], v[1], v[2])).collect()
        })
        .collect();

    let seeds: Vec<i64> = tokens(input.lines().next().unwrap(), None);

    let maps: Vec<Vec<(Range<i64>, i64)>> = maps
        .iter()
        .map(|v| {
            v.iter()
                .map(|(dst, src, len)| ((*src..(src + len), *dst)))
                .collect()
        })
        .collect();

    let part1 = seeds.iter().map(|s| convert(&maps, *s)).min().unwrap();

    // The approach for part 2 is to keep track of ranges, and cut them
    // up into smaller parts based on how they intersect with the references.
    // This finishes far more efficiently than tracking each seed (in the order of a hundred or so pieces of data)

    let mut maps_inv: Vec<Vec<(Range<i64>, i64)>> = maps.to_vec();
    maps_inv.reverse();
    let maps_inv: Vec<Vec<(Range<i64>, i64)>> = maps_inv
        .iter()
        .map(|map| {
            map.iter()
                .map(|(r, dst)| {
                    let len = r.end - r.start as i64;
                    let src = r.start;
                    (*dst..(dst + len), src)
                })
                .collect()
        })
        .collect();

    let inverted_boundaries: Vec<i64> = maps
        .iter()
        .enumerate()
        .flat_map(|(depth, map)| map.iter().map(move |(r, _)| (depth, r.clone())))
        .flat_map(|(d, r)| [(d, r.start), (d, r.end - 1)])
        .map(|(d, v)| convert_inv(&maps_inv, v, d as i64))
        .collect();

    let part2 = seeds
        .chunks(2)
        .map(|c| {
            let r = c[0]..(c[0] + c[1]);
            let mut to_check = inverted_boundaries.clone();
            to_check.push(c[0]);
            to_check.push(c[0] + c[1] - 1);
            to_check
                .into_iter()
                .flat_map(|p| {
                    if r.contains(&p) {
                        Some(convert(&maps, p))
                    } else {
                        None
                    }
                })
                .min()
                .unwrap()
        })
        .min()
        .unwrap();

    (part1, part2)
}

pub fn fn1(input: &str) -> i64 {
    solve(input).0
}

pub fn fn2(input: &str) -> i64 {
    solve(input).1
}

#[cfg(test)]
mod tests {
    use super::*;
    use svutils::load_spec;

    #[test]
    fn test_fn1_example() {
        assert_eq!(
            fn1(include_str!("example.txt")) as i64,
            load_spec(include_str!("example-spec.1.txt"))
        );
    }

    #[test]
    fn test_fn1_input() {
        assert_eq!(
            fn1(include_str!("input.txt")) as i64,
            load_spec(include_str!("input-spec.1.txt"))
        );
    }

    #[test]
    fn test_fn2_example() {
        assert_eq!(
            fn2(include_str!("example.txt")) as i64,
            load_spec(include_str!("example-spec.2.txt"))
        );
    }

    #[test]
    fn test_fn2_input() {
        assert_eq!(
            fn2(include_str!("input.txt")) as i64,
            load_spec(include_str!("input-spec.2.txt"))
        );
    }
}
