use svutils::SolveError;

struct Frame {
    end: usize,
    repeats: u128,
    length: u128,
}

fn marker(bytes: &[u8]) -> Result<(usize, u128, usize), SolveError> {
    let invalid = || SolveError::InvalidInput("expected a marker of the form (AxB)".into());
    let end = bytes
        .iter()
        .position(|&byte| byte == b')')
        .ok_or_else(invalid)?;
    let text = std::str::from_utf8(&bytes[1..end]).map_err(|_| invalid())?;
    let (width, repeats) = text.split_once('x').ok_or_else(invalid)?;
    if [width, repeats]
        .iter()
        .any(|number| number.is_empty() || !number.bytes().all(|byte| byte.is_ascii_digit()))
    {
        return Err(invalid());
    }
    Ok((width.parse()?, repeats.parse()?, end + 1))
}

fn length(input: &str, nested: bool) -> Result<String, SolveError> {
    if !input.is_ascii() {
        return Err(SolveError::InvalidInput(
            "expected ASCII compressed text".into(),
        ));
    }
    let bytes: Vec<_> = input
        .bytes()
        .filter(|byte| !byte.is_ascii_whitespace())
        .collect();
    if bytes.is_empty() {
        return Err(SolveError::EmptyInput);
    }
    let mut frames = vec![Frame {
        end: bytes.len(),
        repeats: 1,
        length: 0,
    }];
    let mut cursor = 0;
    loop {
        let frame = frames.last_mut().unwrap();
        if cursor == frame.end {
            let expanded = frame
                .length
                .checked_mul(frame.repeats)
                .ok_or(SolveError::ArithmeticOverflow)?;
            frames.pop();
            let Some(parent) = frames.last_mut() else {
                return Ok(expanded.to_string());
            };
            parent.length = parent
                .length
                .checked_add(expanded)
                .ok_or(SolveError::ArithmeticOverflow)?;
        } else if bytes[cursor] == b'(' {
            let (width, repeats, header) = marker(&bytes[cursor..frame.end])?;
            cursor += header;
            if width > frame.end - cursor {
                return Err(SolveError::InvalidInput(
                    "marker extends beyond its block".into(),
                ));
            }
            if nested {
                frames.push(Frame {
                    end: cursor + width,
                    repeats,
                    length: 0,
                });
            } else {
                cursor += width;
                let expanded = (width as u128)
                    .checked_mul(repeats)
                    .ok_or(SolveError::ArithmeticOverflow)?;
                frame.length = frame
                    .length
                    .checked_add(expanded)
                    .ok_or(SolveError::ArithmeticOverflow)?;
            }
        } else {
            cursor += 1;
            frame.length = frame
                .length
                .checked_add(1)
                .ok_or(SolveError::ArithmeticOverflow)?;
        }
    }
}

pub fn part1(input: &str) -> Result<String, SolveError> {
    length(input, false)
}

pub fn part2(input: &str) -> Result<String, SolveError> {
    length(input, true)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counts_nested_markers_only_in_part_two() {
        let input = "Z(7x3)(2x4)Q R";
        assert_eq!(part1(input).unwrap(), "22");
        assert_eq!(part2(input).unwrap(), "25");
    }

    #[test]
    fn rejects_markers_that_cross_a_block_boundary() {
        assert_eq!(part1("(5x2)(3x4)XYZ").unwrap(), "13");
        assert!(part2("(5x2)(3x4)XYZ").is_err());
        assert!(part1("(4x2)XYZ").is_err());
    }

    #[test]
    fn detects_overflow() {
        let input = format!("(2x{})QR", u128::MAX);
        assert!(matches!(part2(&input), Err(SolveError::ArithmeticOverflow)));
    }
}
