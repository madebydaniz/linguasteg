use std::io::Write;
use std::process::{Command, Output, Stdio};

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

fn run_lsteg_with_env(args: &[&str], envs: &[(&str, &str)]) -> Output {
    let mut command = base_lsteg_command();
    command.args(args);
    for (key, value) in envs {
        command.env(key, value);
    }
    command.output().expect("failed to run lsteg with env")
}

fn run_lsteg_with_stdin(args: &[&str], stdin: &str) -> Output {
    let mut child = base_lsteg_command()
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to spawn lsteg");

    let mut child_stdin = child.stdin.take().expect("stdin should be piped");
    child_stdin
        .write_all(stdin.as_bytes())
        .expect("failed to write stdin");
    drop(child_stdin);

    child.wait_with_output().expect("failed to wait for lsteg")
}

fn stdout_string(output: &Output) -> String {
    String::from_utf8(output.stdout.clone()).expect("stdout must be valid utf8")
}

fn stderr_string(output: &Output) -> String {
    String::from_utf8(output.stderr.clone()).expect("stderr must be valid utf8")
}

#[test]
fn encode_json_outputs_proto_encode_mode() {
    let output = run_lsteg(&["encode", "--message", "salam", "--format", "json"]);
    assert!(output.status.success());

    let stdout = stdout_string(&output);
    assert!(stdout.contains("\"mode\":\"proto-encode\""));
    assert!(stdout.contains("\"language\":\"fa\""));
    assert!(stdout.contains("\"payload_bytes\":5"));
}

#[test]
fn decode_roundtrip_from_encode_trace_works() {
    let encode_output = run_lsteg(&["encode", "--message", "salam"]);
    assert!(encode_output.status.success());
    let trace_text = stdout_string(&encode_output);

    let decode_output = run_lsteg_with_stdin(&["decode", "--format", "json"], &trace_text);
    assert!(decode_output.status.success());

    let decoded_json = stdout_string(&decode_output);
    assert!(decoded_json.contains("\"mode\":\"proto-decode\""));
    assert!(decoded_json.contains("\"payload_utf8\":\"salam\""));
}

#[test]
fn analyze_from_trace_reports_integrity_ok() {
    let encode_output = run_lsteg(&["encode", "--message", "salam donya"]);
    assert!(encode_output.status.success());
    let trace_text = stdout_string(&encode_output);

    let analyze_output = run_lsteg_with_stdin(&["analyze", "--format", "json"], &trace_text);
    assert!(analyze_output.status.success());

    let analysis_json = stdout_string(&analyze_output);
    assert!(analysis_json.contains("\"mode\":\"analyze\""));
    assert!(analysis_json.contains("\"integrity_ok\":true"));
    assert!(analysis_json.contains("\"payload_utf8\":\"salam donya\""));
}

#[test]
fn parse_errors_return_exit_code_two() {
    let output = run_lsteg(&["analyze", "--unknown"]);
    assert_eq!(output.status.code(), Some(2));
}

#[test]
fn runtime_errors_return_exit_code_one() {
    let output = run_lsteg_with_stdin(&["decode"], "");
    assert_eq!(output.status.code(), Some(1));
}

#[test]
fn decode_rejects_non_contiguous_trace_ranges() {
    let encode_output = run_lsteg(&["encode", "--message", "salam"]);
    assert!(encode_output.status.success());
    let trace_text = stdout_string(&encode_output);
    let broken_trace = trace_text.replacen("bits=18..39", "bits=19..40", 1);

    let decode_output = run_lsteg_with_stdin(&["decode", "--format", "json"], &broken_trace);
    assert_eq!(decode_output.status.code(), Some(1));

    let stderr = stderr_string(&decode_output);
    assert!(stderr.contains("invalid trace frame sequence"));
}

#[test]
fn analyze_reports_integrity_failure_for_non_contiguous_trace() {
    let encode_output = run_lsteg(&["encode", "--message", "salam"]);
    assert!(encode_output.status.success());
    let trace_text = stdout_string(&encode_output);
    let broken_trace = trace_text.replacen("bits=18..39", "bits=19..40", 1);

    let analyze_output = run_lsteg_with_stdin(&["analyze", "--format", "json"], &broken_trace);
    assert!(analyze_output.status.success());

    let analysis_json = stdout_string(&analyze_output);
    assert!(analysis_json.contains("\"integrity_ok\":false"));
    assert!(analysis_json.contains("frame 02 starts at bit 19 but expected 18"));
}

#[test]
fn encode_uses_env_defaults_when_cli_values_missing() {
    let output = run_lsteg_with_env(
        &["encode"],
        &[("LSTEG_ENCODE_MESSAGE", "salam"), ("LSTEG_FORMAT", "json")],
    );
    assert!(output.status.success());

    let stdout = stdout_string(&output);
    assert!(stdout.contains("\"mode\":\"proto-encode\""));
    assert!(stdout.contains("\"input_text\":\"salam\""));
    assert!(stdout.contains("\"payload_bytes\":5"));
}

#[test]
fn cli_flags_override_env_defaults() {
    let output = run_lsteg_with_env(
        &["encode", "--message", "override", "--format", "json"],
        &[
            ("LSTEG_ENCODE_MESSAGE", "from-env"),
            ("LSTEG_FORMAT", "text"),
        ],
    );
    assert!(output.status.success());

    let stdout = stdout_string(&output);
    assert!(stdout.contains("\"input_text\":\"override\""));
}

#[test]
fn invalid_env_format_returns_config_error() {
    let output = run_lsteg_with_env(
        &["encode", "--message", "salam"],
        &[("LSTEG_FORMAT", "xml")],
    );
    assert_eq!(output.status.code(), Some(1));
    let stderr = stderr_string(&output);
    assert!(stderr.contains("LSTEG-CLI-CFG-001"));
}

#[test]
fn decode_uses_env_trace_when_stdin_missing() {
    let encode_output = run_lsteg(&["encode", "--message", "salam"]);
    assert!(encode_output.status.success());
    let trace_text = stdout_string(&encode_output);

    let decode_output = run_lsteg_with_env(
        &["decode", "--format", "json"],
        &[("LSTEG_TRACE", &trace_text)],
    );
    assert!(decode_output.status.success());
    let decoded_json = stdout_string(&decode_output);
    assert!(decoded_json.contains("\"payload_utf8\":\"salam\""));
}

#[test]
fn cli_trace_overrides_env_trace() {
    let good_trace = stdout_string(&run_lsteg(&["encode", "--message", "salam"]));
    let bad_trace = stdout_string(&run_lsteg(&["encode", "--message", "kharab"]));

    let decode_output = run_lsteg_with_env(
        &["decode", "--trace", &good_trace, "--format", "json"],
        &[("LSTEG_TRACE", &bad_trace)],
    );
    assert!(decode_output.status.success());
    let decoded_json = stdout_string(&decode_output);
    assert!(decoded_json.contains("\"payload_utf8\":\"salam\""));
}

#[test]
fn encode_fails_without_secret() {
    let output = run_lsteg_with_env(&["encode", "--message", "salam"], &[("LSTEG_SECRET", "")]);
    assert_eq!(output.status.code(), Some(1));
    let stderr = stderr_string(&output);
    assert!(stderr.contains("LSTEG-CLI-CFG-001"));
}

#[test]
fn decode_fails_with_wrong_secret() {
    let encode_output = run_lsteg(&["encode", "--message", "salam"]);
    assert!(encode_output.status.success());
    let trace_text = stdout_string(&encode_output);

    let decode_output = run_lsteg_with_env(
        &["decode", "--format", "json"],
        &[
            ("LSTEG_TRACE", &trace_text),
            ("LSTEG_SECRET", "wrong-secret"),
        ],
    );
    assert_eq!(decode_output.status.code(), Some(1));
    let stderr = stderr_string(&decode_output);
    assert!(stderr.contains("failed to decrypt payload"));
}

#[test]
fn analyze_without_secret_reports_structural_only() {
    let encode_output = run_lsteg(&["encode", "--message", "salam"]);
    assert!(encode_output.status.success());
    let trace_text = stdout_string(&encode_output);

    let analyze_output = run_lsteg_with_env(
        &["analyze", "--format", "json"],
        &[("LSTEG_TRACE", &trace_text), ("LSTEG_SECRET", "")],
    );
    assert!(analyze_output.status.success());
    let analysis_json = stdout_string(&analyze_output);
    assert!(analysis_json.contains("\"integrity_ok\":true"));
    assert!(analysis_json.contains("\"payload_bytes\":null"));
    assert!(analysis_json.contains("\"payload_utf8\":null"));
}
