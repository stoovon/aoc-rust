extern crate core;

use std::hash::{Hash, Hasher};
use std::ops::Index;

///// TODO: Extract to library

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

impl Point {
    #[inline]
    #[must_use]
    pub const fn new(x: i32, y: i32) -> Self {
        Point { x, y }
    }

    #[inline]
    #[must_use]
    pub fn manhattan(self, other: Self) -> i32 {
        (self.x - other.x).abs() + (self.y - other.y).abs()
    }
}

impl Hash for Point {
    #[inline]
    fn hash<H: Hasher>(&self, hasher: &mut H) {
        hasher.write_i32(self.x);
        hasher.write_i32(self.y);
    }
}

#[derive(Clone)]
pub struct Grid<T> {
    pub width: i32,
    pub height: i32,
    pub bytes: Vec<T>,
}

impl Grid<u8> {
    pub fn parse(input: &str) -> Self {
        let raw: Vec<_> = input.lines().map(str::as_bytes).collect();
        let width = raw[0].len() as i32;
        let height = raw.len() as i32;
        let mut bytes = Vec::with_capacity((width * height) as usize);
        raw.iter().for_each(|slice| bytes.extend_from_slice(slice));
        Grid { width, height, bytes }
    }
}

impl<T> Index<Point> for Grid<T> {
    type Output = T;

    #[inline]
    fn index(&self, point: Point) -> &Self::Output {
        &self.bytes[(self.width * point.y + point.x) as usize]
    }
}

/////

pub struct Parsed {
    points: Vec<Point>,
    horizontal: Vec<i32>,
    vertical: Vec<i32>,
}

pub fn parse(input: &str) -> Parsed {
    let grid: Grid<u8> = Grid::parse(input);
    let size = grid.width as usize;

    let mut points_seen = Vec::new();
    let mut rows = vec![true; size];
    let mut columns = vec![true; size];

    for y in 0..grid.height {
        for x in 0..grid.width {
            let point = Point::new(x, y);
            if grid[point] == b'#' {
                points_seen.push(point);
                rows[y as usize] = false;
                columns[x as usize] = false;
            }
        }
    }

    // [Prefix sum](https://en.wikipedia.org/wiki/Prefix_sum)
    let mut empty_rows = 0; // Track empty rows until now
    let mut empty_cols = 0; // Track empty columns until now

    let mut horizontal = vec![0; size];
    let mut vertical = vec![0; size];

    for i in 0..size {
        empty_rows += rows[i] as i32;
        empty_cols += columns[i] as i32;
        horizontal[i] = empty_rows;
        vertical[i] = empty_cols;
    }

    Parsed { points: points_seen, horizontal, vertical }
}

fn expand(input: &Parsed, times: i32) -> u64 {
    let mut result = 0;
    let points: Vec<_> = input
        .points
        .iter()
        .map(|p| {
            // Each point's position (empty cols up and left) is expanded by the ratio.
            let x = p.x + times * input.vertical[p.x as usize];
            let y = p.y + times * input.horizontal[p.y as usize];
            Point::new(x, y)
        })
        .collect();

    for (i, p1) in points.iter().enumerate().skip(1) {
        result += points.iter().take(i).map(|&p2| p1.manhattan(p2) as u64).sum::<u64>();
    }

    result
}

pub fn fn2(input: &str, times: i32) -> i64 {
    let grid = parse(input);
    expand(&grid, times) as i64
}

#[cfg(test)]
mod tests {
    use super::*;
    use svutils::load_spec;

    #[test]
    fn test_fn1_example() {
        assert_eq!(fn2(include_str!("example.txt"), 1), load_spec(include_str!("example-spec.1.txt")));
    }

    #[test]
    fn test_fn1_input() {
        assert_eq!(fn2(include_str!("input.txt"), 1), load_spec(include_str!("input-spec.1.txt")));
    }

    #[test]
    fn test_fn2_example() {
        assert_eq!(fn2(include_str!("example.txt"), 99), load_spec(include_str!("example-spec.2.txt")));
    }

    #[test]
    fn test_fn2_input() {
        assert_eq!(fn2(include_str!("input.txt"), 999999), load_spec(include_str!("input-spec.2.txt")));
    }
}