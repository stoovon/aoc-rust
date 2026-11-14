use svutils::{glyph_decoder, SolveError};

const WIDTH: usize = 40;
const HEIGHT: usize = 6;

fn simulate(input: &str, mut observe: impl FnMut(usize, i64)) -> Result<(), SolveError> {
    if input.is_empty() {
        return Err(SolveError::EmptyInput);
    }

    let mut cycle = 0;
    let mut x = 1i64;

    for (index, line) in input.lines().enumerate() {
        let line_number = index + 1;
        let mut tokens = line.split_ascii_whitespace();
        let (duration, delta) = match (tokens.next(), tokens.next(), tokens.next()) {
            (Some("noop"), None, None) => (1, 0),
            (Some("addx"), Some(value), None) => (
                2,
                value
                    .parse::<i64>()
                    .map_err(|error| SolveError::from(error).at_line(line_number))?,
            ),
            _ => {
                return Err(
                    SolveError::InvalidInput("expected noop or addx <integer>".into())
                        .at_line(line_number),
                );
            }
        };

        for _ in 0..duration {
            cycle += 1;
            observe(cycle, x);
        }
        x = x
            .checked_add(delta)
            .ok_or_else(|| SolveError::ArithmeticOverflow.at_line(line_number))?;
    }
    Ok(())
}

pub fn part1(input: &str) -> Result<String, SolveError> {
    let mut sum = 0i128;
    simulate(input, |cycle, x| {
        if (20..=220).contains(&cycle) && cycle % 40 == 20 {
            sum += cycle as i128 * i128::from(x);
        }
    })?;
    Ok(sum.to_string())
}

pub fn part2(input: &str) -> Result<String, SolveError> {
    let mut screen = [[false; WIDTH]; HEIGHT];
    simulate(input, |cycle, x| {
        let offset = cycle - 1;
        let column = offset % WIDTH;
        if let Some(row) = screen.get_mut(offset / WIDTH) {
            row[column] = x.abs_diff(column as i64) <= 1;
        }
    })?;
    glyph_decoder::decode(&screen)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn addx_commits_after_its_second_cycle() {
        let mut values = Vec::new();
        simulate("addx 3\r\nnoop\r\naddx -5\r\nnoop", |cycle, x| {
            values.push((cycle, x));
        })
        .unwrap();
        assert_eq!(values, [(1, 1), (2, 1), (3, 4), (4, 4), (5, 4), (6, -1)]);
    }

    #[test]
    fn samples_before_the_register_update() {
        let input = format!("{}addx 3\n{}", "noop\n".repeat(18), "noop\n".repeat(40));
        assert_eq!(part1(&input).unwrap(), "260");
    }

    #[test]
    fn reports_invalid_instructions_and_overflow() {
        assert!(matches!(part1(""), Err(SolveError::EmptyInput)));
        for instruction in [
            "addx",
            "addx nope",
            "noop 1",
            "addx 1 extra",
            "addx 9223372036854775807",
        ] {
            let input = format!("noop\n{instruction}");
            for solve in [part1, part2] {
                assert!(matches!(
                    solve(&input),
                    Err(SolveError::AtLine { line: 2, .. })
                ));
            }
        }
    }
}
