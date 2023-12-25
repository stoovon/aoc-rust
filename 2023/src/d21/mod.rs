extern crate core;

use nalgebra::{DMatrix, DVector};
use std::collections::HashSet;

type Point = (isize, isize);
struct Map {
    width: usize,
    _height: usize,
    tiles: Vec<Vec<char>>,
}

impl Map {
    fn from_str(input: &str) -> Map {
        let tiles: Vec<Vec<char>> = input.lines().map(|line| { line.chars().collect()}).collect();
        let _height = tiles.len();
        let width = tiles.first().map_or(0, |row| row.len());
        Map { width, _height, tiles }
    }

    fn find_char(&self, needle: char) -> (isize, isize) {
        for (y, row) in self.tiles.iter().enumerate() {
            for (x, ch) in row.iter().enumerate() {
                if *ch == needle {
                    return (x as isize, y as isize);
                }
            }
        }
        panic!("No start position");
    }

    fn translate(&self, pos: Point) -> (usize, usize) {
        let (x, y) = pos;
        let width = self.width as isize;
        let rx = (x % width + width) % width;
        let ry = (y % width + width) % width;
        (rx as usize, ry as usize)
    }

    fn get(&self, pos: Point) -> char {
        let (x, y) = self.translate(pos);
        self.tiles[y][x]
    }

    fn get_moves(&self, pos: Point) -> Vec<Point> {
        let (x, y) = pos;
        let moves = vec![(x-1, y), (x+1, y), (x, y-1), (x, y+1)];
        moves.iter().filter(|&pos| self.get(*pos) != '#').copied().collect()
    }
}

pub fn fn1(input: &str) -> i64 {
    let map = Map::from_str(&input);
    let mut cur_pos: Vec<Point> = vec![map.find_char('S')];
    
    for _ in 0..64 {
        let mut new_pos: HashSet<Point> = HashSet::new();
        for pos in cur_pos.drain(..) {
            for npos in map.get_moves(pos) {
                new_pos.insert(npos);
            }
        }
        cur_pos.extend(new_pos); 
    }
    cur_pos.len() as i64
}

pub fn fn2(input: &str) -> i64 {
    let map = Map::from_str(&input);

    // 202300 * 131 + 65, where 131 is map width
    let max_steps = 26501365;
    let mut cur_pos: Vec<Point> = vec![map.find_char('S')];
    
    let mut res: Vec<u64> = vec![];
    let mut i=0;
    
    loop {
        let mut new_pos: HashSet<Point> = HashSet::new();
        for pos in cur_pos.drain(..) {
            for npos in map.get_moves(pos) {
                new_pos.insert(npos);
            }
        }
        cur_pos.extend(new_pos); 
        i += 1;

        if i%map.width == max_steps%map.width  {
             println!("iter {}, val {}", i, cur_pos.len());
             res.push(cur_pos.len() as u64);
             if res.len()==3 {
                break;
            }
        }
    }

    // solve system of quadratic equations
    let x: [f64; 3] = [0.0, 1.0, 2.0];
    let y: Vec<f64> = res.iter().map(|x| *x as f64).collect();

    let a_matrix = DMatrix::from_row_slice(3, 3, &[
        x[0].powi(2), x[0], 1.0,
        x[1].powi(2), x[1], 1.0,
        x[2].powi(2), x[2], 1.0,
    ]);

    let b_vector = DVector::from_column_slice(&y);
    match a_matrix.clone().lu().solve(&b_vector) {
        Some(solution) => {
            let a = solution[0] as i64;
            let b = solution[1] as i64;
            let c = solution[2] as i64;
            println!("Quadratic polynomial coefficients: a = {a}, b = {b}, c = {c}");

            let x = (max_steps/map.width) as i64;
            return a*x*x + b*x + c
        },
        None => println!("Can't solve equations system"),
    }
    unreachable!()
}

#[cfg(test)]
mod tests {
    use super::*;
    use svutils::load_spec;

    // #[test]
    // fn test_fn1_example() {
    //     assert_eq!(fn1(include_str!("example.txt")), load_spec(include_str!("example-spec.1.txt")));
    // }

    #[test]
    fn test_fn1_input() {
        assert_eq!(fn1(include_str!("../../../input/2023/d21/input.txt")), load_spec(include_str!("../../../input/2023/d21/input-spec.1.txt")));
    }

    #[test]
    fn test_fn2_input() {
        assert_eq!(fn2(include_str!("../../../input/2023/d21/input.txt")), load_spec(include_str!("../../../input/2023/d21/input-spec.2.txt")));
    }

}