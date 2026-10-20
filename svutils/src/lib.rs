use std::fs::File;
use std::io::{self, Read};

mod error;
pub use error::SolveError;

pub type Solver = fn(&str) -> Result<String, SolveError>;

pub fn scaffold_test_result(year: i16, day: i16, input: &str, spec: &str, func: Solver) {
    let input = load_file(year, day, input).expect("could not read input fixture");
    let expected = load_file(year, day, spec).expect("could not read expected-answer fixture");
    let answer = func(&input).expect("solver failed");
    assert_eq!(answer, expected.trim());
}

pub fn parse_spec(input: &str) -> i64 {
    input.parse::<i64>().unwrap_or_default()
}

pub fn load_file(year: i16, day: i16, name: &str) -> io::Result<String> {
    let file_path = format!("../input/{year}/d{day:02}/{name}");
    let mut file = File::open(file_path)?;
    let mut content = String::new();
    file.read_to_string(&mut content)?;

    Ok(content)
}

pub fn scaffold_test(year: i16, day: i16, input: &str, spec: &str, func: impl Fn(&str) -> i64) {
    let input_data = load_file(year, day, input);
    let spec_data = load_file(year, day, spec);

    assert!(input_data.is_ok());
    assert!(spec_data.is_ok());

    assert_eq!(
        func(input_data.unwrap().as_str()),
        parse_spec(spec_data.unwrap().as_str())
    );
}

pub fn scaffold_test_string(
    year: i16,
    day: i16,
    input: &str,
    spec: &str,
    func: impl Fn(&str) -> String,
) {
    let input_data = load_file(year, day, input);
    let spec_data = load_file(year, day, spec);

    assert!(input_data.is_ok());
    assert!(spec_data.is_ok());

    assert_eq!(
        func(input_data.unwrap().as_str()),
        spec_data.unwrap().to_string()
    );
}

pub fn scaffold_test_wide(
    year: i16,
    day: i16,
    input: &str,
    spec: &str,
    func: impl Fn(&str) -> i128,
) {
    let input_data = load_file(year, day, input);
    let spec_data = load_file(year, day, spec);

    assert!(input_data.is_ok());
    assert!(spec_data.is_ok());

    assert_eq!(
        func(input_data.unwrap().as_str()),
        parse_spec(spec_data.unwrap().as_str()) as i128
    );
}
