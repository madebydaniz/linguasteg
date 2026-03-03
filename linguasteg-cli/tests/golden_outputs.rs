use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};

const TEST_SECRET: &str = "linguasteg-test-secret";
const ENV_KEYS: [&str; 7] = [
    "LSTEG_LANG",
    "LSTEG_FORMAT",
    "LSTEG_INPUT",
    "LSTEG_OUTPUT",
    "LSTEG_ENCODE_MESSAGE",
    "LSTEG_TRACE",
    "LSTEG_SECRET",
];

fn base_lsteg_command() -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_lsteg"));
    for key in ENV_KEYS {
        command.env_remove(key);
    }
    command.env("LSTEG_SECRET", TEST_SECRET);
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

fn stderr_string(output: &Output) -> String {
    String::from_utf8(output.stderr.clone()).expect("stderr must be valid utf8")
}

#[test]
fn encode_text_contains_expected_sections() {
    let output = run_lsteg(&["encode", "--message", "salam"]);
    assert!(output.status.success());
    let stdout = stdout_string(&output);
    assert!(stdout.contains("Farsi prototype encode"));
    assert!(stdout.contains("input text: salam"));
    assert!(stdout.contains("payload bytes: 5"));
    assert!(stdout.contains("gateway response: stub:encode:fa:symbolic-stub:salam"));
}

#[test]
fn encode_json_contains_expected_contract() {
    let output = run_lsteg(&["encode", "--message", "salam", "--format", "json"]);
    assert!(output.status.success());
    let stdout = stdout_string(&output);
    assert!(stdout.contains("\"mode\":\"proto-encode\""));
    assert!(stdout.contains("\"language\":\"fa\""));
    assert!(stdout.contains("\"input_text\":\"salam\""));
    assert!(stdout.contains("\"payload_bytes\":5"));
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
fn decode_english_json_matches_golden_fixture() {
    let input = fixture_path("encode_hello_en_text.out");
    let input = input.to_string_lossy().into_owned();

    let output = run_lsteg(&[
        "decode", "--lang", "en", "--format", "json", "--input", &input,
    ]);
    assert!(output.status.success());
    assert_eq!(
        stdout_string(&output),
        fixture_contents("decode_hello_en_json.out")
    );
}

#[test]
fn analyze_english_json_matches_golden_fixture() {
    let input = fixture_path("encode_hello_en_text.out");
    let input = input.to_string_lossy().into_owned();

    let output = run_lsteg(&[
        "analyze", "--lang", "en", "--format", "json", "--input", &input,
    ]);
    assert!(output.status.success());
    assert_eq!(
        stdout_string(&output),
        fixture_contents("analyze_hello_en_json.out")
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

#[test]
fn encode_json_unicode_mix_contains_expected_contract() {
    let output = run_lsteg(&["encode", "--message", "salam دنیا 123", "--format", "json"]);
    assert!(output.status.success());
    let stdout = stdout_string(&output);
    assert!(stdout.contains("\"input_text\":\"salam دنیا 123\""));
    assert!(stdout.contains("\"payload_bytes\":18"));
}

#[test]
fn encode_json_preserves_whitespace_in_input_text() {
    let output = run_lsteg(&[
        "encode",
        "--message",
        "  salam   donya  ",
        "--format",
        "json",
    ]);
    assert!(output.status.success());
    let stdout = stdout_string(&output);
    assert!(stdout.contains("\"input_text\":\"  salam   donya  \""));
    assert!(stdout.contains("\"payload_bytes\":17"));
}

#[test]
fn analyze_non_contiguous_trace_matches_golden_fixture() {
    let input = fixture_path("trace_salam_non_contiguous.input");
    let input = input.to_string_lossy().into_owned();

    let output = run_lsteg(&["analyze", "--format", "json", "--input", &input]);
    assert!(output.status.success());
    assert_eq!(
        stdout_string(&output),
        fixture_contents("analyze_non_contiguous_json.out")
    );
}

#[test]
fn decode_non_contiguous_trace_stderr_matches_golden_fixture() {
    let input = fixture_path("trace_salam_non_contiguous.input");
    let input = input.to_string_lossy().into_owned();

    let output = run_lsteg(&["decode", "--format", "json", "--input", &input]);
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(
        stderr_string(&output),
        fixture_contents("decode_non_contiguous_stderr.out")
    );
}

#[test]
fn templates_json_matches_golden_fixture() {
    let output = run_lsteg(&["templates", "--format", "json"]);
    assert!(output.status.success());
    assert_eq!(
        stdout_string(&output),
        fixture_contents("templates_json.out")
    );
}

#[test]
fn profiles_json_matches_golden_fixture() {
    let output = run_lsteg(&["profiles", "--format", "json"]);
    assert!(output.status.success());
    assert_eq!(
        stdout_string(&output),
        fixture_contents("profiles_json.out")
    );
}

#[test]
fn catalog_json_matches_golden_fixture() {
    let output = run_lsteg(&["catalog", "--format", "json"]);
    assert!(output.status.success());
    assert_eq!(stdout_string(&output), fixture_contents("catalog_json.out"));
}

#[test]
fn catalog_en_json_matches_golden_fixture() {
    let output = run_lsteg(&["catalog", "--lang", "en", "--format", "json"]);
    assert!(output.status.success());
    assert_eq!(
        stdout_string(&output),
        fixture_contents("catalog_en_json.out")
    );
}

#[test]
fn validate_json_matches_golden_fixture() {
    let input = fixture_path("encode_salam_text.out");
    let input = input.to_string_lossy().into_owned();

    let output = run_lsteg(&["validate", "--format", "json", "--input", &input]);
    assert!(output.status.success());
    assert_eq!(
        stdout_string(&output),
        fixture_contents("validate_salam_json.out")
    );
}

#[test]
fn validate_non_contiguous_trace_matches_golden_fixtures() {
    let input = fixture_path("trace_salam_non_contiguous.input");
    let input = input.to_string_lossy().into_owned();

    let output = run_lsteg(&["validate", "--format", "json", "--input", &input]);
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(
        stdout_string(&output),
        fixture_contents("validate_non_contiguous_stdout.out")
    );
    assert_eq!(
        stderr_string(&output),
        fixture_contents("validate_non_contiguous_stderr.out")
    );
}

#[test]
fn schemas_json_matches_golden_fixture() {
    let output = run_lsteg(&["schemas", "--format", "json"]);
    assert!(output.status.success());
    assert_eq!(stdout_string(&output), fixture_contents("schemas_json.out"));
}

#[test]
fn schemas_en_json_matches_golden_fixture() {
    let output = run_lsteg(&["schemas", "--lang", "en", "--format", "json"]);
    assert!(output.status.success());
    assert_eq!(
        stdout_string(&output),
        fixture_contents("schemas_en_json.out")
    );
}
