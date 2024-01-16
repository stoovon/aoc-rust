extern crate core;

use regex::Regex;
use std::collections::{HashMap, HashSet};

fn build_screen(input: &str) -> [[bool; 50]; 6] {
    let rxrect = Regex::new(r"^rect (\d*)x(\d*)$").unwrap();
    let rxrow = Regex::new(r"^rotate row y=(\d*) by (\d*)$").unwrap();
    let rxcol = Regex::new(r"^rotate column x=(\d*) by (\d*)$").unwrap();

    const W: usize = 50;
    const H: usize = 6;

    let mut screen = [[false; W]; H];

    input.lines().for_each(|f| {
        if let Some(caps) = rxrect.captures(f) {
            let x: usize = caps[1].parse().unwrap();
            let y: usize = caps[2].parse().unwrap();
            for i in 0..y {
                for j in 0..x {
                    screen[i][j] = true;
                }
            }
        } else if let Some(caps) = rxrow.captures(f) {
            let y: usize = caps[1].parse().unwrap();
            let n: usize = caps[2].parse().unwrap();
            let mut row = [false; W];
            for i in 0..W {
                row[(i + n) % W] = screen[y][i];
            }
            screen[y] = row;
        } else if let Some(caps) = rxcol.captures(f) {
            let x: usize = caps[1].parse().unwrap();
            let n: usize = caps[2].parse().unwrap();
            let mut col = [false; H];
            for i in 0..H {
                col[(i + n) % H] = screen[i][x];
            }
            for i in 0..H {
                screen[i][x] = col[i];
            }
        }
    });

    screen
}

pub fn fn1(input: &str) -> i64 {
    let screen = build_screen(input);

    screen.iter().fold(0, |acc, row| {
        acc + row.iter().fold(0, |acc, &col| if col { acc + 1 } else { acc })
    })
}

pub fn fn2(input: &str) -> i64 {
    let screen = build_screen(input);

    for row in screen.iter() {
        for col in row.iter() {
            print!("{}", if *col { '#' } else { '.' });
        }
        println!();
    }

    0
}

#[cfg(test)]
mod tests {
    use super::*;
    use svutils::scaffold_test;

    const YEAR: i16 = 2016;
    const DAY: i16 = 8;

    #[test]
    fn test_fn1_input() {
        scaffold_test(YEAR, DAY, "input.txt", "input-spec.1.txt", fn1);
    }

    #[test]
    fn test_fn2_input() {
        scaffold_test(YEAR, DAY, "input.txt", "input-spec.2.txt", fn2);
    }
}
