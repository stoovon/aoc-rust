use crate::SolveError;

pub fn decode(screen: &[impl AsRef<[bool]>]) -> Result<String, SolveError> {
    match screen.len() {
        6 => decode_font(screen, SMALL),
        10 => decode_font(screen, LARGE),
        _ => Err(SolveError::InvalidInput(
            "expected a screen with 6 or 10 rows".into(),
        )),
    }
}

fn decode_font<const WIDTH: usize, const HEIGHT: usize>(
    screen: &[impl AsRef<[bool]>],
    font: &[(char, [&[u8; WIDTH]; HEIGHT])],
) -> Result<String, SolveError> {
    let width = screen[0].as_ref().len();
    if width == 0 || screen.iter().any(|row| row.as_ref().len() != width) {
        return Err(SolveError::InvalidInput(
            "expected a nonempty rectangular screen".into(),
        ));
    }

    let mut answer = String::with_capacity(width.div_ceil(WIDTH));
    for start in (0..width).step_by(WIDTH) {
        let lit = |row: &[bool], offset| row.get(start + offset).copied().unwrap_or(false);
        let letter = font.iter().find(|(_, glyph)| {
            screen.iter().zip(glyph).all(|(row, template)| {
                template
                    .iter()
                    .enumerate()
                    .all(|(offset, &pixel)| lit(row.as_ref(), offset) == (pixel == b'#'))
            })
        });
        let Some(&(letter, _)) = letter else {
            let pixels = screen
                .iter()
                .map(|row| {
                    (0..WIDTH)
                        .map(|offset| if lit(row.as_ref(), offset) { '#' } else { '.' })
                        .collect::<String>()
                })
                .collect::<Vec<_>>()
                .join("\n");
            return Err(SolveError::InvalidInput(format!(
                "unrecognised glyph {}\n{pixels}",
                start / WIDTH + 1
            )));
        };
        answer.push(letter);
    }
    Ok(answer)
}

const SMALL: &[(char, [&[u8; 5]; 6])] = &[
    (
        'A',
        [b".##..", b"#..#.", b"#..#.", b"####.", b"#..#.", b"#..#."],
    ),
    (
        'B',
        [b"###..", b"#..#.", b"###..", b"#..#.", b"#..#.", b"###.."],
    ),
    (
        'C',
        [b".##..", b"#..#.", b"#....", b"#....", b"#..#.", b".##.."],
    ),
    (
        'E',
        [b"####.", b"#....", b"###..", b"#....", b"#....", b"####."],
    ),
    (
        'F',
        [b"####.", b"#....", b"###..", b"#....", b"#....", b"#...."],
    ),
    (
        'G',
        [b".##..", b"#..#.", b"#....", b"#.##.", b"#..#.", b".###."],
    ),
    (
        'H',
        [b"#..#.", b"#..#.", b"####.", b"#..#.", b"#..#.", b"#..#."],
    ),
    (
        'J',
        [b"..##.", b"...#.", b"...#.", b"...#.", b"#..#.", b".##.."],
    ),
    (
        'K',
        [b"#..#.", b"#.#..", b"##...", b"#.#..", b"#.#..", b"#..#."],
    ),
    (
        'L',
        [b"#....", b"#....", b"#....", b"#....", b"#....", b"####."],
    ),
    (
        'O',
        [b".##..", b"#..#.", b"#..#.", b"#..#.", b"#..#.", b".##.."],
    ),
    (
        'P',
        [b"###..", b"#..#.", b"#..#.", b"###..", b"#....", b"#...."],
    ),
    (
        'R',
        [b"###..", b"#..#.", b"#..#.", b"###..", b"#.#..", b"#..#."],
    ),
    (
        'U',
        [b"#..#.", b"#..#.", b"#..#.", b"#..#.", b"#..#.", b".##.."],
    ),
    (
        'Y',
        [b"#...#", b"#...#", b".#.#.", b"..#..", b"..#..", b"..#.."],
    ),
    (
        'Z',
        [b"####.", b"...#.", b"..#..", b".#...", b"#....", b"####."],
    ),
];

const LARGE: &[(char, [&[u8; 8]; 10])] = &[
    (
        'B',
        [
            b"#####...",
            b"#....#..",
            b"#....#..",
            b"#....#..",
            b"#####...",
            b"#....#..",
            b"#....#..",
            b"#....#..",
            b"#....#..",
            b"#####...",
        ],
    ),
    (
        'C',
        [
            b".####...",
            b"#....#..",
            b"#.......",
            b"#.......",
            b"#.......",
            b"#.......",
            b"#.......",
            b"#.......",
            b"#....#..",
            b".####...",
        ],
    ),
    (
        'G',
        [
            b".####...",
            b"#....#..",
            b"#.......",
            b"#.......",
            b"#.......",
            b"#..###..",
            b"#....#..",
            b"#....#..",
            b"#...##..",
            b".###.#..",
        ],
    ),
    (
        'H',
        [
            b"#....#..",
            b"#....#..",
            b"#....#..",
            b"#....#..",
            b"######..",
            b"#....#..",
            b"#....#..",
            b"#....#..",
            b"#....#..",
            b"#....#..",
        ],
    ),
    (
        'J',
        [
            b"...###..",
            b"....#...",
            b"....#...",
            b"....#...",
            b"....#...",
            b"....#...",
            b"....#...",
            b"#...#...",
            b"#...#...",
            b".###....",
        ],
    ),
    (
        'L',
        [
            b"#.......",
            b"#.......",
            b"#.......",
            b"#.......",
            b"#.......",
            b"#.......",
            b"#.......",
            b"#.......",
            b"#.......",
            b"######..",
        ],
    ),
    (
        'N',
        [
            b"#....#..",
            b"##...#..",
            b"##...#..",
            b"#.#..#..",
            b"#.#..#..",
            b"#..#.#..",
            b"#..#.#..",
            b"#...##..",
            b"#...##..",
            b"#....#..",
        ],
    ),
    (
        'P',
        [
            b"#####...",
            b"#....#..",
            b"#....#..",
            b"#....#..",
            b"#####...",
            b"#.......",
            b"#.......",
            b"#.......",
            b"#.......",
            b"#.......",
        ],
    ),
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decodes_both_fonts_with_trailing_padding_omitted() {
        let small = [".##", "#..#", "#..#", "####", "#..#", "#..#"];
        let small: Vec<_> = small
            .iter()
            .map(|row| {
                (0..4)
                    .map(|index| row.as_bytes().get(index) == Some(&b'#'))
                    .collect::<Vec<_>>()
            })
            .collect();
        assert_eq!(decode(&small).unwrap(), "A");

        let mut large = [[true, false, false, false, false, false]; 10];
        large[9].fill(true);
        assert_eq!(decode(&large).unwrap(), "L");
    }

    #[test]
    fn rejects_unsupported_height_and_ragged_rows() {
        assert!(decode(&[[false; 3]; 5]).is_err());
        let mut rows = vec![vec![false; 3]; 6];
        rows[1].pop();
        assert!(decode(&rows).is_err());
    }
}
