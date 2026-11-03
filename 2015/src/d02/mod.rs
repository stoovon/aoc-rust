use svutils::SolveError;

#[derive(Debug)]
struct Parcel {
    w: i64,
    d: i64,
    h: i64,
}

impl Parcel {
    fn parse(line: &str) -> Result<Self, SolveError> {
        let mut fields = line.split('x');
        let invalid_count =
            || SolveError::InvalidInput("expected three dimensions separated by 'x'".into());
        let mut dimensions = [0; 3];
        for dimension in &mut dimensions {
            let field = fields.next().ok_or_else(invalid_count)?;
            if field.is_empty() || !field.bytes().all(|byte| byte.is_ascii_digit()) {
                return Err(SolveError::InvalidInput(
                    "dimensions must be positive decimal integers".into(),
                ));
            }
            *dimension = field.parse()?;
            if *dimension == 0 {
                return Err(SolveError::InvalidInput(
                    "dimensions must be positive".into(),
                ));
            }
        }
        if fields.next().is_some() {
            return Err(invalid_count());
        }
        dimensions.sort_unstable();
        let [w, d, h] = dimensions;
        Ok(Self { w, d, h })
    }

    fn paper(&self) -> Result<i64, SolveError> {
        let smallest = self
            .w
            .checked_mul(self.d)
            .ok_or(SolveError::ArithmeticOverflow)?;
        let other = self
            .w
            .checked_mul(self.h)
            .ok_or(SolveError::ArithmeticOverflow)?;
        let largest = self
            .d
            .checked_mul(self.h)
            .ok_or(SolveError::ArithmeticOverflow)?;
        smallest
            .checked_add(other)
            .and_then(|sum| sum.checked_add(largest))
            .and_then(|sum| sum.checked_mul(2))
            .and_then(|surface| surface.checked_add(smallest))
            .ok_or(SolveError::ArithmeticOverflow)
    }

    fn ribbon(&self) -> Result<i64, SolveError> {
        let perimeter = self
            .w
            .checked_add(self.d)
            .and_then(|sum| sum.checked_mul(2))
            .ok_or(SolveError::ArithmeticOverflow)?;
        let volume = self
            .w
            .checked_mul(self.d)
            .and_then(|area| area.checked_mul(self.h))
            .ok_or(SolveError::ArithmeticOverflow)?;
        perimeter
            .checked_add(volume)
            .ok_or(SolveError::ArithmeticOverflow)
    }
}

fn parcels(input: &str) -> Result<Vec<Parcel>, SolveError> {
    if input.trim().is_empty() {
        return Err(SolveError::EmptyInput);
    }
    input
        .lines()
        .enumerate()
        .map(|(index, line)| Parcel::parse(line).map_err(|error| error.at_line(index + 1)))
        .collect()
}

pub fn part1(input: &str) -> Result<String, SolveError> {
    parcels(input)?
        .iter()
        .try_fold(0i64, |total, parcel| {
            total
                .checked_add(parcel.paper()?)
                .ok_or(SolveError::ArithmeticOverflow)
        })
        .map(|total| total.to_string())
}

pub fn part2(input: &str) -> Result<String, SolveError> {
    parcels(input)?
        .iter()
        .try_fold(0i64, |total, parcel| {
            total
                .checked_add(parcel.ribbon()?)
                .ok_or(SolveError::ArithmeticOverflow)
        })
        .map(|total| total.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use svutils::scaffold_test_result;

    #[test]
    fn generated_parcels_match_all_faces_and_perimeters() {
        for a in 1i64..=7 {
            for b in 1i64..=7 {
                for c in 1i64..=7 {
                    let input = format!("{a}x{b}x{c}");
                    let faces = [a * b, a * c, b * c];
                    let paper = 2 * faces.iter().sum::<i64>() + faces.iter().min().unwrap();
                    let ribbon = [2 * (a + b), 2 * (a + c), 2 * (b + c)]
                        .iter()
                        .min()
                        .unwrap()
                        + a * b * c;
                    assert_eq!(part1(&input).unwrap(), paper.to_string());
                    assert_eq!(part2(&input).unwrap(), ribbon.to_string());
                    let repeated = format!("{input}\r\n{input}\r\n");
                    assert_eq!(part1(&repeated).unwrap(), (2 * paper).to_string());
                    assert_eq!(part2(&repeated).unwrap(), (2 * ribbon).to_string());
                }
            }
        }
    }

    #[test]
    fn malformed_records_are_not_discarded() {
        for solver in [part1, part2] {
            for input in ["", " \r\n"] {
                assert!(matches!(solver(input), Err(SolveError::EmptyInput)));
            }
            for bad in [
                "4xjunkx5x6",
                "4x5",
                "4x5x6x7",
                "0x5x6",
                "-4x5x6",
                "+4x5x6",
                "4x 5x6",
                "4X5X6",
                "4xx6",
                "４x5x6",
                "",
                "999999999999999999999x5x6",
            ] {
                let input = format!("4x5x6\n{bad}\n");
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
    fn arithmetic_overflow_is_reported() {
        for solver in [part1, part2] {
            assert!(matches!(
                solver("9223372036854775807x2x2"),
                Err(SolveError::ArithmeticOverflow)
            ));
        }
        assert!(part1("2097152x2097152x2097152").is_ok());
        assert!(matches!(
            part2("2097152x2097152x2097152"),
            Err(SolveError::ArithmeticOverflow)
        ));
        let paper = "1000000000x1000000000x1000000000";
        assert!(part1(paper).is_ok());
        assert!(matches!(
            part1(&format!("{paper}\n{paper}")),
            Err(SolveError::ArithmeticOverflow)
        ));
        let ribbon = "2000000x2000000x2000000";
        assert!(part2(ribbon).is_ok());
        assert!(matches!(
            part2(&format!("{ribbon}\n{ribbon}")),
            Err(SolveError::ArithmeticOverflow)
        ));
        assert!(part1("1x1x1000000000").is_ok());
        assert!(part2("1x1x1000000000").is_ok());
    }

    #[test]
    fn test_part1_input() {
        scaffold_test_result(2015, 2, "input.txt", "input-spec.1.txt", part1);
    }

    #[test]
    fn test_part2_input() {
        scaffold_test_result(2015, 2, "input.txt", "input-spec.2.txt", part2);
    }
}
