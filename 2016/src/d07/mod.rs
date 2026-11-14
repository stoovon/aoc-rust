use svutils::SolveError;

fn segments(line: &str) -> impl Iterator<Item = Result<(bool, &[u8]), SolveError>> {
    let mut rest = Some(line.as_bytes());
    let mut inside = false;
    std::iter::from_fn(move || {
        let bytes = rest.take()?;
        let boundary = bytes.iter().position(|byte| matches!(byte, b'[' | b']'));
        let segment = &bytes[..boundary.unwrap_or(bytes.len())];
        let valid_boundary = match boundary {
            Some(index) => bytes[index] == if inside { b']' } else { b'[' },
            None => !inside,
        };
        if !valid_boundary || !segment.iter().all(u8::is_ascii_lowercase) {
            return Some(Err(SolveError::InvalidInput(
                "invalid address segment or bracket".into(),
            )));
        }

        let result = (inside, segment);
        if let Some(index) = boundary {
            rest = Some(&bytes[index + 1..]);
            inside = !inside;
        }
        Some(Ok(result))
    })
}

fn supports_tls(line: &str) -> Result<bool, SolveError> {
    let mut outside_match = false;
    for segment in segments(line) {
        let (inside, bytes) = segment?;
        if bytes
            .windows(4)
            .any(|w| w[0] != w[1] && w[0] == w[3] && w[1] == w[2])
        {
            if inside {
                return Ok(false);
            }
            outside_match = true;
        }
    }
    Ok(outside_match)
}

fn supports_ssl(line: &str) -> Result<bool, SolveError> {
    let mut outside = [[false; 26]; 26];
    let mut inside = [[false; 26]; 26];
    let mut matched = false;
    for segment in segments(line) {
        let (bracketed, bytes) = segment?;
        if matched {
            continue;
        }
        let (current, opposite) = if bracketed {
            (&mut inside, &outside)
        } else {
            (&mut outside, &inside)
        };
        for window in bytes.windows(3) {
            let [first, middle, last] = [window[0], window[1], window[2]];
            if first == last && first != middle {
                let a = usize::from(first - b'a');
                let b = usize::from(middle - b'a');
                if opposite[b][a] {
                    matched = true;
                    break;
                }
                current[a][b] = true;
            }
        }
    }
    Ok(matched)
}

fn count_addresses(
    input: &str,
    supports: fn(&str) -> Result<bool, SolveError>,
) -> Result<String, SolveError> {
    if input.is_empty() {
        return Err(SolveError::EmptyInput);
    }
    let mut count = 0usize;
    for (index, line) in input.lines().enumerate() {
        if line.is_empty() {
            return Err(SolveError::InvalidInput("empty address".into()).at_line(index + 1));
        }
        count += usize::from(supports(line).map_err(|error| error.at_line(index + 1))?);
    }
    Ok(count.to_string())
}

pub fn part1(input: &str) -> Result<String, SolveError> {
    count_addresses(input, supports_tls)
}

pub fn part2(input: &str) -> Result<String, SolveError> {
    count_addresses(input, supports_ssl)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tls_respects_segment_boundaries_and_checks_later_segments() {
        assert_eq!(
            part1("mnnm[q]rs\nmn[q]nm\nmnnm[q]rs[twwt]uv\nssss").unwrap(),
            "1"
        );
        assert_eq!(part1("mnnm[twwt]uv]").unwrap(), "0");
        for line in ["mnnm]q", "mnnm[q", "mnnm[q[r]s]", "mnnm[Q]rs"] {
            let input = format!("mnnm[q]rs\n{line}");
            assert!(matches!(
                part1(&input),
                Err(SolveError::AtLine { line: 2, .. })
            ));
        }
    }

    #[test]
    fn ssl_matches_in_either_order_without_crossing_boundaries() {
        let input = "kpk[pkp]r\nr[pkp]kpk\nkp[q]k[pkp]r\nkkk[ppp]r\nkpk[kpk]r";
        assert_eq!(part2(input).unwrap(), "2");
        assert!(matches!(
            part2("kpk[pkp]r]"),
            Err(SolveError::AtLine { line: 1, .. })
        ));
    }
}
