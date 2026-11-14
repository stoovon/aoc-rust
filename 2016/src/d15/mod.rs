use regex::Regex;
use std::collections::HashSet;
use svutils::SolveError;

#[derive(Clone, Copy)]
struct Congruence {
    residue: i128,
    period: i128,
}

fn extended_gcd(mut a: i128, mut b: i128) -> Result<(i128, i128), SolveError> {
    let (mut x, mut next_x) = (1_i128, 0_i128);
    while b != 0 {
        let quotient = a / b;
        (a, b) = (b, a % b);
        let updated = quotient
            .checked_mul(next_x)
            .and_then(|product| x.checked_sub(product))
            .ok_or(SolveError::ArithmeticOverflow)?;
        (x, next_x) = (next_x, updated);
    }
    Ok((a, x))
}

impl Congruence {
    fn combine(self, other: Self) -> Result<Option<Self>, SolveError> {
        let (gcd, inverse) = extended_gcd(self.period, other.period)?;
        let difference = other.residue - self.residue;
        if difference % gcd != 0 {
            return Ok(None);
        }
        let reduced = other.period / gcd;
        let multiplier = (((difference / gcd).rem_euclid(reduced) as u128
            * inverse.rem_euclid(reduced) as u128)
            % reduced as u128) as i128;
        let period = self
            .period
            .checked_mul(reduced)
            .ok_or(SolveError::ArithmeticOverflow)?;
        let residue = self
            .period
            .checked_mul(multiplier)
            .and_then(|offset| self.residue.checked_add(offset))
            .ok_or(SolveError::ArithmeticOverflow)?
            % period;
        Ok(Some(Self { residue, period }))
    }
}

fn solve(input: &str, extra_disc: bool) -> Result<String, SolveError> {
    if input.trim().is_empty() {
        return Err(SolveError::EmptyInput);
    }
    let pattern = Regex::new(
        r"\ADisc #([0-9]+) has ([0-9]+) positions; at time=0, it is at position ([0-9]+)\.\z",
    )
    .expect("disc pattern is valid");
    let mut indices = HashSet::new();
    let mut discs = Vec::new();
    let mut last_index = 0;
    for (line, text) in input.lines().enumerate() {
        let invalid = |message: &str| SolveError::InvalidInput(message.into()).at_line(line + 1);
        let fields = pattern
            .captures(text)
            .ok_or_else(|| invalid("expected a disc description"))?;
        let number = |text: &str| {
            text.parse::<u64>()
                .map_err(|error| SolveError::from(error).at_line(line + 1))
        };
        let index = number(&fields[1])?;
        let period = number(&fields[2])?;
        let position = number(&fields[3])?;
        if index == 0 || !indices.insert(index) || position >= period {
            return Err(invalid(
                "expected a unique positive disc index and a position within its range",
            ));
        }
        last_index = last_index.max(index);
        let period = i128::from(period);
        let residue = (-i128::from(index) - i128::from(position)).rem_euclid(period);
        discs.push(Congruence { residue, period });
    }
    if extra_disc {
        discs.push(Congruence {
            residue: (-i128::from(last_index) - 1).rem_euclid(11),
            period: 11,
        });
    }

    let mut time = Congruence {
        residue: 0,
        period: 1,
    };
    for disc in discs {
        time = time
            .combine(disc)?
            .ok_or(SolveError::NoSolution("disc alignments are incompatible"))?;
    }
    Ok(time.residue.to_string())
}

pub fn part1(input: &str) -> Result<String, SolveError> {
    solve(input, false)
}

pub fn part2(input: &str) -> Result<String, SolveError> {
    solve(input, true)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn handles_shared_factors_and_incompatible_discs() {
        let input = "Disc #1 has 6 positions; at time=0, it is at position 5.\r\nDisc #2 has 8 positions; at time=0, it is at position 2.";
        assert_eq!(part1(input).unwrap(), "12");
        assert_eq!(part2(input).unwrap(), "228");
        let incompatible = input.replace("at position 2.", "at position 3.");
        assert!(matches!(
            part1(&incompatible),
            Err(SolveError::NoSolution(_))
        ));
    }

    #[test]
    fn rejects_invalid_ranges_and_duplicate_indices() {
        let disc = "Disc #1 has 4 positions; at time=0, it is at position 2.";
        for row in [
            disc,
            "Disc #2 has 0 positions; at time=0, it is at position 0.",
            "junk",
        ] {
            assert!(matches!(
                part1(&format!("{disc}\n{row}")),
                Err(SolveError::AtLine { line: 2, .. })
            ));
        }
    }
}
