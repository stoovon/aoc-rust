use regex::Regex;
use svutils::SolveError;

const WIDTH: usize = 50;
const HEIGHT: usize = 6;

fn build_screen(input: &str) -> Result<[[bool; WIDTH]; HEIGHT], SolveError> {
    if input.is_empty() {
        return Err(SolveError::EmptyInput);
    }

    let rectangle = Regex::new(r"\Arect ([0-9]+)x([0-9]+)\z").unwrap();
    let rotation = Regex::new(r"\Arotate (row y|column x)=([0-9]+) by ([0-9]+)\z").unwrap();
    let mut screen = [[false; WIDTH]; HEIGHT];

    for (index, line) in input.lines().enumerate() {
        let invalid = |message: &str| SolveError::InvalidInput(message.into()).at_line(index + 1);
        let number = |text: &str| {
            text.parse::<usize>()
                .map_err(|error| SolveError::from(error).at_line(index + 1))
        };
        if let Some(captures) = rectangle.captures(line) {
            let width = number(&captures[1])?;
            let height = number(&captures[2])?;
            if width > WIDTH || height > HEIGHT {
                return Err(invalid("rectangle exceeds screen dimensions"));
            }
            for row in &mut screen[..height] {
                row[..width].fill(true);
            }
        } else if let Some(captures) = rotation.captures(line) {
            let coordinate = number(&captures[2])?;
            let shift = number(&captures[3])?;
            if &captures[1] == "row y" {
                let row = screen
                    .get_mut(coordinate)
                    .ok_or_else(|| invalid("row outside screen"))?;
                row.rotate_right(shift % WIDTH);
            } else {
                if coordinate >= WIDTH {
                    return Err(invalid("column outside screen"));
                }
                let mut column: [bool; HEIGHT] = std::array::from_fn(|row| screen[row][coordinate]);
                column.rotate_right(shift % HEIGHT);
                for (row, pixel) in screen.iter_mut().zip(column) {
                    row[coordinate] = pixel;
                }
            }
        } else {
            return Err(invalid("unrecognised screen command"));
        }
    }
    Ok(screen)
}

pub fn part1(input: &str) -> Result<String, SolveError> {
    let count = build_screen(input)?
        .into_iter()
        .flatten()
        .filter(|&lit| lit)
        .count();
    Ok(count.to_string())
}

const GLYPHS: [(char, [&[u8; 5]; 6]); 16] = [
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

pub fn part2(input: &str) -> Result<String, SolveError> {
    let screen = build_screen(input)?;
    let mut answer = String::new();
    for start in (0..WIDTH).step_by(5) {
        let letter = GLYPHS.iter().find(|(_, glyph)| {
            screen.iter().zip(glyph).all(|(row, template)| {
                row[start..start + 5]
                    .iter()
                    .zip(*template)
                    .all(|(&lit, &pixel)| lit == (pixel == b'#'))
            })
        });
        let Some(&(letter, _)) = letter else {
            let pixels = screen
                .iter()
                .map(|row| {
                    row[start..start + 5]
                        .iter()
                        .map(|&lit| if lit { '#' } else { '.' })
                        .collect::<String>()
                })
                .collect::<Vec<_>>()
                .join("\n");
            return Err(SolveError::InvalidInput(format!(
                "unrecognised glyph {}\n{pixels}",
                start / 5 + 1
            )));
        };
        answer.push(letter);
    }
    Ok(answer)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rotates_across_both_edges() {
        let input = "rect 1x1\r\nrotate row y=0 by 99\r\nrotate column x=49 by 11";
        let screen = build_screen(input).unwrap();
        assert!(screen[5][49]);
        assert_eq!(part1(input).unwrap(), "1");
        assert_eq!(part1("rect 50x6").unwrap(), "300");
    }

    #[test]
    fn reports_invalid_commands_on_their_line() {
        for command in [
            "rect 51x1",
            "rect 1x7",
            "rect x2",
            "rotate row y=6 by 1",
            "rotate column x=50 by 1",
            "rotate row y=0 by 1 extra",
            "",
            "unknown",
        ] {
            let input = format!("rect 1x1\n{command}\n");
            assert!(matches!(
                part1(&input),
                Err(SolveError::AtLine { line: 2, .. })
            ));
        }
    }

    #[test]
    fn reports_unknown_glyph_and_its_pixels() {
        let Err(SolveError::InvalidInput(message)) = part2("rect 1x1") else {
            panic!("expected an unknown glyph error");
        };
        assert!(message.starts_with("unrecognised glyph 1\n#....\n"));
    }
}
