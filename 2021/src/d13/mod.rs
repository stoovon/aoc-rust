use std::collections::HashSet;
use svutils::{glyph_decoder, SolveError};

type Dot = (usize, usize);

enum Fold {
    X(usize),
    Y(usize),
}

impl Fold {
    fn apply(&self, dots: HashSet<Dot>) -> Result<HashSet<Dot>, SolveError> {
        fn reflect(value: usize, crease: usize) -> Result<usize, SolveError> {
            if value <= crease {
                return Ok(value);
            }
            crease.checked_sub(value - crease).ok_or_else(|| {
                SolveError::InvalidInput("fold reflects a dot past the paper origin".into())
            })
        }

        dots.into_iter()
            .map(|(x, y)| match *self {
                Self::X(crease) => Ok((reflect(x, crease)?, y)),
                Self::Y(crease) => Ok((x, reflect(y, crease)?)),
            })
            .collect()
    }
}

fn parse(input: &str) -> Result<(HashSet<Dot>, Vec<(usize, Fold)>), SolveError> {
    if input.is_empty() {
        return Err(SolveError::EmptyInput);
    }
    let mut dots = HashSet::new();
    let mut folds = Vec::new();
    let mut reading_folds = false;
    for (index, line) in input.lines().enumerate() {
        let line_number = index + 1;
        let invalid = |message: &str| SolveError::InvalidInput(message.into()).at_line(line_number);
        let number = |text: &str| {
            text.parse::<usize>()
                .map_err(|error| SolveError::from(error).at_line(line_number))
        };

        if !reading_folds && line.is_empty() {
            reading_folds = true;
        } else if reading_folds {
            let (axis, crease) = line
                .strip_prefix("fold along ")
                .and_then(|text| text.split_once('='))
                .ok_or_else(|| invalid("expected fold along x=N or y=N"))?;
            let fold = match axis {
                "x" => Fold::X(number(crease)?),
                "y" => Fold::Y(number(crease)?),
                _ => return Err(invalid("expected fold axis x or y")),
            };
            folds.push((line_number, fold));
        } else {
            let (x, y) = line
                .split_once(',')
                .ok_or_else(|| invalid("expected x,y dot coordinates"))?;
            dots.insert((number(x)?, number(y)?));
        }
    }
    if dots.is_empty() || folds.is_empty() {
        return Err(SolveError::InvalidInput(
            "expected dots and at least one fold".into(),
        ));
    }
    Ok((dots, folds))
}

pub fn part1(input: &str) -> Result<String, SolveError> {
    let (dots, folds) = parse(input)?;
    let (line, fold) = &folds[0];
    Ok(fold
        .apply(dots)
        .map_err(|error| error.at_line(*line))?
        .len()
        .to_string())
}

pub fn part2(input: &str) -> Result<String, SolveError> {
    let (mut dots, folds) = parse(input)?;
    for (line, fold) in folds {
        dots = fold.apply(dots).map_err(|error| error.at_line(line))?;
    }
    let width = dots
        .iter()
        .map(|&(x, _)| x)
        .max()
        .unwrap()
        .checked_add(1)
        .ok_or(SolveError::ArithmeticOverflow)?;
    let height = dots
        .iter()
        .map(|&(_, y)| y)
        .max()
        .unwrap()
        .checked_add(1)
        .ok_or(SolveError::ArithmeticOverflow)?;
    if width.saturating_mul(height) > 1_000_000 {
        return Err(SolveError::ResourceLimit(
            "folded screen exceeds one million pixels",
        ));
    }
    let mut screen = vec![vec![false; width]; height];
    for (x, y) in dots {
        screen[y][x] = true;
    }
    glyph_decoder::decode(&screen)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn folds_merge_dots_and_preserve_crease() {
        let dots = HashSet::from([(2, 1), (8, 1), (5, 3), (5, 1)]);
        let dots = Fold::X(5).apply(dots).unwrap();
        assert_eq!(dots, HashSet::from([(2, 1), (5, 3), (5, 1)]));
        assert_eq!(
            Fold::Y(2).apply(dots).unwrap(),
            HashSet::from([(2, 1), (5, 1)])
        );
    }

    #[test]
    fn validates_fold_syntax_and_reflection() {
        assert!(matches!(
            part1("8,1\n\nfold along x=3"),
            Err(SolveError::AtLine { line: 3, .. })
        ));
        assert!(matches!(
            part2("2,1\n\nfold along z=3"),
            Err(SolveError::AtLine { line: 3, .. })
        ));
        assert!(part1("2,1").is_err());
    }

    #[test]
    fn decodes_folded_glyph() {
        let input =
            "0,0\r\n0,1\r\n0,2\r\n0,3\r\n0,4\r\n0,5\r\n1,5\r\n2,5\r\n7,5\r\n\r\nfold along x=5";
        assert_eq!(part1(input).unwrap(), "9");
        assert_eq!(part2(input).unwrap(), "L");
    }
}
