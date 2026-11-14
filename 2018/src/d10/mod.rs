use svutils::{glyph_decoder, SolveError};

const FRAMES: i128 = 20_000;

struct Point {
    x: i128,
    y: i128,
    vx: i128,
    vy: i128,
}

impl Point {
    fn at(&self, time: i128) -> (i128, i128) {
        (self.x + self.vx * time, self.y + self.vy * time)
    }
}

fn parse(input: &str) -> Result<Vec<Point>, SolveError> {
    if input.is_empty() {
        return Err(SolveError::EmptyInput);
    }
    input
        .lines()
        .enumerate()
        .map(|(index, line)| parse_point(line).map_err(|error| error.at_line(index + 1)))
        .collect()
}

fn parse_point(line: &str) -> Result<Point, SolveError> {
    let invalid = || SolveError::InvalidInput("expected position=<x,y> velocity=<vx,vy>".into());
    let (position, velocity) = line
        .strip_prefix("position=<")
        .and_then(|line| line.strip_suffix('>'))
        .and_then(|line| line.split_once("> velocity=<"))
        .ok_or_else(invalid)?;
    let pair = |text: &str| -> Result<(i128, i128), SolveError> {
        let (x, y) = text.split_once(',').ok_or_else(invalid)?;
        Ok((
            x.trim().parse::<i64>()?.into(),
            y.trim().parse::<i64>()?.into(),
        ))
    };
    let (x, y) = pair(position)?;
    let (vx, vy) = pair(velocity)?;
    Ok(Point { x, y, vx, vy })
}

struct Bounds {
    min_x: i128,
    min_y: i128,
    max_x: i128,
    max_y: i128,
}

impl Bounds {
    fn at(points: &[Point], time: i128) -> Self {
        let (x, y) = points[0].at(time);
        let mut bounds = Self {
            min_x: x,
            min_y: y,
            max_x: x,
            max_y: y,
        };
        for point in &points[1..] {
            let (x, y) = point.at(time);
            bounds.min_x = bounds.min_x.min(x);
            bounds.min_y = bounds.min_y.min(y);
            bounds.max_x = bounds.max_x.max(x);
            bounds.max_y = bounds.max_y.max(y);
        }
        bounds
    }

    fn spans(&self) -> (i128, i128) {
        (self.max_x - self.min_x, self.max_y - self.min_y)
    }
}

fn best_frame(points: &[Point]) -> Result<(i128, Bounds), SolveError> {
    let mut best = None;
    for time in 0..FRAMES {
        let bounds = Bounds::at(points, time);
        let (width, height) = bounds.spans();
        if let Some(area) = width.checked_mul(height) {
            if best
                .as_ref()
                .is_none_or(|&(smallest, _, _)| area < smallest)
            {
                best = Some((area, time, bounds));
            }
        }
    }
    best.map(|(_, time, bounds)| (time, bounds))
        .ok_or(SolveError::ArithmeticOverflow)
}

pub fn part1(input: &str) -> Result<String, SolveError> {
    let points = parse(input)?;
    let (time, bounds) = best_frame(&points)?;
    let (x_span, y_span) = bounds.spans();
    let (width, height) = (x_span + 1, y_span + 1);
    if height != 10 {
        return Err(SolveError::InvalidInput("expected a 10-row message".into()));
    }
    if width > 100_000 {
        return Err(SolveError::ResourceLimit(
            "message exceeds one million pixels",
        ));
    }
    let mut screen = vec![vec![false; width as usize]; height as usize];
    for point in points {
        let (x, y) = point.at(time);
        screen[(y - bounds.min_y) as usize][(x - bounds.min_x) as usize] = true;
    }
    glyph_decoder::decode(&screen)
}

pub fn part2(input: &str) -> Result<String, SolveError> {
    let (time, _) = best_frame(&parse(input)?)?;
    Ok(time.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_earliest_minimum_and_decodes_translated_glyph() {
        let mut input = String::new();
        for (x, y) in (0..10).map(|y| (0, y)).chain((1..6).map(|x| (x, 9))) {
            let (vx, vy) = (x, y);
            input.push_str(&format!(
                "position=<{}, {}> velocity=<{vx}, {vy}>\r\n",
                -10 - 2 * x,
                -20 - 2 * y
            ));
        }
        assert_eq!(part2(&input).unwrap(), "2");
        let input = (0..10)
            .map(|y| (0, y))
            .chain((1..6).map(|x| (x, 9)))
            .map(|(x, y)| format!("position=<{}, {}> velocity=<0, 0>\n", x - 10, y - 20))
            .collect::<String>();
        assert_eq!(part1(&input).unwrap(), "L");
        assert_eq!(part2(&input).unwrap(), "0");
    }

    #[test]
    fn reports_invalid_rows() {
        assert!(matches!(part1(""), Err(SolveError::EmptyInput)));
        assert!(matches!(
            part2("position=<0, 0> velocity=<1, 2>\njunk"),
            Err(SolveError::AtLine { line: 2, .. })
        ));
    }
}
