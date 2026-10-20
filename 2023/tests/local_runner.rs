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
    let output = Command::new(env!("CARGO_BIN_EXE_aoc"))
        .arg("1")
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
