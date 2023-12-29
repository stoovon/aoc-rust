extern crate core;

enum Instruction {
    Rotate(Turn),
    Forward(u8),
}

enum Turn {
    L,
    R,
}

#[derive(PartialEq)]
enum Tile {
    Open,
    Solid,
    None,
}

#[derive(Clone)]
struct Coord {
    row: i32,
    col: i32,
}

enum Direction {
    L,
    R,
    U,
    D,
}

impl Direction {
    // This feels quite a lot like a Rubik's solver.
    fn score(&self) -> usize {
        use Direction::*;
        match self {
            R => 0,
            D => 1,
            L => 2,
            U => 3,
        }
    }

    fn turn(self, turn: &Turn) -> Direction {
        use Direction::*;
        match (self, turn) {
            (L, Turn::L) => D,
            (L, Turn::R) => U,
            (R, Turn::L) => U,
            (R, Turn::R) => D,
            (U, Turn::L) => L,
            (U, Turn::R) => R,
            (D, Turn::L) => R,
            (D, Turn::R) => L,
        }
    }

    fn offset(&self) -> Coord {
        use Direction::*;
        match &self {
            L => Coord { row: 0, col: -1 },
            R => Coord { row: 0, col: 1 },
            U => Coord { row: -1, col: 0 },
            D => Coord { row: 1, col: 0 },
        }
    }
}

fn parse(input: &str) -> (Vec<Vec<Tile>>, Vec<Instruction>) {
    // Starting whitespace is significant
    let (grid, moves) = input.trim_end().split_once("\n\n").unwrap();
    let mut instructions = Vec::new();
    let mut digits = Vec::new();
    for c in moves.chars() {
        if c.is_numeric() {
            // accumulate digits
            let digit = c.to_digit(10).unwrap() as u8;
            digits.push(digit);
        } else {
            // Accumulate digits and construct the number.
            let num = digits.iter().fold(0, |num, digit| num * 10 + digit);
            digits.clear();
            instructions.push(Instruction::Forward(num));

            // parse turn
            let turn = match c {
                'L' => Turn::L,
                'R' => Turn::R,
                _ => panic!("Invalid input"),
            };
            instructions.push(Instruction::Rotate(turn));
        }
    }
    // Accumulate digits again. This is at least the third time I've done this (2023d03 is another example). Helper function on the way.
    let num = digits.iter().fold(0, |num, digit| num * 10 + digit);
    instructions.push(Instruction::Forward(num));

    let mut map = Vec::new();
    for line in grid.lines() {
        let mut row = Vec::new();
        for c in line.chars() {
            let tile = match c {
                '.' => Tile::Open,
                '#' => Tile::Solid,
                ' ' => Tile::None,
                _ => panic!("invalid input"),
            };
            row.push(tile);
        }
        map.push(row);
    }

    (map, instructions)
}

fn wrap_2d(map: &[Vec<Tile>], pos: &Coord, dir: &Direction) -> Coord {
    let Coord { row: dr, col: dc } = dir.offset();
    let mut curr = pos.clone();
    // while an open or solid tile exists in the map when walking in the opposite direction, update pos
    while let Some(tile) = map
        .get((curr.row - dr) as usize)
        .and_then(|row| row.get((curr.col - dc) as usize))
    {
        if *tile == Tile::None {
            break;
        }
        curr = Coord {
            row: curr.row - dr,
            col: curr.col - dc,
        };
    }

    curr
}

fn example_strat(rowp: i32, colp: i32, dir: &Direction) -> (i32, i32, Direction) {
    let (cube_row, cube_col, new_dir) = match (rowp, colp, dir) {
        // FIXME: I think we can replace with a solution that will work for all cubes.
        // Probably worth revisiting, using 3d vectors.
        // Once I add a cube data structure, I can probably use that.

        /*
                    0                 1                 2                 3

                                                        A
                                                        v
        0                                    G -->  | (0,2) | <-- F
                                                        ^
                                                        |
                    A                 G                 |
                    v                 v                 v
        1 B --> | (1,0) | <---->  | (1,1) | <----> |  (1,2) | <-- E
                    ^                 ^                 ^
                    C                 D                 |
                                                        |                 E
                                                        v                 v
        2                                    D -->  | (2,2) | <---->  | (2,3) | <-- F
                                                        ^                 ^
                                                        C                 B
         */
        (0, 2, Direction::U) => (1, 0, Direction::D), // FOLLOW A
        (0, 2, Direction::R) => (2, 3, Direction::L), // FOLLOW F
        (0, 2, Direction::D) => (1, 2, Direction::D), // INTRINSIC
        (0, 2, Direction::L) => (1, 1, Direction::D), // FOLLOW G

        (1, 0, Direction::U) => (0, 2, Direction::D), // FOLLOW A
        (1, 0, Direction::R) => (1, 1, Direction::R), // INTRINSIC
        (1, 0, Direction::D) => (2, 2, Direction::U), // FOLLOW C
        (1, 0, Direction::L) => (2, 3, Direction::U), // FOLLOW B

        (1, 1, Direction::U) => (0, 2, Direction::R), // FOLLOW G
        (1, 1, Direction::R) => (1, 2, Direction::R), // INTRINSIC
        (1, 1, Direction::D) => (2, 2, Direction::R), // FOLLOW D
        (1, 1, Direction::L) => (1, 0, Direction::L), // INTRINSIC

        (1, 2, Direction::U) => (0, 2, Direction::U), // INTRINSIC
        (1, 2, Direction::R) => (2, 3, Direction::D), // FOLLOW E
        (1, 2, Direction::D) => (2, 2, Direction::D), // INTRINSIC
        (1, 2, Direction::L) => (1, 1, Direction::L), // INTRINSIC

        (2, 2, Direction::U) => (1, 2, Direction::U), // INTRINSIC
        (2, 2, Direction::R) => (2, 3, Direction::R), // INTRINSIC
        (2, 2, Direction::D) => (1, 0, Direction::U), // FOLLOW C
        (2, 2, Direction::L) => (1, 1, Direction::U), // FOLLOW D

        (2, 3, Direction::U) => (1, 2, Direction::L), // FOLLOW E
        (2, 3, Direction::R) => (0, 2, Direction::L), // FOLLOW F
        (2, 3, Direction::D) => (1, 0, Direction::R), // FOLLOW B
        (2, 3, Direction::L) => (2, 2, Direction::L), // INTRINSIC

        _ => unreachable!(),
    };

    (cube_row, cube_col, new_dir)
}

fn input_strat(rowp: i32, colp: i32, dir: &Direction) -> (i32, i32, Direction) {
    let (cube_row, cube_col, new_dir) = match (rowp, colp, dir) {
        /*
             0                        1                 2

                                      B                 C
                                      v                 v
        0                  A -->  | (0,1) | <---->  | (0,2) | <-- D
                                                        ^
                                                        E

        1                  G -->  | (1,1) | <-- E


                    G
                    v
        2 A --> | (2,0) | <---->  | (2,1) | <-- D
                                      ^
                                      F

        3 B --> | (3,0) | <-- F
                    ^
                    C
         */
        (2, 0, Direction::U) => (1, 1, Direction::R), // FOLLOW G
        (2, 0, Direction::R) => (2, 1, Direction::R), // INTRINSIC
        (2, 0, Direction::D) => (3, 0, Direction::D), // INTRINSIC
        (2, 0, Direction::L) => (0, 1, Direction::R), // FOLLOW A

        (3, 0, Direction::U) => (2, 0, Direction::U), // INTRINSIC
        (3, 0, Direction::R) => (2, 1, Direction::U), // FOLLOW F
        (3, 0, Direction::D) => (0, 2, Direction::D), // FOLLOW C
        (3, 0, Direction::L) => (0, 1, Direction::D), // FOLLOW B

        (0, 1, Direction::U) => (3, 0, Direction::R), // FOLLOW B
        (0, 1, Direction::R) => (0, 2, Direction::R), // INTRINSIC
        (0, 1, Direction::D) => (1, 1, Direction::D), // INTRINSIC
        (0, 1, Direction::L) => (2, 0, Direction::R), // FOLLOW A

        (1, 1, Direction::U) => (0, 1, Direction::U), // INTRINSIC
        (1, 1, Direction::R) => (0, 2, Direction::U), // FOLLOW E
        (1, 1, Direction::D) => (2, 1, Direction::L), // INTRINSIC
        (1, 1, Direction::L) => (2, 0, Direction::D), // FOLLOW G

        (0, 2, Direction::U) => (3, 0, Direction::U), // FOLLOW C
        (0, 2, Direction::R) => (2, 1, Direction::L), // FOLLOW D
        (0, 2, Direction::D) => (1, 1, Direction::L), // FOLLOW E
        (0, 2, Direction::L) => (0, 1, Direction::L), // INTRINSIC

        (2, 1, Direction::U) => (1, 1, Direction::U), // INTRINSIC
        (2, 1, Direction::R) => (0, 2, Direction::L), // FOLLOW D
        (2, 1, Direction::D) => (3, 0, Direction::L), // FOLLOW F
        (2, 1, Direction::L) => (2, 0, Direction::L), // INTRINSIC

        _ => unreachable!(),
    };

    (cube_row, cube_col, new_dir)
}

fn wrap_3d(pos: &Coord, dir: &Direction, edge_length: i32) -> (Coord, Direction) {
    let rowp = pos.row / edge_length;
    let colp = pos.col / edge_length;

    let cube_row: i32;
    let cube_col: i32;
    let new_dir: Direction;

    if edge_length == 50 {
        (cube_row, cube_col, new_dir) = input_strat(rowp, colp, dir);
    } else if edge_length == 4 {
        (cube_row, cube_col, new_dir) = example_strat(rowp, colp, dir);
    } else {
        unreachable!()
    }

    // find idxes within the cube
    let (row_idx, col_idx) = (pos.row % edge_length, pos.col % edge_length);

    let i = match dir {
        Direction::L => (edge_length - 1) - row_idx,
        Direction::R => row_idx,
        Direction::U => col_idx,
        Direction::D => (edge_length - 1) - col_idx,
    };

    // find new idxes within the cube
    let new_row = match new_dir {
        Direction::L => (edge_length - 1) - i,
        Direction::R => i,
        Direction::U => edge_length - 1,
        Direction::D => 0,
    };
    let new_col = match new_dir {
        Direction::L => edge_length - 1,
        Direction::R => 0,
        Direction::U => i,
        Direction::D => (edge_length - 1) - i,
    };

    let new_pos = Coord {
        row: cube_row * edge_length + new_row,
        col: cube_col * edge_length + new_col,
    };

    (new_pos, new_dir)
}

pub fn fn1(input: &str) -> i64 {
    let (map, instructions) = parse(input);
    // go to the first open position on the top row (skip the Nones)
    let start_col = map[0].iter().position(|tile| *tile == Tile::Open).unwrap() as i32;

    let mut pos = Coord {
        row: 0,
        col: start_col,
    };
    let mut dir = Direction::R;

    for inst in &instructions {
        match inst {
            Instruction::Rotate(turn) => dir = dir.turn(turn),
            Instruction::Forward(amount) => {
                // take a step "amount" times
                for _ in 0..*amount {
                    let Coord { row: dr, col: dc } = dir.offset();
                    let new_tile = map
                        .get((pos.row + dr) as usize)
                        .and_then(|row| row.get((pos.col + dc) as usize))
                        .unwrap_or(&Tile::None);

                    match new_tile {
                        // if new tile is solid, stop moving
                        Tile::Solid => break,
                        // if new tile is open, move there
                        Tile::Open => {
                            pos = Coord {
                                row: pos.row + dr,
                                col: pos.col + dc,
                            };
                        }
                        // if new tile is not found, wrap around
                        Tile::None => {
                            let new_pos = wrap_2d(&map, &pos, &dir);
                            // if the new_pos is solid, stop moving
                            if map[new_pos.row as usize][new_pos.col as usize] == Tile::Solid {
                                break;
                            }
                            // if the new_pos is open, move there
                            pos = new_pos;
                        }
                    }
                }
            }
        }
    }

    (1000 * (pos.row + 1) + 4 * (pos.col + 1) + dir.score() as i32) as i64
}

pub fn fn2(input: &str, edge_length: i32) -> i64 {
    let (map, instructions) = parse(input);
    // go to the first open position on the top row (skip the Nones)
    let start_col = map[0].iter().position(|tile| *tile == Tile::Open).unwrap() as i32;

    let mut pos = Coord {
        row: 0,
        col: start_col,
    };
    let mut dir = Direction::R;

    for inst in &instructions {
        match inst {
            Instruction::Rotate(turn) => dir = dir.turn(turn),
            Instruction::Forward(amount) => {
                // take a step "amount" times
                for _ in 0..*amount {
                    let Coord { row: dr, col: dc } = dir.offset();
                    let new_tile = map
                        .get((pos.row + dr) as usize)
                        .and_then(|row| row.get((pos.col + dc) as usize))
                        .unwrap_or(&Tile::None);

                    match new_tile {
                        // if new tile is solid, stop moving
                        Tile::Solid => break,
                        // if new tile is open, move there
                        Tile::Open => {
                            pos = Coord {
                                row: pos.row + dr,
                                col: pos.col + dc,
                            };
                        }
                        // if new tile is not found, wrap around
                        Tile::None => {
                            let (new_pos, new_dir) = wrap_3d(&pos, &dir, edge_length);
                            // if the new_pos is solid, stop moving
                            if map[new_pos.row as usize][new_pos.col as usize] == Tile::Solid {
                                break;
                            }
                            // if the new_pos is open, move there
                            pos = new_pos;
                            dir = new_dir
                        }
                    }
                }
            }
        }
    }

    (1000 * (pos.row + 1) + 4 * (pos.col + 1) + dir.score() as i32) as i64
}

#[cfg(test)]
mod tests {
    use super::*;
    use svutils::scaffold_test;

    const YEAR: i16 = 2022;
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
        scaffold_test(YEAR, DAY, "example.txt", "example-spec.2.txt", |input| {
            fn2(input, 4)
        });
    }

    #[test]
    fn test_fn2_input() {
        scaffold_test(YEAR, DAY, "input.txt", "input-spec.2.txt", |input| {
            fn2(input, 50)
        });
    }
}
