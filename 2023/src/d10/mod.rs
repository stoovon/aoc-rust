extern crate core;

use std::ops::{Add, AddAssign, Index, IndexMut};
use std::hash::{Hash, Hasher};

type Input = (i32, i32);

pub const ORIGIN: Point = Point::new(0, 0);
pub const UP: Point = Point::new(0, -1);
pub const DOWN: Point = Point::new(0, 1);
pub const LEFT: Point = Point::new(-1, 0);
pub const RIGHT: Point = Point::new(1, 0);
pub const ORTHOGONAL: [Point; 4] = [UP, DOWN, LEFT, RIGHT];

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
}

impl Add for Point {
    type Output = Self;

    #[inline]
    #[must_use]
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

impl Hash for Point {
    #[inline]
    fn hash<H: Hasher>(&self, hasher: &mut H) {
        hasher.write_u32(self.x as u32);
        hasher.write_u32(self.y as u32);
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

impl<T: Copy + PartialEq> Grid<T> {
    pub fn find(&self, needle: T) -> Option<Point> {
        let to_point = |index| {
            let x = (index as i32) % self.width;
            let y = (index as i32) / self.width;
            Point::new(x, y)
        };
        self.bytes.iter().position(|&h| h == needle).map(to_point)
    }
}

impl<T> Index<Point> for Grid<T> {
    type Output = T;

    #[inline]
    fn index(&self, point: Point) -> &Self::Output {
        &self.bytes[(self.width * point.y + point.x) as usize]
    }
}

impl<T> IndexMut<Point> for Grid<T> {
    #[inline]
    fn index_mut(&mut self, point: Point) -> &mut Self::Output {
        &mut self.bytes[(self.width * point.y + point.x) as usize]
    }
}

pub fn parse(input: &str) -> Input {
    let grid = Grid::parse(input);
    let determinant = |a: Point, b: Point| a.x * b.y - a.y * b.x;

    // Some people hard-coded this but this is nice
    let mut corner = grid.find(b'S').unwrap();

    let mut direction = if matches!(grid[corner + UP], b'|' | b'7' | b'F') { UP } else { DOWN };
    let mut position = corner + direction;

    let mut perimeter = 1;
    let mut area = 0;

    loop {
        // Follow straight paths.
        while grid[position] == b'-' || grid[position] == b'|' {
            position += direction;
            perimeter += 1;
        }

        // Inspired by sea monster
        direction = match grid[position] {
            b'7' if direction == UP => LEFT,
            b'F' if direction == UP => RIGHT,
            b'J' if direction == DOWN => LEFT,
            b'L' if direction == DOWN => RIGHT,
            b'J' | b'L' => UP,
            b'7' | b'F' => DOWN,
            _ => {
                // We're back at start
                area += determinant(corner, position);
                break;
            }
        };

        area += determinant(corner, position);
        corner = position;
        position += direction;
        perimeter += 1;
    }

    // Shoelace
    let part_one = perimeter / 2;

    // Pick's
    let part_two = area.abs() / 2 - perimeter / 2 + 1;
    (part_one, part_two)
}

pub fn fn1(input: &str) -> i64 {
    parse(input).0 as i64
}

pub fn fn2(input: &str) -> i64 {
    parse(input).1 as i64
}

#[cfg(test)]
mod tests {
    use super::*;
    use svutils::load_spec;

    #[test]
    fn test_example() {
        assert_eq!(fn1(include_str!("example.txt")), load_spec(include_str!("example-spec.1.txt")));
    }

    #[test]
    fn test_fn1_input() {
        assert_eq!(fn1(include_str!("input.txt")), load_spec(include_str!("input-spec.1.txt")));
    }

    #[test]
    fn test_fn2_input() {
        assert_eq!(fn2(include_str!("input.txt")), load_spec(include_str!("input-spec.2.txt")));
    }
}