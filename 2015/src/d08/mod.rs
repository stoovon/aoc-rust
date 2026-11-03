use svutils::SolveError;

struct Literal<'a> {
    source: &'a [u8],
    decoded_len: usize,
}

impl<'a> Literal<'a> {
    fn parse(line: &'a str) -> Result<Self, SolveError> {
        let source = line.as_bytes();
        let invalid = || {
            SolveError::InvalidInput("expected a quoted ASCII literal with valid escapes".into())
        };
        let body = source
            .strip_prefix(b"\"")
            .and_then(|rest| rest.strip_suffix(b"\""))
            .ok_or_else(invalid)?;
        let mut position = 0;
        let mut decoded_len = 0;
        while position < body.len() {
            match body[position] {
                b'\\' => match body.get(position + 1) {
                    Some(b'\\' | b'"') => position += 2,
                    Some(b'x') => {
                        let digits = body.get(position + 2..position + 4).ok_or_else(invalid)?;
                        if !digits.iter().all(u8::is_ascii_hexdigit) {
                            return Err(invalid());
                        }
                        position += 4;
                    }
                    _ => return Err(invalid()),
                },
                b'"' => return Err(invalid()),
                0x20..=0x7e => position += 1,
                _ => return Err(invalid()),
            }
            decoded_len += 1;
        }
        Ok(Self {
            source,
            decoded_len,
        })
    }

    fn encoding_overhead(&self) -> Result<usize, SolveError> {
        self.source
            .iter()
            .filter(|&&byte| matches!(byte, b'"' | b'\\'))
            .count()
            .checked_add(2)
            .ok_or(SolveError::ArithmeticOverflow)
    }
}

fn literals(
    input: &str,
) -> Result<impl Iterator<Item = Result<Literal<'_>, SolveError>>, SolveError> {
    if input.trim().is_empty() {
        return Err(SolveError::EmptyInput);
    }
    Ok(input
        .lines()
        .enumerate()
        .map(|(index, line)| Literal::parse(line).map_err(|error| error.at_line(index + 1))))
}

pub fn part1(input: &str) -> Result<String, SolveError> {
    literals(input)?
        .try_fold(0usize, |total, literal| {
            let literal = literal?;
            total
                .checked_add(literal.source.len() - literal.decoded_len)
                .ok_or(SolveError::ArithmeticOverflow)
        })
        .map(|total| total.to_string())
}

pub fn part2(input: &str) -> Result<String, SolveError> {
    literals(input)?
        .try_fold(0usize, |total, literal| {
            total
                .checked_add(literal?.encoding_overhead()?)
                .ok_or(SolveError::ArithmeticOverflow)
        })
        .map(|total| total.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use svutils::scaffold_test_result;

    fn encode(source: &[u8]) -> Vec<u8> {
        let mut encoded = vec![b'"'];
        for &byte in source {
            if matches!(byte, b'"' | b'\\') {
                encoded.push(b'\\');
            }
            encoded.push(byte);
        }
        encoded.push(b'"');
        encoded
    }

    #[test]
    fn generated_literals_match_constructed_decoded_and_encoded_lengths() {
        let tokens = [
            ("r", b'r'),
            (" ", b' '),
            (r#"\""#, b'"'),
            (r#"\\"#, b'\\'),
            (r#"\xA7"#, 0xa7),
        ];
        for length in 0..=5 {
            for mut encoded in 0..5usize.pow(length) {
                let mut source = String::from("\"");
                let mut decoded = Vec::new();
                for _ in 0..length {
                    let (token, byte) = tokens[encoded % 5];
                    encoded /= 5;
                    source.push_str(token);
                    decoded.push(byte);
                }
                source.push('"');
                let literal = Literal::parse(&source).unwrap();
                assert_eq!(literal.decoded_len, decoded.len());
                assert_eq!(
                    part1(&source).unwrap(),
                    (source.len() - decoded.len()).to_string()
                );
                assert_eq!(
                    part2(&source).unwrap(),
                    (encode(source.as_bytes()).len() - source.len()).to_string()
                );
                let records = format!("{source}\r\n{source}\r\n");
                assert_eq!(
                    part1(&records).unwrap(),
                    (2 * (source.len() - decoded.len())).to_string()
                );
                assert_eq!(
                    part2(&records).unwrap(),
                    (2 * (encode(source.as_bytes()).len() - source.len())).to_string()
                );
            }
        }
    }

    #[test]
    fn malformed_literals_have_line_context() {
        let valid = encode(b"rr");
        let valid = std::str::from_utf8(&valid).unwrap();
        for solver in [part1, part2] {
            for input in ["", " \r\n"] {
                assert!(matches!(solver(input), Err(SolveError::EmptyInput)));
            }
            for bad in [
                "",
                "r",
                "\"",
                "\"rr",
                "rr\"",
                "\"r\"r\"",
                r#""\q""#,
                r#""\x""#,
                r#""\xA""#,
                r#""\xGG""#,
                r#""r\""#,
                "\"é\"",
                "\"r\t\"",
                " \"rr\"",
                "\"rr\" ",
            ] {
                let input = format!("{valid}\n{bad}\n");
                assert!(
                    solver(&input)
                        .unwrap_err()
                        .to_string()
                        .starts_with("line 2:"),
                    "{bad:?}"
                );
            }
        }
    }

    #[test]
    fn hexadecimal_escape_digits_cover_all_byte_values() {
        for byte in 0..=255u8 {
            for digits in [format!("{byte:02x}"), format!("{byte:02X}")] {
                let source = format!("\"\\x{digits}\"");
                assert_eq!(Literal::parse(&source).unwrap().decoded_len, 1);
            }
        }
    }

    #[test]
    fn test_part1_input() {
        scaffold_test_result(2015, 8, "input.txt", "input-spec.1.txt", part1);
    }

    #[test]
    fn test_part2_input() {
        scaffold_test_result(2015, 8, "input.txt", "input-spec.2.txt", part2);
    }
}
