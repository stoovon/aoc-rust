extern crate core;

use std::collections::HashMap;

const SCANLINE_LENGTH: usize = 8;
// const FINAL_IMAGE_PIXELS_TEST: usize = 24; // 3 x 8
const FINAL_IMAGE_PIXELS_REAL: usize = 96; // 12 x 8

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Tile {
    id: i64,
    data: [[char; 10]; 10],
    trimmed_data: [[char; 8]; 8],
    sides: [[char; 10]; 4],
    side_hashes: [i64; 4],
    side_scores: [i64; 4],
}

fn hash_side(side: [char; 10]) -> i64 {
    let mut forward_hash = 0;
    for i in 0..10 {
        forward_hash <<= 1;
        if side[i] == '#' {
            forward_hash |= 1;
        }
    }

    let mut reverse_hash = 0;
    for i in 0..10 {
        reverse_hash <<= 1;
        if side[9 - i] == '#' {
            reverse_hash |= 1;
        }
    }

    // Possibly an assumption too far, but let's start with this before implementing anything more complex.
    if forward_hash < reverse_hash {
        forward_hash
    } else {
        reverse_hash
    }
}

impl Tile {
    fn new(id: i64, data: [[char; 10]; 10]) -> Tile {
        let sides = Tile::data_to_sides(&data);
        Tile { 
            id, 
            data,
            trimmed_data: [['x'; 8]; 8],
            sides: sides, 
            side_hashes: Tile::side_hashes(sides),
            side_scores: [0; 4],
        }
    }

    fn trimmed_data(&mut self) -> [[char; 8]; 8] {
        if self.trimmed_data[0][0] != 'x' {
            return self.trimmed_data;
        }

        let mut new_data = [['.'; 8]; 8];
        for row in 1..9 {
            for col in 1..9 {
                new_data[row - 1][col - 1] = self.data[row][col];
            }
        }
        self.trimmed_data = new_data;
        self.trimmed_data
    }

    fn side_hashes(sides: [[char; 10]; 4]) -> [i64; 4] {
        let mut hashes = [0; 4];
        for i in 0..4 {
            hashes[i] = hash_side(sides[i]);
        }
        hashes
    }

    fn set_side_score(&mut self, side: usize, score: i64) {
        self.side_scores[side] = score;
    }

    // URDL as normal
    fn data_to_sides(data: &[[char; 10]; 10]) -> [[char; 10]; 4] {
        let mut sides: [[char; 10]; 4] = [['.'; 10]; 4];

        // Scan across top edge left to right to get top
        sides[0] = Tile::horizontal_scan(data, 0);

        // Scan across right edge top to bottom to get right
        sides[1] = Tile::vertical_scan(data, 9);

        // Scan across bottom edge left to right to get bottom
        sides[2] = Tile::horizontal_scan(data, 9);

        // Scan across left edge top to bottom to get left
        sides[3] = Tile::vertical_scan(data, 0);

        sides
    }

    fn horizontal_scan(data: &[[char; 10]; 10], row: usize) -> [char; 10] {
        let mut result = ['.'; 10];
        for i in 0..10 {
            result[i] = data[row][i];
        }
        result
    }
    
    fn vertical_scan(data: &[[char; 10]; 10], col: usize) -> [char; 10] {
        let mut result = ['.'; 10];
        for i in 0..10 {
            result[i] = data[i][col];
        }
        result
    }

    fn rotate_90_degrees_clockwise(&mut self) {
        let mut new_data = [['.'; 10]; 10];
        for row in 0..10 {
            for col in 0..10 {
                new_data[col][9 - row] = self.data[row][col];
            }
        }
        self.data = new_data;
        self.sides = Tile::data_to_sides(&self.data);
        self.side_hashes = Tile::side_hashes(self.sides);

        // Remember to ratate the scores too
        let mut new_side_scores = [0; 4];
        for side in 0..4 {
            new_side_scores[(side + 1) % 4] = self.side_scores[side];
        }
        self.side_scores = new_side_scores;
    }

    fn flip_horizontal(&mut self) {
        let mut new_data = [['.'; 10]; 10];
        for row in 0..10 {
            for col in 0..10 {
                new_data[row][9 - col] = self.data[row][col];
            }
        }
        self.data = new_data;
        self.sides = Tile::data_to_sides(&self.data);
        self.side_hashes = Tile::side_hashes(self.sides);

        // Remember to flip the scores too
        let mut new_side_scores = [0; 4];
        new_side_scores[0] = self.side_scores[0];
        new_side_scores[1] = self.side_scores[3];
        new_side_scores[2] = self.side_scores[2];
        new_side_scores[3] = self.side_scores[1];
        self.side_scores = new_side_scores;
    }
}

fn parse (input: &str) -> Vec<Tile> {
    let mut tiles = Vec::new();
    let mut id = 0;
    let mut data = [['.'; 10]; 10];
    let mut row = 0;
    for line in input.lines() {
        if line.starts_with("Tile") {
            id = line[5..9].parse::<i64>().unwrap();
        } else if line.is_empty() {
            tiles.push(Tile::new(id, data));
            data = [['.'; 10]; 10];
            row = 0
        } else {
            data[row] = line.chars().collect::<Vec<char>>().try_into().unwrap();
            row += 1;
        }
    }
    tiles.push(Tile::new(id, data));
    tiles
}

fn rotate_image_90_degrees_clockwise(original_image: [[char; FINAL_IMAGE_PIXELS_REAL]; FINAL_IMAGE_PIXELS_REAL]) -> [[char; FINAL_IMAGE_PIXELS_REAL]; FINAL_IMAGE_PIXELS_REAL] {
    let mut new_data = [['.'; FINAL_IMAGE_PIXELS_REAL]; FINAL_IMAGE_PIXELS_REAL];
    for row in 0..FINAL_IMAGE_PIXELS_REAL {
        for col in 0..FINAL_IMAGE_PIXELS_REAL {
            new_data[col][FINAL_IMAGE_PIXELS_REAL - 1 - row] = original_image[row][col];
        }
    }
    new_data
}

fn flip_image_horizontal(original_image: [[char; FINAL_IMAGE_PIXELS_REAL]; FINAL_IMAGE_PIXELS_REAL]) -> [[char; FINAL_IMAGE_PIXELS_REAL]; FINAL_IMAGE_PIXELS_REAL] {
    let mut new_data = [['.'; FINAL_IMAGE_PIXELS_REAL]; FINAL_IMAGE_PIXELS_REAL];
    for row in 0..FINAL_IMAGE_PIXELS_REAL {
        for col in 0..FINAL_IMAGE_PIXELS_REAL {
            new_data[row][FINAL_IMAGE_PIXELS_REAL - 1 - col] = original_image[row][col];
        }
    }
    new_data
}

fn solve(input: &str, grid_size: usize) -> (i64, i64) {
    let part1: i64;
    let part2: i64;

    let tiles = parse(input);
    let sides_to_tiles = tiles.iter().fold(HashMap::new(), |mut acc, tile| {
        for side in 0..4 {
            let hash = tile.side_hashes[side];
            let entry = acc.entry(hash).or_insert(Vec::new());
            entry.push(tile);
        }
        acc
    });

    let mut tiles = tiles.clone();
    for (hash, tile_ids) in sides_to_tiles.iter() {
        // Find each tile which has hash in side_hashes.
        let mut matching_tiles = tiles.iter_mut().filter(|t| t.side_hashes.contains(hash)).collect::<Vec<&mut Tile>>();

        for tile in matching_tiles.iter_mut() {
            for side in 0..4 {
                if tile.side_hashes[side] == *hash {
                    assert!(tile_ids.len() == 1 || tile_ids.len() == 2, "Unexpected number of matches for side hash");
                    tile.set_side_score(side, tile_ids.len() as i64);
                }
            }
        }
    }

    // Since two-way matches score two, and one-way matches score one, then we know that the four corners are the four tiles with score six.
    let corners = tiles.iter().filter(|t| t.side_scores.iter().sum::<i64>() == 6).collect::<Vec<&Tile>>();

    // I'd like to improve on this but it greatly simplifies debugging.
    let sides_to_tiles2 = tiles.iter().fold(HashMap::new(), |mut acc, tile| {
        for side in 0..4 {
            let hash = tile.side_hashes[side];
            let entry = acc.entry(hash).or_insert(Vec::new());
            entry.push(tile);
        }
        acc
    });

    // The answer is the product of the IDs of the four corners.
    part1 = corners.iter().fold(1, |acc, t| acc * t.id);

    // Now we need to assemble the image.
    // We'll start with the top-left corner, and rotate/flip it until the top and left sides are one-way matches.
    let mut top_left_corner = corners[0].clone();
    while top_left_corner.side_scores[0] != 1 || top_left_corner.side_scores[3] != 1 {
        top_left_corner.rotate_90_degrees_clockwise();
        if top_left_corner.side_scores[0] == 1 && top_left_corner.side_scores[3] == 1 {
            break;
        }
        top_left_corner.flip_horizontal();
    }

    // if grid_size == 3 {
    //     part2 = 0;
    //     return (part1, part2)
    //     // // FIXME: Let's revisit this and improve in future.
    //     // let mut final_image = [['.'; FINAL_IMAGE_PIXELS_TEST]; FINAL_IMAGE_PIXELS_TEST];
    // }

    // This is a bit nasty but it'll do.
    // Ideally I'd do default, but ultimately all I care about is the malloc.
    let mut image = [[top_left_corner; 12]; 12];

    // Loop from 0 to grid_size - 1
    for row in 0..grid_size {
        // Loop from 0 to grid_size - 1
        for col in 0..grid_size {
            // If row == 0 and col == 0 then
            if row == 0 && col == 0 {
                // image[0][0] = top_left_corner
                image[row][col] = top_left_corner;
                continue;
            } else if col == 0 {
                // Special case: first column is matched to the bottom of the tile above it.
                let top_tile = image[row - 1][col];
                let top_tile_bottom_side = top_tile.sides[2];
                let top_tile_bottom_hash = top_tile.side_hashes[2];

                // Find the entry in sides_to_tiles which matches top_tile_bottom_hash, and take the tile which isn't equal to top_tile.
                let bottom_tile = sides_to_tiles2[&top_tile_bottom_hash].iter().find(|t| t.id != top_tile.id);
                if bottom_tile.is_none() {
                    panic!("No bottom tile found");
                }

                // // Rotate/flip bottom_tile until bottom_tile_matching_side is on the top.
                let mut bottom_tile_matching_side = bottom_tile.unwrap().sides.iter().position(|s| hash_side(*s) == hash_side(top_tile_bottom_side)).unwrap();
                let mut bottom_tile = (*(bottom_tile.unwrap())).clone(); // Change the type of bottom_tile to Tile

                let mut flips_remaining = 1;
                let mut spins_remaining = 4;
                while bottom_tile_matching_side != 0 {
                    if spins_remaining > 0 {
                        bottom_tile.rotate_90_degrees_clockwise();
                        spins_remaining -= 1;
                    } else {
                        if flips_remaining == 0 {
                            panic!("Infinite loop");
                        }
                        bottom_tile.flip_horizontal();
                        flips_remaining -= 1;
                        spins_remaining = 4;
                    }
                    bottom_tile_matching_side = bottom_tile.sides.iter().position(|s| hash_side(*s) == hash_side(top_tile_bottom_side)).unwrap();
                }

                // There are two other edge cases. 
                let mut horizontal_flip_needed = false;

                if col == 0 {
                    // Firstly, if we're on the first column, then the 1-score edge must be to the left. 
                    if bottom_tile.side_scores[3] != 1 {
                        horizontal_flip_needed = true;
                    }
                } else {
                    // Secondly, if we're on a column after the first, then left edge must match the tile to the left.
                    let left_tile = image[row][col - 1];
                    let left_tile_right_hash = left_tile.side_hashes[1];
                    if left_tile_right_hash != bottom_tile.side_hashes[3] {
                        horizontal_flip_needed = true;
                    }
                }

                if horizontal_flip_needed {
                    bottom_tile.flip_horizontal()
                }

                image[row][col] = bottom_tile;
            } else {
                // Find the tile which matches the right side of image[row][col - 1]
                let left_tile = image[row][col - 1];
                let left_tile_right_side = left_tile.sides[1];
                let left_tile_right_hash = left_tile.side_hashes[1];

                // Find the entry in sides_to_tiles which matches left_tile_right_hash, and take the tile which isn't equal to left_tile.
                let right_tile = sides_to_tiles2[&left_tile_right_hash].iter().find(|t| t.id != left_tile.id);
                if right_tile.is_none() {
                    panic!("No right tile found");
                }

                // Rotate/flip right_tile until right_tile_matching_side is on the left.
                let mut right_tile_matching_side = right_tile.unwrap().sides.iter().position(|s| hash_side(*s) == hash_side(left_tile_right_side)).unwrap();
                let mut right_tile = (*(right_tile.unwrap())).clone();

                let mut flips_remaining = 1;
                let mut spins_remaining = 4;
                while right_tile_matching_side != 3 {
                    if spins_remaining > 0 {
                        right_tile.rotate_90_degrees_clockwise();
                        spins_remaining -= 1;
                    } else {
                        if flips_remaining == 0 {
                            panic!("Infinite loop");
                        }
                        right_tile.flip_horizontal();
                        flips_remaining -= 1;
                        spins_remaining = 4;
                    }
                    right_tile_matching_side = right_tile.sides.iter().position(|s| hash_side(*s) == hash_side(left_tile_right_side)).unwrap();
                }

                // There are two other edge cases. 
                let mut vertical_flip_needed = false;
                
                if row == 0 {
                    // Firstly, if we're on the first row, then the 1-score edge must be above us. 
                    if right_tile.side_scores[0] != 1 {
                        vertical_flip_needed = true;
                    }
                } else {
                    // Secondly, if we're on a row after the first row, then top edge must match the tile above.
                    let top_tile = image[row - 1][col];
                    let top_tile_bottom_hash = top_tile.side_hashes[2];
                    if top_tile_bottom_hash != right_tile.side_hashes[0] {
                        vertical_flip_needed = true;
                    }
                }

                if vertical_flip_needed {
                    // We need to flip vertical (which we can achieve by flipping horizontal and rotating 180 degrees).
                    right_tile.flip_horizontal();
                    right_tile.rotate_90_degrees_clockwise();
                    right_tile.rotate_90_degrees_clockwise();
                }

                image[row][col] = right_tile;
            }
        }
    }

    let mut final_image = [['.'; FINAL_IMAGE_PIXELS_REAL]; FINAL_IMAGE_PIXELS_REAL];

    // Render final image
    for row in 0..grid_size {
        for h_scanline in 0..SCANLINE_LENGTH {
            for col in 0..grid_size {
                for v_scanline in 0..SCANLINE_LENGTH {
                    final_image[row * SCANLINE_LENGTH + h_scanline][col * SCANLINE_LENGTH + v_scanline] = image[row][col].trimmed_data()[h_scanline][v_scanline]
                }
            }
        }
    }

    // Count the number of # in final image
    let mut rough_water = 0;

    for row in 0..FINAL_IMAGE_PIXELS_REAL {
        for col in 0..FINAL_IMAGE_PIXELS_REAL {
            if final_image[row][col] == '#' {
                rough_water += 1;
            }
        }
    }

    let mut monsters_found = 0;

    for orientation in 0..8 {
        // Search for monsters
        for row in 0..FINAL_IMAGE_PIXELS_REAL - 3 {
            for col in 0..FINAL_IMAGE_PIXELS_REAL - 20 {
                if final_image[row][col + 18] == '#' &&
                    final_image[row + 1][col] == '#' &&
                    final_image[row + 1][col + 5] == '#' &&
                    final_image[row + 1][col + 6] == '#' &&
                    final_image[row + 1][col + 11] == '#' &&
                    final_image[row + 1][col + 12] == '#' &&
                    final_image[row + 1][col + 17] == '#' &&
                    final_image[row + 1][col + 18] == '#' &&
                    final_image[row + 1][col + 19] == '#' &&
                    final_image[row + 2][col + 1] == '#' &&
                    final_image[row + 2][col + 4] == '#' &&
                    final_image[row + 2][col + 7] == '#' &&
                    final_image[row + 2][col + 10] == '#' &&
                    final_image[row + 2][col + 13] == '#' &&
                    final_image[row + 2][col + 16] == '#' {
                        monsters_found += 1;
                    }
            }
        }

        if monsters_found > 0 {
            break;
        }

        if orientation == 3 {
            final_image = flip_image_horizontal(final_image);
        } else {
            final_image = rotate_image_90_degrees_clockwise(final_image);
        }
    }

    part2 = rough_water - 15 * monsters_found;

    (part1, part2)
}

pub fn fn1(input: &str, grid_size: usize) -> i64 {
    let (part1, _part2) = solve(input, grid_size);
    part1
}

pub fn fn2(input: &str, grid_size: usize) -> i64 {
    let (_part1, part2) = solve(input, grid_size);
    part2
}

#[cfg(test)]
mod tests {
    use super::*;
    use svutils::scaffold_test;

    const YEAR: i16 = 2020;
    const DAY: i16 = 20;

    #[test]
    fn test_data_to_sides() {
        let tile = Tile::new(3079, [
            "#.#.#####.".chars().collect::<Vec<char>>().try_into().unwrap(),
            ".#..######".chars().collect::<Vec<char>>().try_into().unwrap(),
            "..#.......".chars().collect::<Vec<char>>().try_into().unwrap(),
            "######....".chars().collect::<Vec<char>>().try_into().unwrap(),
            "####.#..#.".chars().collect::<Vec<char>>().try_into().unwrap(),
            ".#...#.##.".chars().collect::<Vec<char>>().try_into().unwrap(),
            "#.#####.##".chars().collect::<Vec<char>>().try_into().unwrap(),
            "..#.###...".chars().collect::<Vec<char>>().try_into().unwrap(),
            "..#.......".chars().collect::<Vec<char>>().try_into().unwrap(),
            "..#.###...".chars().collect::<Vec<char>>().try_into().unwrap(),
        ]);

        // Note: we are always "canonical". This means we scan the same way 
        // so that lefts and rights marry up, and tops and bottoms also.
        // So:
        // - We scan left to right for tops;
        // - Left to right for bottoms;
        // - Top to bottom for lefts; and
        // - Top to bottom for rights.

        assert_eq!(tile.sides[0].iter().collect::<String>(), "#.#.#####."); // UP / TOP
        assert_eq!(tile.sides[1].iter().collect::<String>(), ".#....#..."); // RIGHT
        assert_eq!(tile.sides[2].iter().collect::<String>(), "..#.###..."); // DOWN / BOTTOM
        assert_eq!(tile.sides[3].iter().collect::<String>(), "#..##.#..."); // LEFT
    }

    #[test]
    fn test_rotate_90_degrees_clockwise() {
        let mut tile = Tile::new(3079, [
            "#.#.#####.".chars().collect::<Vec<char>>().try_into().unwrap(),
            ".#..######".chars().collect::<Vec<char>>().try_into().unwrap(),
            "..#.......".chars().collect::<Vec<char>>().try_into().unwrap(),
            "######....".chars().collect::<Vec<char>>().try_into().unwrap(),
            "####.#..#.".chars().collect::<Vec<char>>().try_into().unwrap(),
            ".#...#.##.".chars().collect::<Vec<char>>().try_into().unwrap(),
            "#.#####.##".chars().collect::<Vec<char>>().try_into().unwrap(),
            "..#.###...".chars().collect::<Vec<char>>().try_into().unwrap(),
            "..#.......".chars().collect::<Vec<char>>().try_into().unwrap(),
            "..#.###...".chars().collect::<Vec<char>>().try_into().unwrap(),
        ]);

        // We'll only ever have 1 or 2 in reality but using 3 and 4 for clarity in test.
        tile.set_side_score(0, 1);
        tile.set_side_score(1, 2);
        tile.set_side_score(2,3);
        tile.set_side_score(3,4);

        assert_eq!(tile.sides[0].iter().collect::<String>(), "#.#.#####.");
        assert_eq!(tile.sides[1].iter().collect::<String>(), ".#....#...");
        assert_eq!(tile.sides[2].iter().collect::<String>(), "..#.###...");
        assert_eq!(tile.sides[3].iter().collect::<String>(), "#..##.#...");

        // The values of the hashes and scores stay the same (of course), but the order rotates, which is exactly what we want.
        assert_eq!(tile.side_hashes, [0b0111110101, 0b0001000010, 0b0001110100, 0b0001011001]);
        assert_eq!(tile.side_scores, [1, 2, 3, 4]);

        tile.rotate_90_degrees_clockwise();

        // Remember the canonical order (mind-melting here for simpler matching 
        // logic, which I think is a good trade), so as we rotate 90 things look weird.
        // For example, left is now up. We were reading left top-to-bottom, but we
        // read top left-to-right, so although the content is identical, it's inverted.
        assert_eq!(tile.sides[0].iter().collect::<String>(), "...#.##..#"); 
        assert_eq!(tile.sides[1].iter().collect::<String>(), "#.#.#####.");
        assert_eq!(tile.sides[2].iter().collect::<String>(), "...#....#.");
        assert_eq!(tile.sides[3].iter().collect::<String>(), "..#.###...");
        assert_eq!(tile.side_hashes, [0b0001011001, 0b0111110101, 0b0001000010, 0b0001110100]);
        assert_eq!(tile.side_scores, [4, 1, 2, 3]);

        tile.rotate_90_degrees_clockwise();

        // At 180 things are fully inverted, so everything's backwards.
        assert_eq!(tile.sides[0].iter().collect::<String>(), "...###.#..");
        assert_eq!(tile.sides[1].iter().collect::<String>(), "...#.##..#");
        assert_eq!(tile.sides[2].iter().collect::<String>(), ".#####.#.#");
        assert_eq!(tile.sides[3].iter().collect::<String>(), "...#....#.");
        assert_eq!(tile.side_hashes, [0b0001110100, 0b0001011001, 0b0111110101, 0b0001000010]);
        assert_eq!(tile.side_scores, [3, 4, 1, 2]);

        tile.rotate_90_degrees_clockwise();

        // Also weird... don't panic
        assert_eq!(tile.sides[3].iter().collect::<String>(), ".#####.#.#");
        assert_eq!(tile.sides[0].iter().collect::<String>(), ".#....#...");
        assert_eq!(tile.sides[1].iter().collect::<String>(), "...###.#..");
        assert_eq!(tile.sides[2].iter().collect::<String>(), "#..##.#...");
        assert_eq!(tile.side_hashes, [0b0001000010, 0b0001110100, 0b0001011001, 0b0111110101]);
        assert_eq!(tile.side_scores, [2, 3, 4, 1]);

        tile.rotate_90_degrees_clockwise();

        // Let's validate identity
        assert_eq!(tile.sides[0].iter().collect::<String>(), "#.#.#####.");
        assert_eq!(tile.sides[1].iter().collect::<String>(), ".#....#...");
        assert_eq!(tile.sides[2].iter().collect::<String>(), "..#.###...");
        assert_eq!(tile.sides[3].iter().collect::<String>(), "#..##.#...");
        assert_eq!(tile.side_hashes, [0b0111110101, 0b0001000010, 0b0001110100, 0b0001011001]);
        assert_eq!(tile.side_scores, [1, 2, 3, 4]);
    }

    #[test]
    fn test_flip_horizontal() {
        let mut tile = Tile::new(3079, [
            "#.#.#####.".chars().collect::<Vec<char>>().try_into().unwrap(),
            ".#..######".chars().collect::<Vec<char>>().try_into().unwrap(),
            "..#.......".chars().collect::<Vec<char>>().try_into().unwrap(),
            "######....".chars().collect::<Vec<char>>().try_into().unwrap(),
            "####.#..#.".chars().collect::<Vec<char>>().try_into().unwrap(),
            ".#...#.##.".chars().collect::<Vec<char>>().try_into().unwrap(),
            "#.#####.##".chars().collect::<Vec<char>>().try_into().unwrap(),
            "..#.###...".chars().collect::<Vec<char>>().try_into().unwrap(),
            "..#.......".chars().collect::<Vec<char>>().try_into().unwrap(),
            "..#.###...".chars().collect::<Vec<char>>().try_into().unwrap(),
        ]);

        // We'll only ever have 1 or 2 in reality but using 3 and 4 for clarity in test.
        tile.set_side_score(0, 1);
        tile.set_side_score(1, 2);
        tile.set_side_score(2,3);
        tile.set_side_score(3,4);

        assert_eq!(tile.sides[0].iter().collect::<String>(), "#.#.#####.");
        assert_eq!(tile.sides[1].iter().collect::<String>(), ".#....#...");
        assert_eq!(tile.sides[2].iter().collect::<String>(), "..#.###...");
        assert_eq!(tile.sides[3].iter().collect::<String>(), "#..##.#...");

        assert_eq!(tile.side_hashes, [0b0111110101, 0b0001000010, 0b0001110100, 0b0001011001]);
        assert_eq!(tile.side_scores, [1, 2, 3, 4]);

        tile.flip_horizontal();

        // Remember the canonical order (mind-melting here for simpler matching 
        // logic, which I think is a good trade), so as we flip things look weird
        // (but less weird than rotate).
        // For example, right is now left, but we still read top to bottom.
        // Whereas left is now up. We were reading left top-to-bottom, but we
        // read top left-to-right, so although the content is identical, it's inverted.
        assert_eq!(tile.sides[0].iter().collect::<String>(), ".#####.#.#"); // Reversed and not exchanged
        assert_eq!(tile.sides[1].iter().collect::<String>(), "#..##.#..."); // Not reversed and exchanged
        assert_eq!(tile.sides[2].iter().collect::<String>(), "...###.#.."); // Reversed and not exchanged
        assert_eq!(tile.sides[3].iter().collect::<String>(), ".#....#..."); // Not reversed and exchanged

        // Further mind-twist, the hashes individually stay the same but the order flips about.
        // (This is because the hashes try both ways and take the lowest, which is 
        // a hefty assumption, but we can get fancier if we need to).
        assert_eq!(tile.side_hashes, [0b0111110101, 0b0001011001, 0b0001110100, 0b0001000010]);
        assert_eq!(tile.side_scores, [1, 4, 3, 2]);

        tile.flip_horizontal();

        // Let's validate identity
        assert_eq!(tile.sides[0].iter().collect::<String>(), "#.#.#####.");
        assert_eq!(tile.sides[1].iter().collect::<String>(), ".#....#...");
        assert_eq!(tile.sides[2].iter().collect::<String>(), "..#.###...");
        assert_eq!(tile.sides[3].iter().collect::<String>(), "#..##.#...");
        assert_eq!(tile.side_hashes, [0b0111110101, 0b0001000010, 0b0001110100, 0b0001011001]);
        assert_eq!(tile.side_scores, [1, 2, 3, 4]);
    }

    #[test]
    fn test_fn1_example() {
        scaffold_test(YEAR, DAY, "example.txt", "example-spec.1.txt", | input | { fn1(input, 3) });
    }

    #[test]
    fn test_fn1_input() {
        scaffold_test(YEAR, DAY, "input.txt", "input-spec.1.txt", | input | { fn1(input, 12) });
    }

    // #[test]
    // #[ignore]
    // fn test_fn2_example() {
    //     scaffold_test(YEAR, DAY, "example.txt", "example-spec.2.txt", | input | { fn2(input, 3) });
    // }

    #[test]
    fn test_fn2_input() {
        scaffold_test(YEAR, DAY, "input.txt", "input-spec.2.txt", | input | { fn2(input, 12) });
    }
}
