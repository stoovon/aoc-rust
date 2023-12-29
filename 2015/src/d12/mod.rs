extern crate core;

use serde_json::Value;

pub fn fn1(input: &str) -> i64 {
    let v: Value = serde_json::from_str(input).unwrap();

    fn sum(v: &Value) -> i64 {
        match v {
            Value::Null => 0,
            Value::Bool(_) => 0,
            Value::Number(n) => n.as_i64().unwrap(),
            Value::String(_) => 0,
            Value::Array(a) => a.iter().map(sum).sum(),
            Value::Object(o) => o.values().map(sum).sum(),
        }
    }

    sum(&v)
}

pub fn fn2(input: &str) -> i64 {
    let v: Value = serde_json::from_str(input).unwrap();

    // Sum values in input as per fn1, but ignore any object that has a red property
    fn sum(v: &Value) -> i64 {
        match v {
            Value::Null => 0,
            Value::Bool(_) => 0,
            Value::Number(n) => n.as_i64().unwrap(),
            Value::String(_) => 0,
            Value::Array(a) => a.iter().map(sum).sum(),
            Value::Object(o) => {
                if o.values().any(|v| v == "red") {
                    0
                } else {
                    o.values().map(sum).sum()
                }
            }
        }
    }

    sum(&v)
}

#[cfg(test)]
mod tests {
    use super::*;
    use svutils::scaffold_test;

    const YEAR: i16 = 2015;
    const DAY: i16 = 12;

    #[test]
    fn test_fn1_example() {
        scaffold_test(YEAR, DAY, "example.1.txt", "example-spec.1.txt", fn1);
    }

    #[test]
    fn test_fn1_input() {
        scaffold_test(YEAR, DAY, "input.json", "input-spec.1.txt", fn1);
    }

    #[test]
    fn test_fn2_example() {
        scaffold_test(YEAR, DAY, "example.2.txt", "example-spec.2.txt", fn2);
    }

    #[test]
    fn test_fn2_input() {
        scaffold_test(YEAR, DAY, "input.json", "input-spec.2.txt", fn2);
    }
}
