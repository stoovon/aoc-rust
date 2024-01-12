extern crate core;

fn parse(input: &str) -> Vec<Instruction> {
    input
        .split(", ")
        .map(|s| {
            let (dir, dist) = s.split_at(1);
            let dist = dist.parse::<i64>().unwrap();
            match dir {
                "L" => Instruction::Left(dist),
                "R" => Instruction::Right(dist),
                _ => panic!("Invalid direction: {}", dir),
            }
        })
        .collect()
}

#[derive(Debug)]
enum Instruction {
    Left(i64),
    Right(i64),
}

impl Instruction {
    fn follow(&self, cur_dir: &Direction) -> (Direction, i64) {
        match self {
            Instruction::Left(n) => (cur_dir.turn_left(), *n),
            Instruction::Right(n) => (cur_dir.turn_right(), *n),
        }
    }
}

enum Direction {
    North,
    South,
    East,
    West,
}

impl Direction {
    fn turn_right(&self) -> Direction {
        match self {
            Direction::North => Direction::East,
            Direction::South => Direction::West,
            Direction::East => Direction::South,
            Direction::West => Direction::North,
        }
    }

    fn turn_left(&self) -> Direction {
        match self {
            Direction::North => Direction::West,
            Direction::South => Direction::East,
            Direction::East => Direction::North,
            Direction::West => Direction::South,
        }
    }
}

pub fn fn1(input: &str) -> i64 {
    let instrs = parse(input);

    let mut x = 0i64;
    let mut y = 0i64;
    let mut dir = Direction::North;

    for i in instrs {
        let (new_dir, dist) = i.follow(&dir);
        dir = new_dir;

        match dir {
            Direction::North => y -= dist,
            Direction::South => y += dist,
            Direction::East => x += dist,
            Direction::West => x -= dist,
        }
    }
    
    x.abs() + y.abs()
}

pub fn fn2(input: &str) -> i64 {
    let instrs = parse(input);

    let mut visited = std::collections::HashSet::new();
    visited.insert((0i64, 0i64));

    let mut x = 0i64;
    let mut y = 0i64;
    let mut dir = Direction::North;

    for i in instrs {
        let (new_dir, dist) = i.follow(&dir);
        dir = new_dir;

        let (dy, dx) = match dir {
            Direction::North => (0, -1),
            Direction::South => (0, 1),
            Direction::East => (1, 0),
            Direction::West => (-1, 0),
        };

        for _ in 0..dist {
            x += dx;
            y += dy;

            if !visited.insert((x, y)) {
                return x.abs() + y.abs();
            }
        }
    }

    0
}

#[cfg(test)]
mod tests {
    use super::*;
    use svutils::scaffold_test;

    const YEAR: i16 = 2016;
    const DAY: i16 = 1;

    #[test]
    fn test_fn1_example() {
        scaffold_test(YEAR, DAY, "example.txt", "example-spec.1.txt", fn1);
    }

    #[test]
    fn test_fn1_input() {
        scaffold_test(YEAR, DAY, "input.txt", "input-spec.1.txt", fn1);
    }

    
    #[test]
    fn test_fn2_input() {
        scaffold_test(YEAR, DAY, "input.txt", "input-spec.2.txt", fn2);
    }
}
