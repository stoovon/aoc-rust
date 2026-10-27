use std::borrow::Cow;
use svutils::SolveError;

fn holiday_hash(s: &str) -> usize {
    s.as_bytes()
        .iter()
        .fold(0, |hash, &byte| (hash + usize::from(byte)) * 17 % 256)
}

struct Lens<'a> {
    label: &'a str,
    focal_length: i64,
}

struct LensBoxes<'a> {
    boxes: [Vec<Lens<'a>>; 256],
}

impl<'a> LensBoxes<'a> {
    fn new() -> Self {
        Self {
            boxes: std::array::from_fn(|_| Vec::new()),
        }
    }

    fn insert(&mut self, label: &'a str, focal_length: i64) {
        let lens_box = &mut self.boxes[holiday_hash(label)];
        if let Some(lens) = lens_box.iter_mut().find(|lens| lens.label == label) {
            lens.focal_length = focal_length;
        } else {
            lens_box.push(Lens {
                label,
                focal_length,
            });
        }
    }

    fn remove(&mut self, label: &str) {
        let lens_box = &mut self.boxes[holiday_hash(label)];
        if let Some(index) = lens_box.iter().position(|lens| lens.label == label) {
            lens_box.remove(index);
        }
    }

    fn total_power(&self) -> Result<i64, SolveError> {
        let mut total = 0_i64;
        for (index, lens_box) in self.boxes.iter().enumerate() {
            let box_number =
                i64::try_from(index + 1).map_err(|_| SolveError::ArithmeticOverflow)?;
            for (slot, lens) in lens_box.iter().enumerate() {
                let slot_number =
                    i64::try_from(slot + 1).map_err(|_| SolveError::ArithmeticOverflow)?;
                let power = box_number
                    .checked_mul(slot_number)
                    .and_then(|weight| weight.checked_mul(lens.focal_length))
                    .ok_or(SolveError::ArithmeticOverflow)?;
                total = total
                    .checked_add(power)
                    .ok_or(SolveError::ArithmeticOverflow)?;
            }
        }
        Ok(total)
    }
}

enum Instruction<'a> {
    Remove { label: &'a str },
    Set { label: &'a str, focal_length: i64 },
}

struct Step<'a> {
    raw: &'a str,
    instruction: Instruction<'a>,
}

fn parse_step(raw: &str) -> Result<Step<'_>, SolveError> {
    let valid_label =
        |label: &str| !label.is_empty() && label.bytes().all(|byte| byte.is_ascii_lowercase());
    let instruction = if let Some(label) = raw.strip_suffix('-') {
        if !valid_label(label) {
            return Err(SolveError::InvalidInput(
                "removal needs a lowercase ASCII label followed by one '-'".into(),
            ));
        }
        Instruction::Remove { label }
    } else {
        let (label, length) = raw.split_once('=').ok_or_else(|| {
            SolveError::InvalidInput("step must remove a label or set its focal length".into())
        })?;
        if !valid_label(label) {
            return Err(SolveError::InvalidInput(
                "lens label must contain lowercase ASCII letters".into(),
            ));
        }
        if length.is_empty() || !length.bytes().all(|byte| byte.is_ascii_digit()) {
            return Err(SolveError::InvalidInput(
                "focal length must be a positive decimal integer".into(),
            ));
        }
        let focal_length: i64 = length.parse()?;
        if focal_length == 0 {
            return Err(SolveError::InvalidInput(
                "focal length must be positive".into(),
            ));
        }
        Instruction::Set {
            label,
            focal_length,
        }
    };
    Ok(Step { raw, instruction })
}

fn parse_steps(input: &str) -> Result<Vec<Step<'_>>, SolveError> {
    if input.trim().is_empty() {
        return Err(SolveError::EmptyInput);
    }
    input
        .split(',')
        .enumerate()
        .map(|(index, raw)| parse_step(raw).map_err(|error| error.at_step(index + 1)))
        .collect()
}

fn without_newlines(input: &str) -> Cow<'_, str> {
    if input.contains(['\n', '\r']) {
        Cow::Owned(
            input
                .chars()
                .filter(|&character| !matches!(character, '\n' | '\r'))
                .collect(),
        )
    } else {
        Cow::Borrowed(input)
    }
}

pub fn part1(input: &str) -> Result<String, SolveError> {
    let input = without_newlines(input);
    let steps = parse_steps(&input)?;
    let total = steps.iter().try_fold(0_i64, |total, step| {
        total
            .checked_add(holiday_hash(step.raw) as i64)
            .ok_or(SolveError::ArithmeticOverflow)
    })?;
    Ok(total.to_string())
}

pub fn part2(input: &str) -> Result<String, SolveError> {
    let input = without_newlines(input);
    let steps = parse_steps(&input)?;
    let mut boxes = LensBoxes::new();
    for step in steps {
        match step.instruction {
            Instruction::Remove { label } => boxes.remove(label),
            Instruction::Set {
                label,
                focal_length,
            } => boxes.insert(label, focal_length),
        }
    }
    Ok(boxes.total_power()?.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use svutils::scaffold_test_result;

    const YEAR: i16 = 2023;
    const DAY: i16 = 15;

    #[test]
    fn test_part1_input() {
        scaffold_test_result(YEAR, DAY, "input.txt", "input-spec.1.txt", part1);
    }

    #[test]
    fn test_part2_input() {
        scaffold_test_result(YEAR, DAY, "input.txt", "input-spec.2.txt", part2);
    }

    #[test]
    fn malformed_and_semantically_invalid_steps_are_rejected() {
        for input in [
            ",a=1",
            "a=1,",
            "a=1,,b=2",
            "a",
            "a--",
            "a-b",
            "a-=1",
            "=1",
            "-",
            "A=1",
            "é=1",
            "a=",
            "a=0",
            "a=-1",
            "a=+1",
            "a=nope",
            "a=1=2",
            "a=1 ",
            "a=1\nb=2",
            "a=9223372036854775808",
        ] {
            assert!(part1(input).is_err(), "{input:?}");
            assert!(part2(input).is_err(), "{input:?}");
        }
        assert!(matches!(part1(""), Err(SolveError::EmptyInput)));
        assert!(matches!(part2(" \t\n"), Err(SolveError::EmptyInput)));
    }

    #[test]
    fn raw_spelling_is_preserved_while_instructions_are_parsed() {
        for input in ["a=01", "a=01\n", "a=01\r\n"] {
            assert_eq!(part1(input).unwrap(), holiday_hash("a=01").to_string());
            assert_eq!(part2(input).unwrap(), "114");
        }
        assert_ne!(holiday_hash("a=01"), holiday_hash("a=1"));
    }

    #[test]
    fn absent_removal_and_replacement_follow_the_puzzle_rules() {
        assert_eq!(part2("a-").unwrap(), "0");
        assert_eq!(part2("a=1,a=2").unwrap(), "228");
        assert_eq!(part2("a=1,a-,a-").unwrap(), "0");
    }

    #[test]
    fn focusing_power_overflow_returns_an_error() {
        assert!(matches!(
            part2(&format!("a={}", i64::MAX)),
            Err(SolveError::ArithmeticOverflow)
        ));
    }

    #[test]
    fn numeric_failure_keeps_its_step_and_error_source() {
        use std::error::Error;
        let error = part1("a=1,b=9223372036854775808").unwrap_err();
        assert!(matches!(error, SolveError::AtStep { step: 2, .. }));
        let source = error
            .source()
            .unwrap()
            .downcast_ref::<SolveError>()
            .unwrap();
        assert!(matches!(source, SolveError::InvalidNumber(_)));
        assert!(source.source().is_some());
    }

    #[test]
    fn colliding_labels_preserve_slots_when_replaced_removed_and_reinserted() {
        let labels = ["a", "hy", "ih"];
        assert!(labels.iter().all(|label| holiday_hash(label) == 113));
        assert_eq!(part2("a=1,hy=2,ih=3").unwrap(), "1596");
        assert_eq!(part2("a=1,hy=2,ih=3,hy=5").unwrap(), "2280");
        assert_eq!(part2("a=1,hy=2,ih=3,hy=5,a-").unwrap(), "1254");
        assert_eq!(part2("a=1,hy=2,ih=3,hy=5,a-,a=7").unwrap(), "3648");
        assert_eq!(part2("a=1,hy=2,ih=3,hy-").unwrap(), "798");
    }

    #[test]
    fn newlines_are_ignored_as_required_by_the_initialization_sequence() {
        for input in ["a=1\n\n", "a\n=1", "a=1,\nhy=2", "a=1,\r\nhy=2\r\n"] {
            let normalized = input.replace(['\n', '\r'], "");
            assert_eq!(part1(input).unwrap(), part1(&normalized).unwrap());
            assert_eq!(part2(input).unwrap(), part2(&normalized).unwrap());
        }
    }
}
