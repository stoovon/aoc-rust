use std::{
    fs,
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

#[test]
fn solver_failure_exits_unsuccessfully_without_printing_an_answer() {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let path = std::env::temp_dir().join(format!("aoc-runner-{}-{unique}.txt", std::process::id()));
    fs::write(&path, format!("{}*", "9".repeat(30))).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_aoc-runner"))
        .args(["2023", "3", "1"])
        .arg(&path)
        .output()
        .unwrap();
    fs::remove_file(path).unwrap();
    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    assert!(String::from_utf8(output.stderr)
        .unwrap()
        .contains("could not parse puzzle number"));
}

#[test]
fn unknown_solver_is_rejected_before_reading_input() {
    let output = Command::new(env!("CARGO_BIN_EXE_aoc-runner"))
        .args(["2023", "2", "1", "nonexistent-input"])
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    assert!(String::from_utf8(output.stderr)
        .unwrap()
        .contains("no solver registered"));
}

#[test]
fn missing_calibration_digit_exits_without_printing_an_answer() {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let path = std::env::temp_dir().join(format!("aoc-day1-{}-{unique}.txt", std::process::id()));
    fs::write(&path, "letters").unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_aoc-runner"))
        .args(["2023", "1", "2"])
        .arg(&path)
        .output()
        .unwrap();
    fs::remove_file(path).unwrap();
    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    assert!(String::from_utf8(output.stderr)
        .unwrap()
        .contains("line 1 contains no recognised digit"));
}
