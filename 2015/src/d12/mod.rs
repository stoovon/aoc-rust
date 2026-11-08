use serde_json::Value;
use svutils::SolveError;

fn sum_values<'a>(
    mut values: impl Iterator<Item = &'a Value>,
    ignore_red: bool,
) -> Result<i64, SolveError> {
    values.try_fold(0i64, |total, value| {
        total
            .checked_add(sum(value, ignore_red)?)
            .ok_or(SolveError::ArithmeticOverflow)
    })
}

fn sum(value: &Value, ignore_red: bool) -> Result<i64, SolveError> {
    match value {
        Value::Number(number) => number
            .as_i64()
            .ok_or_else(|| SolveError::InvalidInput("expected a signed 64-bit integer".into())),
        Value::Array(values) => sum_values(values.iter(), ignore_red),
        Value::Object(values) => {
            if ignore_red && values.values().any(|value| value == "red") {
                Ok(0)
            } else {
                sum_values(values.values(), ignore_red)
            }
        }
        Value::Null | Value::Bool(_) | Value::String(_) => Ok(0),
    }
}

fn solve(input: &str, ignore_red: bool) -> Result<String, SolveError> {
    if input.trim().is_empty() {
        return Err(SolveError::EmptyInput);
    }
    let value: Value =
        serde_json::from_str(input).map_err(|error| SolveError::InvalidInput(error.to_string()))?;
    sum(&value, ignore_red).map(|total| total.to_string())
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
    use svutils::scaffold_test_result;

    #[test]
    fn red_policy_is_local_to_object_values() {
        let input =
            r#"{"count":17,"payload":{"shade":"red","weight":-6},"items":[3,"red",{"red":8}]}"#;
        assert_eq!(part1(input).unwrap(), "22");
        assert_eq!(part2(input).unwrap(), "28");
        assert_eq!(
            part2(r#"{"shade":"red","child":{"weight":17}}"#).unwrap(),
            "0"
        );
        assert_eq!(part2(r#"{"shade":["red"],"weight":17}"#).unwrap(), "17");
        assert_eq!(
            part2(r#"{"shade":{"value":"red"},"weight":17}"#).unwrap(),
            "17"
        );
        assert_eq!(part2(r#"{"shade":"\u0072ed","weight":17}"#).unwrap(), "0");
        assert_eq!(part2(r#"{"shade":"Red","weight":17}"#).unwrap(), "17");
        assert_eq!(part2(r#"{"red":17}"#).unwrap(), "17");
    }

    #[test]
    fn generated_nested_documents_match_constructed_totals() {
        for a in -8..=8 {
            for b in -8..=8 {
                for red in [false, true] {
                    let shade = if red { "red" } else { "green" };
                    let input = format!(
                        r#"{{"red":{a},"branch":{{"shade":"{shade}","weights":[{b},{a}]}},"array":["red",{b},null,true]}}"#
                    );
                    assert_eq!(part1(&input).unwrap(), (2 * a + 2 * b).to_string());
                    let expected = if red { a + b } else { 2 * a + 2 * b };
                    assert_eq!(part2(&input).unwrap(), expected.to_string());
                    let wrapped = format!(r#"{{"shade":"red","payload":{input}}}"#);
                    assert_eq!(part1(&wrapped).unwrap(), (2 * a + 2 * b).to_string());
                    assert_eq!(part2(&wrapped).unwrap(), "0");
                }
            }
        }
    }

    #[test]
    fn scalar_types_and_integer_boundaries() {
        for solver in [part1, part2] {
            for input in ["null", "true", "false", r#""red""#, r#""765""#, "[]", "{}"] {
                assert_eq!(solver(input).unwrap(), "0");
            }
            for number in [i64::MIN, -37, 0, 83, i64::MAX] {
                assert_eq!(
                    solver(&format!(" \r\n{number}\t ")).unwrap(),
                    number.to_string()
                );
            }
            for input in [
                "0.5",
                "1.0",
                "2e3",
                "9223372036854775808",
                "-9223372036854775809",
            ] {
                assert!(solver(input).is_err());
            }
            for input in [
                "[9223372036854775807,4]",
                "[-9223372036854775808,-4]",
                r#"{"a":9223372036854775807,"b":4}"#,
            ] {
                assert!(matches!(solver(input), Err(SolveError::ArithmeticOverflow)));
            }
        }
        assert_eq!(part2(r#"{"shade":"red","ignored":0.5}"#).unwrap(), "0");
        assert_eq!(
            part2(r#"{"shade":"red","ignored":[9223372036854775807,4]}"#).unwrap(),
            "0"
        );
    }

    #[test]
    fn malformed_and_excessively_deep_documents_return_errors() {
        for solver in [part1, part2] {
            for input in ["", " \r\n"] {
                assert!(matches!(solver(input), Err(SolveError::EmptyInput)));
            }
            for input in [
                "[",
                "{",
                "[4,]",
                "{oops}",
                "17 trailing",
                "null null",
                "NaN",
                "01",
            ] {
                assert!(solver(input).is_err());
            }
            let shallow = format!("{}17{}", "[".repeat(30), "]".repeat(30));
            assert_eq!(solver(&shallow).unwrap(), "17");
            let deep = format!("{}17{}", "[".repeat(200), "]".repeat(200));
            assert!(solver(&deep)
                .unwrap_err()
                .to_string()
                .contains("recursion limit"));
            let malformed = "{\n\"weight\": ]\n}";
            assert!(solver(malformed)
                .unwrap_err()
                .to_string()
                .contains("line 2"));
        }
    }

    #[test]
    fn test_part1_input() {
        scaffold_test_result(2015, 12, "input.json", "input-spec.1.txt", part1);
    }
    #[test]
    fn test_part2_input() {
        scaffold_test_result(2015, 12, "input.json", "input-spec.2.txt", part2);
    }
}
