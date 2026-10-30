use std::collections::{HashMap, HashSet};
use svutils::SolveError;

const SCANLINE_LENGTH: usize = 8;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Tile {
    id: i64,
    data: [[char; 10]; 10],
    sides: [[char; 10]; 4],
    side_hashes: [i64; 4],
    matched_sides: [bool; 4],
}

// Canonical border encoding identifies candidates; oriented borders still need exact equality.
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

    forward_hash.min(reverse_hash)
}

impl Tile {
    fn new(id: i64, data: [[char; 10]; 10]) -> Tile {
        let sides = Tile::data_to_sides(&data);
        Tile {
            id,
            data,
            sides,
            side_hashes: Tile::side_hashes(sides),
            matched_sides: [false; 4],
        }
    }

    fn is_corner(&self) -> bool {
        let unmatched = self
            .matched_sides
            .iter()
            .filter(|&&matched| !matched)
            .count();
        unmatched == 2
            && (0..4).any(|side| !self.matched_sides[side] && !self.matched_sides[(side + 1) % 4])
    }

    fn trimmed_data(&self) -> [[char; 8]; 8] {
        let mut new_data = [['.'; 8]; 8];
        for row in 1..9 {
            for col in 1..9 {
                new_data[row - 1][col - 1] = self.data[row][col];
            }
        }
        new_data
    }

    fn side_hashes(sides: [[char; 10]; 4]) -> [i64; 4] {
        let mut hashes = [0; 4];
        for i in 0..4 {
            hashes[i] = hash_side(sides[i]);
        }
        hashes
    }

    // Top/bottom run left to right; right/left run top to bottom.
    fn data_to_sides(data: &[[char; 10]; 10]) -> [[char; 10]; 4] {
        let mut sides: [[char; 10]; 4] = [['.'; 10]; 4];

        sides[0] = Tile::horizontal_scan(data, 0);

        sides[1] = Tile::vertical_scan(data, 9);

        sides[2] = Tile::horizontal_scan(data, 9);

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

        let mut new_matched_sides = [false; 4];
        for side in 0..4 {
            new_matched_sides[(side + 1) % 4] = self.matched_sides[side];
        }
        self.matched_sides = new_matched_sides;
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

        let mut new_matched_sides = [false; 4];
        new_matched_sides[0] = self.matched_sides[0];
        new_matched_sides[1] = self.matched_sides[3];
        new_matched_sides[2] = self.matched_sides[2];
        new_matched_sides[3] = self.matched_sides[1];
        self.matched_sides = new_matched_sides;
    }
}

fn parse(input: &str) -> Result<(Vec<Tile>, usize), SolveError> {
    let mut lines = input.lines().enumerate().peekable();
    let mut tiles = Vec::new();
    let mut ids = HashSet::new();
    while let Some((line_index, header)) = lines.next() {
        if header.is_empty() {
            continue;
        }
        let invalid =
            |message: &str| SolveError::InvalidInput(message.into()).at_line(line_index + 1);
        let id = header
            .strip_prefix("Tile ")
            .and_then(|rest| rest.strip_suffix(':'))
            .filter(|id| !id.is_empty() && id.bytes().all(|byte| byte.is_ascii_digit()))
            .ok_or_else(|| invalid("expected 'Tile <positive ID>:'"))?;
        let id: i64 = id.parse().map_err(|error: std::num::ParseIntError| {
            SolveError::from(error).at_line(line_index + 1)
        })?;
        if id == 0 || !ids.insert(id) {
            return Err(invalid("tile IDs must be positive and unique"));
        }
        let mut data = [['.'; 10]; 10];
        for row in &mut data {
            let (index, line) = lines
                .next()
                .ok_or_else(|| invalid("tile must contain ten rows"))?;
            if line.len() != 10 || !line.bytes().all(|byte| matches!(byte, b'.' | b'#')) {
                return Err(SolveError::InvalidInput(
                    "tile rows must contain ten '.' or '#' cells".into(),
                )
                .at_line(index + 1));
            }
            for (cell, byte) in row.iter_mut().zip(line.bytes()) {
                *cell = char::from(byte);
            }
        }
        if let Some(&(index, line)) = lines.peek() {
            if !line.is_empty() {
                return Err(
                    SolveError::InvalidInput("expected a blank line between tiles".into())
                        .at_line(index + 1),
                );
            }
        }
        tiles.push(Tile::new(id, data));
    }
    if tiles.is_empty() {
        return Err(SolveError::EmptyInput);
    }
    let mut grid_size = 1usize;
    while grid_size <= tiles.len() / grid_size {
        if grid_size * grid_size == tiles.len() {
            return Ok((tiles, grid_size));
        }
        grid_size += 1;
    }
    Err(SolveError::InvalidInput(
        "tile count must be a perfect square".into(),
    ))
}

#[cfg(test)]
fn rotate_image_90_degrees_clockwise(original_image: Vec<Vec<char>>) -> Vec<Vec<char>> {
    let size = original_image.len();
    let mut result = vec![vec!['.'; size]; size];
    for row in 0..size {
        for col in 0..size {
            result[col][size - 1 - row] = original_image[row][col];
        }
    }
    result
}

#[cfg(test)]
fn flip_image_horizontal(mut image: Vec<Vec<char>>) -> Vec<Vec<char>> {
    for row in &mut image {
        row.reverse();
    }
    image
}

struct Jigsaw {
    tiles: Vec<Tile>,
    grid_size: usize,
    borders: HashMap<i64, Vec<usize>>,
}

impl Jigsaw {
    fn parse(input: &str) -> Result<Self, SolveError> {
        let (mut tiles, grid_size) = parse(input)?;
        let borders = mark_matching_sides(&mut tiles);
        Ok(Self {
            tiles,
            grid_size,
            borders,
        })
    }

    fn corners(&self) -> impl Iterator<Item = &Tile> {
        self.tiles.iter().filter(|tile| tile.is_corner())
    }
}

fn mark_matching_sides(tiles: &mut [Tile]) -> HashMap<i64, Vec<usize>> {
    let mut borders: HashMap<i64, Vec<usize>> = HashMap::new();
    for (index, tile) in tiles.iter().enumerate() {
        for &hash in &tile.side_hashes {
            borders.entry(hash).or_default().push(index);
        }
    }
    for (index, tile) in tiles.iter_mut().enumerate() {
        for side in 0..4 {
            tile.matched_sides[side] = borders[&tile.side_hashes[side]]
                .iter()
                .any(|&other| other != index);
        }
    }
    borders
}

fn orientations(tile: Tile) -> Vec<Tile> {
    let mut result = Vec::new();
    for flipped in [false, true] {
        let mut oriented = tile;
        if flipped {
            oriented.flip_horizontal();
        }
        for _ in 0..4 {
            if !result
                .iter()
                .any(|other: &Tile| other.data == oriented.data)
            {
                result.push(oriented);
            }
            oriented.rotate_90_degrees_clockwise();
        }
    }
    result
}

const ASSEMBLY_WORK_LIMIT: usize = 1_000_000;

fn spend_assembly_work(remaining: &mut usize) -> Result<(), SolveError> {
    *remaining = remaining
        .checked_sub(1)
        .ok_or(SolveError::ResourceLimit("assembly search limit exceeded"))?;
    Ok(())
}

fn placement_candidates(
    jigsaw: &Jigsaw,
    variants: &[Vec<Tile>],
    placed: &[Tile],
    used: &[bool],
    work_remaining: &mut usize,
) -> Result<Vec<(usize, usize)>, SolveError> {
    let row = placed.len() / jigsaw.grid_size;
    let col = placed.len() % jigsaw.grid_size;
    let above = (row > 0).then(|| &placed[placed.len() - jigsaw.grid_size]);
    let left = (col > 0).then(|| &placed[placed.len() - 1]);
    let indices: Vec<usize> = if let Some(tile) = left {
        jigsaw.borders[&tile.side_hashes[1]].clone()
    } else if let Some(tile) = above {
        jigsaw.borders[&tile.side_hashes[2]].clone()
    } else {
        let corners: Vec<_> = jigsaw
            .tiles
            .iter()
            .enumerate()
            .filter(|(_, tile)| tile.is_corner())
            .map(|(index, _)| index)
            .collect();
        if corners.len() == 4 {
            // Four forced corners occupy all four corner positions. Global rotation
            // can put this one at the top left without excluding any arrangement.
            vec![corners[0]]
        } else {
            (0..jigsaw.tiles.len()).collect()
        }
    };
    let mut candidates = Vec::new();
    let mut seen = vec![false; jigsaw.tiles.len()];
    for index in indices {
        spend_assembly_work(work_remaining)?;
        if used[index] || seen[index] {
            continue;
        }
        seen[index] = true;
        for (orientation, tile) in variants[index].iter().enumerate() {
            spend_assembly_work(work_remaining)?;
            if above.is_some_and(|above| above.sides[2] != tile.sides[0])
                || left.is_some_and(|left| left.sides[1] != tile.sides[3])
            {
                continue;
            }
            // An unmatched border cannot face another tile.
            if (row > 0 && !tile.matched_sides[0])
                || (col + 1 < jigsaw.grid_size && !tile.matched_sides[1])
                || (row + 1 < jigsaw.grid_size && !tile.matched_sides[2])
                || (col > 0 && !tile.matched_sides[3])
            {
                continue;
            }
            candidates.push((index, orientation));
        }
    }
    candidates.reverse();
    Ok(candidates)
}

fn assemble(jigsaw: &Jigsaw) -> Result<Vec<Vec<Tile>>, SolveError> {
    assemble_with_limit(jigsaw, ASSEMBLY_WORK_LIMIT)
}

fn assemble_with_limit(
    jigsaw: &Jigsaw,
    mut work_remaining: usize,
) -> Result<Vec<Vec<Tile>>, SolveError> {
    // Bound orientation storage and per-depth bookkeeping before allocating them.
    if jigsaw.tiles.len() > 1024 {
        return Err(SolveError::ResourceLimit(
            "assembly supports at most 1024 tiles",
        ));
    }
    let variants: Vec<_> = jigsaw.tiles.iter().copied().map(orientations).collect();
    let mut used = vec![false; jigsaw.tiles.len()];
    let mut placed = Vec::new();
    let mut placed_indices = Vec::new();
    // Each depth retains its untried choices while deeper placements are explored.
    let mut choices = vec![placement_candidates(
        jigsaw,
        &variants,
        &placed,
        &used,
        &mut work_remaining,
    )?];
    while let Some(remaining) = choices.last_mut() {
        if let Some((index, orientation)) = remaining.pop() {
            used[index] = true;
            placed.push(variants[index][orientation]);
            placed_indices.push(index);
            if placed.len() == jigsaw.tiles.len() {
                return Ok(placed
                    .chunks(jigsaw.grid_size)
                    .map(|row| row.to_vec())
                    .collect());
            }
            choices.push(placement_candidates(
                jigsaw,
                &variants,
                &placed,
                &used,
                &mut work_remaining,
            )?);
        } else {
            choices.pop();
            if let Some(index) = placed_indices.pop() {
                used[index] = false;
                placed.pop();
            }
        }
    }
    Err(SolveError::InvalidInput(
        "tiles cannot form a square image".into(),
    ))
}

fn render(image: &[Vec<Tile>]) -> Result<Vec<Vec<char>>, SolveError> {
    let grid_size = image.len();
    let image_size = grid_size
        .checked_mul(SCANLINE_LENGTH)
        .ok_or(SolveError::ArithmeticOverflow)?;
    let mut final_image = vec![vec!['.'; image_size]; image_size];

    for row in 0..grid_size {
        for col in 0..grid_size {
            let interior = image[row][col].trimmed_data();
            for h_scanline in 0..SCANLINE_LENGTH {
                for v_scanline in 0..SCANLINE_LENGTH {
                    final_image[row * SCANLINE_LENGTH + h_scanline]
                        [col * SCANLINE_LENGTH + v_scanline] = interior[h_scanline][v_scanline];
                }
            }
        }
    }

    Ok(final_image)
}

const MONSTER_PIXELS: [(usize, usize); 15] = [
    (0, 18),
    (1, 0),
    (1, 5),
    (1, 6),
    (1, 11),
    (1, 12),
    (1, 17),
    (1, 18),
    (1, 19),
    (2, 1),
    (2, 4),
    (2, 7),
    (2, 10),
    (2, 13),
    (2, 16),
];

#[derive(Clone, Copy)]
struct Monster {
    pixels: [(usize, usize); 15],
    height: usize,
    width: usize,
}

impl Monster {
    fn orientations() -> [Self; 8] {
        let original = Self {
            pixels: MONSTER_PIXELS,
            height: 3,
            width: 20,
        };
        let mut result = [original; 8];
        for flipped in 0..2 {
            let mut monster = original;
            if flipped == 1 {
                for (_, col) in &mut monster.pixels {
                    *col = monster.width - 1 - *col;
                }
            }
            for rotation in 0..4 {
                result[flipped * 4 + rotation] = monster;
                for (row, col) in &mut monster.pixels {
                    (*row, *col) = (*col, monster.height - 1 - *row);
                }
                std::mem::swap(&mut monster.height, &mut monster.width);
            }
        }
        result
    }

    fn coverage(&self, image: &[Vec<char>], covered: &mut [Vec<bool>]) -> usize {
        let height = image.len();
        let width = image.first().map_or(0, Vec::len);
        if height < self.height || width < self.width {
            return 0;
        }
        for row in covered.iter_mut() {
            row.fill(false);
        }
        for row in 0..=height - self.height {
            for col in 0..=width - self.width {
                if self
                    .pixels
                    .iter()
                    .all(|&(dr, dc)| image[row + dr][col + dc] == '#')
                {
                    for &(dr, dc) in &self.pixels {
                        covered[row + dr][col + dc] = true;
                    }
                }
            }
        }
        covered.iter().flatten().filter(|&&pixel| pixel).count()
    }
}

fn water_roughness(image: Vec<Vec<char>>) -> usize {
    let total = image
        .iter()
        .flatten()
        .filter(|&&pixel| pixel == '#')
        .count();
    let mut covered: Vec<_> = image.iter().map(|row| vec![false; row.len()]).collect();
    let most_covered = Monster::orientations()
        .iter()
        .map(|monster| monster.coverage(&image, &mut covered))
        .max()
        .unwrap_or(0);
    total - most_covered
}

pub fn part1(input: &str) -> Result<String, SolveError> {
    let jigsaw = Jigsaw::parse(input)?;
    let mut product = 1i64;
    let mut count = 0;
    for corner in jigsaw.corners() {
        product = product
            .checked_mul(corner.id)
            .ok_or(SolveError::ArithmeticOverflow)?;
        count += 1;
    }
    if count != 4 {
        return Err(SolveError::InvalidInput(
            "expected exactly four corner tiles".into(),
        ));
    }
    Ok(product.to_string())
}

pub fn part2(input: &str) -> Result<String, SolveError> {
    let jigsaw = Jigsaw::parse(input)?;
    let image = assemble(&jigsaw)?;
    Ok(water_roughness(render(&image)?).to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use svutils::scaffold_test_result;

    const YEAR: i16 = 2020;
    const DAY: i16 = 20;

    fn synthetic_tile() -> Tile {
        let mut data = [['.'; 10]; 10];
        for (row, col) in [(0, 1), (0, 4), (2, 9), (9, 6), (5, 0), (3, 5)] {
            data[row][col] = '#';
        }
        Tile::new(41, data)
    }

    #[test]
    fn test_data_to_sides() {
        let tile = synthetic_tile();
        let expected = [".#..#.....", "..#.......", "......#...", ".....#...."];
        for (side, expected) in tile.sides.iter().zip(expected) {
            assert_eq!(side.iter().collect::<String>(), expected);
        }
        assert_eq!(tile.side_hashes, [18, 4, 8, 16]);
    }

    #[test]
    fn test_rotate_90_degrees_clockwise() {
        let mut tile = synthetic_tile();
        for side in 0..4 {
            tile.matched_sides[side] = side == 0 || side == 1;
        }
        let original = tile;
        tile.rotate_90_degrees_clockwise();
        let expected = ["....#.....", ".#..#.....", ".......#..", "......#..."];
        for (side, expected) in tile.sides.iter().zip(expected) {
            assert_eq!(side.iter().collect::<String>(), expected);
        }
        assert_eq!(tile.data[5][6], '#');
        assert_eq!(tile.side_hashes, [16, 18, 4, 8]);
        assert_eq!(tile.matched_sides, [false, true, true, false]);
        for _ in 0..3 {
            tile.rotate_90_degrees_clockwise();
        }
        assert_eq!(tile.data, original.data);
        assert_eq!(tile.side_hashes, original.side_hashes);
        assert_eq!(tile.matched_sides, original.matched_sides);
    }

    #[test]
    fn test_flip_horizontal() {
        let mut tile = synthetic_tile();
        for side in 0..4 {
            tile.matched_sides[side] = side == 0 || side == 1;
        }
        let original = tile;
        tile.flip_horizontal();
        let expected = [".....#..#.", ".....#....", "...#......", "..#......."];
        for (side, expected) in tile.sides.iter().zip(expected) {
            assert_eq!(side.iter().collect::<String>(), expected);
        }
        assert_eq!(tile.data[3][4], '#');
        assert_eq!(tile.side_hashes, [18, 16, 8, 4]);
        assert_eq!(tile.matched_sides, [true, false, false, true]);
        tile.flip_horizontal();
        assert_eq!(tile.data, original.data);
        assert_eq!(tile.side_hashes, original.side_hashes);
        assert_eq!(tile.matched_sides, original.matched_sides);
    }

    #[test]
    fn test_interior_tracks_transformations() {
        let mut tile = synthetic_tile();
        let mut expected = [['.'; 8]; 8];
        expected[2][4] = '#';
        assert_eq!(tile.trimmed_data(), expected);

        tile.rotate_90_degrees_clockwise();
        expected[2][4] = '.';
        expected[4][5] = '#';
        assert_eq!(tile.trimmed_data(), expected);

        tile.flip_horizontal();
        expected[4][5] = '.';
        expected[4][2] = '#';
        assert_eq!(tile.trimmed_data(), expected);
    }

    #[test]
    fn test_repeated_edges_require_another_tile() {
        let mut tiles = [Tile::new(11, [['.'; 10]; 10])];
        let borders = mark_matching_sides(&mut tiles);
        assert_eq!(borders[&0], [0, 0, 0, 0]);
        assert_eq!(tiles[0].matched_sides, [false; 4]);

        let mut tiles = [tiles[0], Tile::new(23, [['.'; 10]; 10])];
        mark_matching_sides(&mut tiles);
        assert!(tiles.iter().all(|tile| tile.matched_sides == [true; 4]));
    }

    #[test]
    fn test_assembly_uses_each_tile_once_and_checks_both_neighbours() {
        let mut tiles = Vec::new();
        // Shared edges are generated from a larger pixel grid, independently of
        // the tile transformation and assembly implementations.
        let mut pixels = [[false; 19]; 19];
        let mut state = 71u32;
        for row in &mut pixels {
            for pixel in row {
                state = state.wrapping_mul(1664525).wrapping_add(1013904223);
                *pixel = state & (1 << 30) != 0;
            }
        }
        for (id, row, col) in [(17, 1, 1), (29, 0, 1), (43, 1, 0), (61, 0, 0)] {
            let mut data = [['.'; 10]; 10];
            for r in 0..10 {
                for c in 0..10 {
                    if pixels[row * 9 + r][col * 9 + c] {
                        data[r][c] = '#';
                    }
                }
            }
            let mut tile = Tile::new(id, data);
            tile.rotate_90_degrees_clockwise();
            tile.flip_horizontal();
            tiles.push(tile);
        }
        let borders = mark_matching_sides(&mut tiles);
        let jigsaw = Jigsaw {
            tiles,
            grid_size: 2,
            borders,
        };
        let corners: Vec<_> = jigsaw
            .tiles
            .iter()
            .enumerate()
            .filter(|(_, tile)| tile.is_corner())
            .map(|(index, _)| index)
            .collect();
        assert_eq!(corners.len(), 4);
        let variants: Vec<_> = jigsaw.tiles.iter().copied().map(orientations).collect();
        let mut work = ASSEMBLY_WORK_LIMIT;
        let candidates =
            placement_candidates(&jigsaw, &variants, &[], &[false; 4], &mut work).unwrap();
        assert_eq!(candidates.len(), 2);
        assert!(candidates.iter().all(|&(index, orientation)| {
            index == corners[0]
                && !variants[index][orientation].matched_sides[0]
                && !variants[index][orientation].matched_sides[3]
        }));
        let image = assemble(&jigsaw).unwrap();
        let ids: HashSet<_> = image.iter().flatten().map(|tile| tile.id).collect();
        assert_eq!(ids.len(), 4);
        for row in 0..2 {
            assert_eq!(image[row][0].sides[1], image[row][1].sides[3]);
        }
        for col in 0..2 {
            assert_eq!(image[0][col].sides[2], image[1][col].sides[0]);
        }

        let mut tiles: Vec<_> = (1..=4).map(|id| Tile::new(id, [['.'; 10]; 10])).collect();
        let borders = mark_matching_sides(&mut tiles);
        let image = assemble(&Jigsaw {
            tiles,
            grid_size: 2,
            borders,
        })
        .unwrap();
        assert_eq!(
            image
                .iter()
                .flatten()
                .map(|tile| tile.id)
                .collect::<HashSet<_>>()
                .len(),
            4
        );
    }

    #[test]
    fn test_assembly_rejects_impossible_joins() {
        let mut tiles = vec![Tile::new(1, [['.'; 10]; 10]), Tile::new(2, [['#'; 10]; 10])];
        tiles.extend([synthetic_tile(), Tile::new(3, [['.'; 10]; 10])]);
        let borders = mark_matching_sides(&mut tiles);
        assert!(assemble(&Jigsaw {
            tiles,
            grid_size: 2,
            borders
        })
        .is_err());
        assert_eq!(orientations(synthetic_tile()).len(), 8);
        assert_eq!(orientations(Tile::new(1, [['.'; 10]; 10])).len(), 1);
    }

    #[test]
    fn test_monster_search_includes_last_valid_position() {
        let points = [
            (0, 18),
            (1, 0),
            (1, 5),
            (1, 6),
            (1, 11),
            (1, 12),
            (1, 17),
            (1, 18),
            (1, 19),
            (2, 1),
            (2, 4),
            (2, 7),
            (2, 10),
            (2, 13),
            (2, 16),
        ];
        for (size, top, left) in [(20, 17, 0), (23, 20, 3)] {
            let mut image = vec![vec!['.'; size]; size];
            for (row, col) in points {
                image[top + row][left + col] = '#';
            }
            image[1][1] = '#';
            assert_eq!(water_roughness(image), 1);
        }
        let mut small = vec![vec!['.'; 2]; 2];
        small[0][0] = '#';
        assert_eq!(water_roughness(small), 1);
    }

    #[test]
    fn test_overlapping_monsters_count_shared_pixels_once() {
        let points = [
            (0, 18),
            (1, 0),
            (1, 5),
            (1, 6),
            (1, 11),
            (1, 12),
            (1, 17),
            (1, 18),
            (1, 19),
            (2, 1),
            (2, 4),
            (2, 7),
            (2, 10),
            (2, 13),
            (2, 16),
        ];
        let mut image = vec![vec!['.'; 21]; 21];
        for offset in [0, 1] {
            for (row, col) in points {
                image[row + 7][col + offset] = '#';
            }
        }
        image[0][0] = '#';
        assert_eq!(water_roughness(image), 1);
    }

    #[test]
    fn test_monster_orientations_and_greatest_coverage() {
        let variants = Monster::orientations();
        let shapes: HashSet<_> = variants
            .iter()
            .map(|monster| {
                let mut pixels = monster.pixels;
                pixels.sort_unstable();
                pixels
            })
            .collect();
        assert_eq!(shapes.len(), 8);
        for monster in variants {
            let mut image = vec![vec!['.'; 40]; 40];
            for (row, col) in monster.pixels {
                image[row + 5][col + 7] = '#';
            }
            assert_eq!(water_roughness(image), 0);
        }

        let mut image = vec![vec!['.'; 64]; 64];
        for (row, col) in MONSTER_PIXELS {
            image[row][col] = '#';
        }
        for offset in [10, 35] {
            for (row, col) in variants[1].pixels {
                image[row + offset][col + 40] = '#';
            }
        }
        // One original monster versus two rotated monsters: keep orientations separate.
        assert_eq!(water_roughness(image.clone()), 15);

        let original = Monster {
            pixels: MONSTER_PIXELS,
            height: 3,
            width: 20,
        };
        let mut most = 0;
        for _ in 0..2 {
            for _ in 0..4 {
                let mut mask = vec![vec![false; 64]; 64];
                most = most.max(original.coverage(&image, &mut mask));
                image = rotate_image_90_degrees_clockwise(image);
            }
            image = flip_image_horizontal(image);
        }
        assert_eq!(most, 30);
    }

    #[test]
    fn test_assembly_limit_is_not_no_solution() {
        let mut tiles = vec![Tile::new(1, [['.'; 10]; 10])];
        let borders = mark_matching_sides(&mut tiles);
        let jigsaw = Jigsaw {
            tiles,
            grid_size: 1,
            borders,
        };
        assert!(matches!(
            assemble_with_limit(&jigsaw, 0),
            Err(SolveError::ResourceLimit(_))
        ));
        assert!(matches!(
            assemble_with_limit(&jigsaw, 1),
            Err(SolveError::ResourceLimit(_))
        ));
        assert!(assemble_with_limit(&jigsaw, 2).is_ok());
    }

    #[test]
    fn test_assembly_undoes_a_locally_valid_dead_end() {
        let labels = [[4u8, 2, 1, 4], [4, 1, 4, 1], [4, 4, 1, 1], [3, 2, 2, 1]];
        let mut tiles = Vec::new();
        for (index, edges) in labels.iter().enumerate() {
            let mut data = [['.'; 10]; 10];
            for (side, label) in edges.iter().enumerate() {
                for bit in 0..8 {
                    if label & (1 << bit) != 0 {
                        let (row, col) = match side {
                            0 => (0, bit + 1),
                            1 => (bit + 1, 9),
                            2 => (9, bit + 1),
                            _ => (bit + 1, 0),
                        };
                        data[row][col] = '#';
                    }
                }
            }
            data[index + 1][index + 1] = '#';
            tiles.push(Tile::new(index as i64 + 1, data));
        }
        let borders = mark_matching_sides(&mut tiles);
        let jigsaw = Jigsaw {
            tiles,
            grid_size: 2,
            borders,
        };
        let variants: Vec<_> = jigsaw.tiles.iter().copied().map(orientations).collect();
        let mut work = ASSEMBLY_WORK_LIMIT;
        let first = placement_candidates(&jigsaw, &variants, &[], &[false; 4], &mut work)
            .unwrap()
            .pop()
            .unwrap();
        assert_eq!(first, (0, 0));

        // Exhaustively try the remaining pieces using only direct edge comparisons.
        // The first locally valid root placement has no complete arrangement.
        fn can_complete(variants: &[Vec<Tile>], placed: &mut Vec<Tile>, used: u8) -> bool {
            if placed.len() == 4 {
                return true;
            }
            let position = placed.len();
            for index in 0..4 {
                if used & (1 << index) != 0 {
                    continue;
                }
                for &tile in &variants[index] {
                    if (position >= 2 && placed[position - 2].sides[2] != tile.sides[0])
                        || (position % 2 == 1 && placed[position - 1].sides[1] != tile.sides[3])
                    {
                        continue;
                    }
                    placed.push(tile);
                    let complete = can_complete(variants, placed, used | (1 << index));
                    placed.pop();
                    if complete {
                        return true;
                    }
                }
            }
            false
        }
        assert!(!can_complete(&variants, &mut vec![variants[0][0]], 1));
        let image = assemble(&jigsaw).unwrap();
        assert_ne!(image[0][0].data, variants[0][0].data);
        assert_eq!(
            image
                .iter()
                .flatten()
                .map(|tile| tile.id)
                .collect::<HashSet<_>>()
                .len(),
            4
        );
        for index in 0..2 {
            assert_eq!(image[index][0].sides[1], image[index][1].sides[3]);
            assert_eq!(image[0][index].sides[2], image[1][index].sides[0]);
        }
    }

    #[test]
    fn test_corners_require_exactly_two_adjacent_unmatched_sides() {
        for mask in 0u8..16 {
            let mut tile = synthetic_tile();
            for side in 0..4 {
                tile.matched_sides[side] = mask & (1 << side) != 0;
            }
            let expected = matches!(mask, 0b0011 | 0b0110 | 0b1100 | 0b1001);
            for _ in 0..4 {
                assert_eq!(tile.is_corner(), expected);
                tile.rotate_90_degrees_clockwise();
            }
            tile.flip_horizontal();
            assert_eq!(tile.is_corner(), expected);
        }
    }

    #[test]
    fn test_parser() {
        let tile = |id| format!("Tile {id}:\n{}", "..........\n".repeat(10));
        let input = tile(12345);
        let (tiles, size) = parse(&input).unwrap();
        assert_eq!(size, 1);
        assert_eq!(tiles[0].id, 12345);
        assert!(parse(&input.replace('\n', "\r\n")).is_ok());
        let four = [tile(1), tile(2), tile(3), tile(4)].join("\n");
        assert_eq!(parse(&four).unwrap().1, 2);
        for invalid in [
            String::new(),
            input.replace("12345", "0"),
            input.replace("12345", "-1"),
            input.replace("12345", "1x"),
            input.replace("Tile ", "Tiles "),
            input.replace(':', ""),
            input.replace("..........", "........."),
            input.replacen('.', "é", 1),
            input.replacen('.', "x", 1),
            "Tile 1:\n..........".into(),
            format!("{}\n{}", tile(1), tile(1)),
            format!("{}\n{}", tile(1), tile(2)),
            format!("{}..........\n", tile(1)),
        ] {
            assert!(parse(&invalid).is_err(), "accepted malformed tile data");
        }
    }

    #[test]
    fn test_part1_input() {
        scaffold_test_result(YEAR, DAY, "input.txt", "input-spec.1.txt", part1);
    }

    #[test]
    fn test_part2_input() {
        scaffold_test_result(YEAR, DAY, "input.txt", "input-spec.2.txt", part2);
    }
}
