use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};

const ENV_KEYS: [&str; 6] = [
    "LSTEG_LANG",
    "LSTEG_FORMAT",
    "LSTEG_INPUT",
    "LSTEG_OUTPUT",
    "LSTEG_ENCODE_MESSAGE",
    "LSTEG_TRACE",
];

fn base_lsteg_command() -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_lsteg"));
    for key in ENV_KEYS {
        command.env_remove(key);
    }
    command
}

fn run_lsteg(args: &[&str]) -> Output {
    base_lsteg_command()
        .args(args)
        .output()
        .expect("failed to run lsteg")
}

fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(name)
}

fn fixture_contents(name: &str) -> String {
    fs::read_to_string(fixture_path(name)).expect("failed to read fixture")
}

fn stdout_string(output: &Output) -> String {
    String::from_utf8(output.stdout.clone()).expect("stdout must be valid utf8")
}

#[test]
fn encode_text_matches_golden_fixture() {
    let output = run_lsteg(&["encode", "--message", "salam"]);
    assert!(output.status.success());
    assert_eq!(
        stdout_string(&output),
        fixture_contents("encode_salam_text.out")
    );
}

#[test]
fn encode_json_matches_golden_fixture() {
    let output = run_lsteg(&["encode", "--message", "salam", "--format", "json"]);
    assert!(output.status.success());
    assert_eq!(
        stdout_string(&output),
        fixture_contents("encode_salam_json.out")
    );
}

#[test]
fn decode_json_matches_golden_fixture() {
    let input = fixture_path("encode_salam_text.out");
    let input = input.to_string_lossy().into_owned();

    let output = run_lsteg(&["decode", "--format", "json", "--input", &input]);
    assert!(output.status.success());
    assert_eq!(
        stdout_string(&output),
        fixture_contents("decode_salam_json.out")
    );
}

#[test]
fn analyze_json_matches_golden_fixture() {
    let input = fixture_path("encode_salam_text.out");
    let input = input.to_string_lossy().into_owned();

    let output = run_lsteg(&["analyze", "--format", "json", "--input", &input]);
    assert!(output.status.success());
    assert_eq!(
        stdout_string(&output),
        fixture_contents("analyze_salam_json.out")
    );
}
