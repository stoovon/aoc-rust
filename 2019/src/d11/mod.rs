use std::collections::HashMap;

use crate::intcode::{Event, Machine};
use svutils::{glyph_decoder, SolveError};

type Position = (i64, i64);
const DIRECTIONS: [Position; 4] = [(0, -1), (1, 0), (0, 1), (-1, 0)];

fn paint(input: &str, start_white: bool) -> Result<HashMap<Position, bool>, SolveError> {
    let mut machine = Machine::parse(input)?;
    let mut panels = HashMap::new();
    let mut position = (0i64, 0i64);
    let mut direction = 0;
    let mut pending_paint = None;
    let mut input = None;
    let protocol_error = |message: &str| SolveError::InvalidInput(message.into());

    loop {
        match machine.run(input.take())? {
            Event::NeedInput => {
                let white = panels
                    .get(&position)
                    .copied()
                    .unwrap_or(start_white && position == (0, 0));
                input = Some(i64::from(white));
            }
            Event::Output(value) => {
                if !(0..=1).contains(&value) {
                    return Err(protocol_error("expected robot output 0 or 1"));
                }
                if let Some(white) = pending_paint.take() {
                    panels.insert(position, white);
                    direction = (direction + if value == 0 { 3 } else { 1 }) % DIRECTIONS.len();
                    let (dx, dy) = DIRECTIONS[direction];
                    position = (
                        position
                            .0
                            .checked_add(dx)
                            .ok_or(SolveError::ArithmeticOverflow)?,
                        position
                            .1
                            .checked_add(dy)
                            .ok_or(SolveError::ArithmeticOverflow)?,
                    );
                } else {
                    pending_paint = Some(value == 1);
                }
            }
            Event::Halted => {
                if pending_paint.is_some() {
                    return Err(protocol_error(
                        "robot halted after paint output without a turn",
                    ));
                }
                return Ok(panels);
            }
        }
    }
}

pub fn part1(input: &str) -> Result<String, SolveError> {
    Ok(paint(input, false)?.len().to_string())
}

pub fn part2(input: &str) -> Result<String, SolveError> {
    let mut panels = paint(input, true)?;
    panels.entry((0, 0)).or_insert(true);
    let mut white = panels
        .into_iter()
        .filter_map(|(position, white)| white.then_some(position));
    let (x, y) = white
        .next()
        .ok_or(SolveError::NoSolution("robot left no white panels"))?;
    let mut pixels = vec![(x, y)];
    let (mut left, mut right, mut top, mut bottom) = (x, x, y, y);
    for (x, y) in white {
        left = left.min(x);
        right = right.max(x);
        top = top.min(y);
        bottom = bottom.max(y);
        pixels.push((x, y));
    }
    let width = right
        .abs_diff(left)
        .checked_add(1)
        .ok_or(SolveError::ArithmeticOverflow)?;
    let height = bottom
        .abs_diff(top)
        .checked_add(1)
        .ok_or(SolveError::ArithmeticOverflow)?;
    if !matches!(height, 6 | 10) {
        return Err(SolveError::InvalidInput(
            "expected a registration with 6 or 10 rows".into(),
        ));
    }
    let mut row = Vec::new();
    let width = usize::try_from(width).map_err(|_| SolveError::ArithmeticOverflow)?;
    row.try_reserve_exact(width)
        .map_err(|_| SolveError::ResourceLimit("registration grid is too wide"))?;
    row.resize(width, false);
    let mut screen = vec![row; height as usize];
    for (x, y) in pixels {
        screen[y.abs_diff(top) as usize][x.abs_diff(left) as usize] = true;
    }
    glyph_decoder::decode(&screen)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counts_only_painted_panels_and_preserves_latest_colour() {
        let program = "3,100,4,100,104,1,104,1,104,1,104,0,104,1,104,1,104,1,104,0,104,0,99";
        assert_eq!(part1(program).unwrap(), "4");
        let panels = paint(program, false).unwrap();
        assert_eq!(panels.get(&(0, 0)), Some(&false));
        assert_eq!(panels.get(&(1, 0)), Some(&true));
        assert_eq!(panels.get(&(0, 1)), Some(&true));
        assert_eq!(
            paint("3,10,4,10,104,1,99", true).unwrap().get(&(0, 0)),
            Some(&true)
        );
        assert_eq!(part1("99").unwrap(), "0");
    }

    #[test]
    fn rejects_invalid_and_incomplete_output() {
        assert!(part1("104,2,104,1,99").is_err());
        assert!(part1("104,1,104,2,99").is_err());
        assert!(part1("104,1,99").is_err());
    }
}
