use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Output, Stdio};
use std::time::{SystemTime, UNIX_EPOCH};

const TEST_SECRET: &str = "linguasteg-test-secret";
const ENV_KEYS: [&str; 8] = [
    "LSTEG_LANG",
    "LSTEG_FORMAT",
    "LSTEG_INPUT",
    "LSTEG_OUTPUT",
    "LSTEG_ENCODE_MESSAGE",
    "LSTEG_PROFILE",
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

struct TempSecretFile {
    path: PathBuf,
}

impl TempSecretFile {
    fn create(secret: &str) -> Self {
        let mut path = std::env::temp_dir();
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock should be after unix epoch")
            .as_nanos();
        path.push(format!("linguasteg-secret-{nanos}.txt"));
        std::fs::write(&path, secret).expect("failed to write temp secret file");
        Self { path }
    }

    fn as_str(&self) -> &str {
        self.path
            .to_str()
            .expect("temp secret file path must be valid utf8")
    }
}

impl Drop for TempSecretFile {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
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
fn encode_json_reports_null_style_profile_when_not_set() {
    let output = run_lsteg(&["encode", "--message", "salam", "--format", "json"]);
    assert!(output.status.success());

    let stdout = stdout_string(&output);
    assert!(stdout.contains("\"style_profile\":null"));
}

#[test]
fn encode_json_reports_style_profile_when_set() {
    let output = run_lsteg(&[
        "encode",
        "--lang",
        "fa",
        "--message",
        "salam",
        "--profile",
        "fa-saadi-inspired-light",
        "--format",
        "json",
    ]);
    assert!(output.status.success());

    let stdout = stdout_string(&output);
    assert!(stdout.contains("\"style_profile\":\"fa-saadi-inspired-light\""));
}

#[test]
fn encode_json_supports_english_target() {
    let output = run_lsteg(&[
        "encode",
        "--lang",
        "en",
        "--message",
        "hello",
        "--format",
        "json",
    ]);
    assert!(output.status.success());

    let stdout = stdout_string(&output);
    assert!(stdout.contains("\"mode\":\"proto-encode\""));
    assert!(stdout.contains("\"language\":\"en\""));
    assert!(stdout.contains("\"input_text\":\"hello\""));
}

#[test]
fn languages_text_lists_supported_languages() {
    let output = run_lsteg(&["languages"]);
    assert!(output.status.success());

    let stdout = stdout_string(&output);
    assert!(stdout.contains("supported languages:"));
    assert!(stdout.contains("- fa (Farsi, rtl)"));
    assert!(stdout.contains("- en (English, ltr)"));
}

#[test]
fn languages_json_exposes_contract() {
    let output = run_lsteg(&["languages", "--format", "json"]);
    assert!(output.status.success());

    let stdout = stdout_string(&output);
    assert!(stdout.contains("\"mode\":\"languages\""));
    assert!(stdout.contains("\"code\":\"fa\""));
    assert!(stdout.contains("\"direction\":\"rtl\""));
    assert!(stdout.contains("\"code\":\"en\""));
    assert!(stdout.contains("\"direction\":\"ltr\""));
}

#[test]
fn strategies_text_lists_supported_strategies() {
    let output = run_lsteg(&["strategies"]);
    assert!(output.status.success());

    let stdout = stdout_string(&output);
    assert!(stdout.contains("supported strategies:"));
    assert!(stdout.contains("- symbolic-stub (Symbolic Stub) capabilities: deterministic-seed"));
}

#[test]
fn strategies_json_exposes_contract() {
    let output = run_lsteg(&["strategies", "--format", "json"]);
    assert!(output.status.success());

    let stdout = stdout_string(&output);
    assert!(stdout.contains("\"mode\":\"strategies\""));
    assert!(stdout.contains("\"id\":\"symbolic-stub\""));
    assert!(stdout.contains("\"display\":\"Symbolic Stub\""));
    assert!(stdout.contains("\"required_capabilities\":[\"deterministic-seed\"]"));
}

#[test]
fn models_text_lists_supported_models() {
    let output = run_lsteg(&["models"]);
    assert!(output.status.success());

    let stdout = stdout_string(&output);
    assert!(stdout.contains("supported models:"));
    assert!(stdout.contains(
        "- stub/stub-local (Stub Local) languages: fa,en capabilities: deterministic-seed"
    ));
}

#[test]
fn models_json_exposes_contract() {
    let output = run_lsteg(&["models", "--format", "json"]);
    assert!(output.status.success());

    let stdout = stdout_string(&output);
    assert!(stdout.contains("\"mode\":\"models\""));
    assert!(stdout.contains("\"provider\":\"stub\""));
    assert!(stdout.contains("\"id\":\"stub-local\""));
    assert!(stdout.contains("\"display\":\"Stub Local\""));
    assert!(stdout.contains("\"languages\":[\"fa\",\"en\"]"));
    assert!(stdout.contains("\"capabilities\":[\"deterministic-seed\"]"));
}

#[test]
fn catalog_text_contains_all_sections() {
    let output = run_lsteg(&["catalog"]);
    assert!(output.status.success());

    let stdout = stdout_string(&output);
    assert!(stdout.contains("catalog:"));
    assert!(stdout.contains("languages:"));
    assert!(stdout.contains("strategies:"));
    assert!(stdout.contains("models:"));
    assert!(stdout.contains("templates:"));
    assert!(stdout.contains("profiles:"));
    assert!(stdout.contains("schemas:"));
    assert!(stdout.contains("- fa (Farsi, rtl)"));
    assert!(stdout.contains("- symbolic-stub (Symbolic Stub) capabilities: deterministic-seed"));
    assert!(stdout.contains(
        "- stub/stub-local (Stub Local) languages: fa,en capabilities: deterministic-seed"
    ));
    assert!(stdout.contains("- fa/fa-basic-sov (Basic SOV) slots: 4"));
    assert!(stdout.contains(
        "- en/en-neutral-prototype (Neutral English Prototype) register: neutral strength: light inspiration: register-only (<none>)"
    ));
    assert!(stdout.contains("- en/en-basic-svo total_bits: 18"));
}

#[test]
fn catalog_json_exposes_all_sections() {
    let output = run_lsteg(&["catalog", "--format", "json"]);
    assert!(output.status.success());

    let stdout = stdout_string(&output);
    assert!(stdout.contains("\"mode\":\"catalog\""));
    assert!(stdout.contains("\"languages\":["));
    assert!(stdout.contains("\"strategies\":["));
    assert!(stdout.contains("\"models\":["));
    assert!(stdout.contains("\"templates\":["));
    assert!(stdout.contains("\"profiles\":["));
    assert!(stdout.contains("\"schemas\":["));
    assert!(stdout.contains("\"code\":\"fa\""));
    assert!(stdout.contains("\"id\":\"symbolic-stub\""));
    assert!(stdout.contains("\"provider\":\"stub\""));
    assert!(stdout.contains("\"id\":\"fa-basic-sov\""));
    assert!(stdout.contains("\"id\":\"fa-neutral-formal\""));
    assert!(stdout.contains("\"template_id\":\"fa-basic-sov\""));
}

#[test]
fn catalog_lang_filter_limits_language_scoped_sections() {
    let output = run_lsteg(&["catalog", "--lang", "en", "--format", "json"]);
    assert!(output.status.success());

    let stdout = stdout_string(&output);
    assert!(stdout.contains("\"mode\":\"catalog\""));
    assert!(stdout.contains("\"code\":\"en\""));
    assert!(!stdout.contains("\"code\":\"fa\""));
    assert!(stdout.contains("\"id\":\"en-basic-svo\""));
    assert!(!stdout.contains("\"id\":\"fa-basic-sov\""));
    assert!(stdout.contains("\"id\":\"en-neutral-prototype\""));
    assert!(!stdout.contains("\"id\":\"fa-neutral-formal\""));
    assert!(stdout.contains("\"template_id\":\"en-basic-svo\""));
    assert!(!stdout.contains("\"template_id\":\"fa-basic-sov\""));
}

#[test]
fn templates_text_lists_supported_templates() {
    let output = run_lsteg(&["templates"]);
    assert!(output.status.success());

    let stdout = stdout_string(&output);
    assert!(stdout.contains("supported templates:"));
    assert!(stdout.contains("- fa/fa-basic-sov (Basic SOV) slots: 4"));
    assert!(stdout.contains("- fa/fa-time-location-sov (Time + Location + SOV) slots: 5"));
    assert!(stdout.contains("- en/en-basic-svo (English Basic SVO) slots: 4"));
    assert!(stdout.contains("- en/en-time-location-svo (English Time Location SVO) slots: 5"));
}

#[test]
fn templates_json_exposes_contract() {
    let output = run_lsteg(&["templates", "--format", "json"]);
    assert!(output.status.success());

    let stdout = stdout_string(&output);
    assert!(stdout.contains("\"mode\":\"templates\""));
    assert!(stdout.contains("\"language\":\"fa\""));
    assert!(stdout.contains("\"language_display\":\"Farsi\""));
    assert!(stdout.contains("\"id\":\"fa-basic-sov\""));
    assert!(stdout.contains("\"id\":\"en-basic-svo\""));
    assert!(stdout.contains("\"slot_count\":4"));
}

#[test]
fn templates_lang_filter_limits_output() {
    let output = run_lsteg(&["templates", "--lang", "en", "--format", "json"]);
    assert!(output.status.success());

    let stdout = stdout_string(&output);
    assert!(stdout.contains("\"mode\":\"templates\""));
    assert!(stdout.contains("\"language\":\"en\""));
    assert!(stdout.contains("\"id\":\"en-basic-svo\""));
    assert!(!stdout.contains("\"id\":\"fa-basic-sov\""));
}

#[test]
fn profiles_text_lists_supported_profiles() {
    let output = run_lsteg(&["profiles"]);
    assert!(output.status.success());

    let stdout = stdout_string(&output);
    assert!(stdout.contains("supported profiles:"));
    assert!(stdout.contains("- fa/fa-neutral-formal (Formal Persian (Neutral))"));
    assert!(stdout.contains("- fa/fa-saadi-inspired-light (Saadi-inspired (Light))"));
    assert!(stdout.contains("- en/en-neutral-prototype (Neutral English Prototype)"));
}

#[test]
fn profiles_json_exposes_contract() {
    let output = run_lsteg(&["profiles", "--format", "json"]);
    assert!(output.status.success());

    let stdout = stdout_string(&output);
    assert!(stdout.contains("\"mode\":\"profiles\""));
    assert!(stdout.contains("\"language\":\"fa\""));
    assert!(stdout.contains("\"id\":\"fa-saadi-inspired-light\""));
    assert!(stdout.contains("\"inspiration_kind\":\"author-inspired\""));
    assert!(stdout.contains("\"inspiration_label\":\"Saadi\""));
    assert!(stdout.contains("\"language\":\"en\""));
}

#[test]
fn profiles_lang_filter_limits_output() {
    let output = run_lsteg(&["profiles", "--lang", "en", "--format", "json"]);
    assert!(output.status.success());

    let stdout = stdout_string(&output);
    assert!(stdout.contains("\"mode\":\"profiles\""));
    assert!(stdout.contains("\"language\":\"en\""));
    assert!(stdout.contains("\"id\":\"en-neutral-prototype\""));
    assert!(!stdout.contains("\"id\":\"fa-neutral-formal\""));
}

#[test]
fn schemas_text_lists_supported_schemas() {
    let output = run_lsteg(&["schemas"]);
    assert!(output.status.success());

    let stdout = stdout_string(&output);
    assert!(stdout.contains("supported schemas:"));
    assert!(stdout.contains("- fa/fa-basic-sov total_bits: 18"));
    assert!(stdout.contains("- en/en-time-location-svo total_bits: 21"));
}

#[test]
fn schemas_json_exposes_contract() {
    let output = run_lsteg(&["schemas", "--format", "json"]);
    assert!(output.status.success());

    let stdout = stdout_string(&output);
    assert!(stdout.contains("\"mode\":\"schemas\""));
    assert!(stdout.contains("\"language\":\"fa\""));
    assert!(stdout.contains("\"template_id\":\"fa-basic-sov\""));
    assert!(stdout.contains("\"slot\":\"subject\""));
    assert!(stdout.contains("\"bit_width\":5"));
}

#[test]
fn schemas_lang_filter_limits_output() {
    let output = run_lsteg(&["schemas", "--lang", "en", "--format", "json"]);
    assert!(output.status.success());

    let stdout = stdout_string(&output);
    assert!(stdout.contains("\"mode\":\"schemas\""));
    assert!(stdout.contains("\"language\":\"en\""));
    assert!(stdout.contains("\"template_id\":\"en-basic-svo\""));
    assert!(!stdout.contains("\"template_id\":\"fa-basic-sov\""));
}

#[test]
fn validate_json_reports_integrity_ok() {
    let encode_output = run_lsteg(&["encode", "--message", "salam", "--emit-trace"]);
    assert!(encode_output.status.success());
    let trace_text = stdout_string(&encode_output);

    let validate_output = run_lsteg_with_stdin(&["validate", "--format", "json"], &trace_text);
    assert!(validate_output.status.success());
    let stdout = stdout_string(&validate_output);
    assert!(stdout.contains("\"mode\":\"validate\""));
    assert!(stdout.contains("\"language\":\"fa\""));
    assert!(stdout.contains("\"integrity_ok\":true"));
}

#[test]
fn validate_fails_for_non_contiguous_trace() {
    let encode_output = run_lsteg(&["encode", "--message", "salam", "--emit-trace"]);
    assert!(encode_output.status.success());
    let trace_text = stdout_string(&encode_output);
    let broken_trace = trace_text.replacen("bits=18..39", "bits=19..40", 1);

    let validate_output = run_lsteg_with_stdin(&["validate", "--format", "json"], &broken_trace);
    assert_eq!(validate_output.status.code(), Some(1));

    let stdout = stdout_string(&validate_output);
    assert!(stdout.contains("\"mode\":\"validate\""));
    assert!(stdout.contains("\"integrity_ok\":false"));
    let stderr = stderr_string(&validate_output);
    assert!(stderr.contains("LSTEG-CLI-TRC-001"));
    assert!(stderr.contains("validation failed"));
}

#[test]
fn decode_roundtrip_from_encode_trace_works() {
    let encode_output = run_lsteg(&["encode", "--message", "salam", "--emit-trace"]);
    assert!(encode_output.status.success());
    let trace_text = stdout_string(&encode_output);

    let decode_output = run_lsteg_with_stdin(&["decode", "--format", "json"], &trace_text);
    assert!(decode_output.status.success());

    let decoded_json = stdout_string(&decode_output);
    assert!(decoded_json.contains("\"mode\":\"proto-decode\""));
    assert!(decoded_json.contains("\"payload_utf8\":\"salam\""));
}

#[test]
fn encode_text_output_defaults_to_final_stego_text() {
    let output = run_lsteg(&["encode", "--message", "salam"]);
    assert!(output.status.success());

    let stdout = stdout_string(&output);
    assert!(stdout.contains(" را "));
    assert!(!stdout.contains("prototype encode"));
    assert!(!stdout.contains("frame 01"));
}

#[test]
fn decode_auto_roundtrip_from_farsi_plain_text_works() {
    let encode_output = run_lsteg(&["encode", "--message", "salam"]);
    assert!(encode_output.status.success());
    let stego_text = stdout_string(&encode_output);

    let decode_output = run_lsteg_with_stdin(&["decode", "--format", "json"], &stego_text);
    assert!(decode_output.status.success());

    let decoded_json = stdout_string(&decode_output);
    assert!(decoded_json.contains("\"language\":\"fa\""));
    assert!(decoded_json.contains("\"payload_utf8\":\"salam\""));
}

#[test]
fn analyze_from_farsi_plain_text_reports_integrity_ok() {
    let encode_output = run_lsteg(&["encode", "--message", "salam donya"]);
    assert!(encode_output.status.success());
    let stego_text = stdout_string(&encode_output);

    let analyze_output = run_lsteg_with_stdin(&["analyze", "--format", "json"], &stego_text);
    assert!(analyze_output.status.success());

    let analysis_json = stdout_string(&analyze_output);
    assert!(analysis_json.contains("\"mode\":\"analyze\""));
    assert!(analysis_json.contains("\"language\":\"fa\""));
    assert!(analysis_json.contains("\"integrity_ok\":true"));
    assert!(analysis_json.contains("\"payload_utf8\":\"salam donya\""));
}

#[test]
fn validate_from_farsi_plain_text_reports_integrity_ok() {
    let encode_output = run_lsteg(&["encode", "--message", "salam"]);
    assert!(encode_output.status.success());
    let stego_text = stdout_string(&encode_output);

    let validate_output = run_lsteg_with_stdin(&["validate", "--format", "json"], &stego_text);
    assert!(validate_output.status.success());

    let validation_json = stdout_string(&validate_output);
    assert!(validation_json.contains("\"mode\":\"validate\""));
    assert!(validation_json.contains("\"language\":\"fa\""));
    assert!(validation_json.contains("\"integrity_ok\":true"));
}

#[test]
fn decode_text_input_roundtrip_from_farsi_plain_text_works() {
    let encode_output = run_lsteg(&["encode", "--message", "salam"]);
    assert!(encode_output.status.success());
    let stego_text = stdout_string(&encode_output);

    let decode_output =
        run_lsteg_with_stdin(&["decode", "--text-input", "--format", "json"], &stego_text);
    assert!(decode_output.status.success());

    let decoded_json = stdout_string(&decode_output);
    assert!(decoded_json.contains("\"language\":\"fa\""));
    assert!(decoded_json.contains("\"payload_utf8\":\"salam\""));
}

#[test]
fn decode_text_input_roundtrip_from_english_plain_text_works() {
    let encode_output = run_lsteg(&["encode", "--lang", "en", "--message", "hello world"]);
    assert!(encode_output.status.success());
    let stego_text = stdout_string(&encode_output);

    let decode_output = run_lsteg_with_stdin(
        &["decode", "--lang", "en", "--text-input", "--format", "json"],
        &stego_text,
    );
    assert!(decode_output.status.success());

    let decoded_json = stdout_string(&decode_output);
    assert!(decoded_json.contains("\"language\":\"en\""));
    assert!(decoded_json.contains("\"payload_utf8\":\"hello world\""));
}

#[test]
fn decode_auto_roundtrip_from_english_plain_text_works() {
    let encode_output = run_lsteg(&["encode", "--lang", "en", "--message", "hello world"]);
    assert!(encode_output.status.success());
    let stego_text = stdout_string(&encode_output);

    let decode_output =
        run_lsteg_with_stdin(&["decode", "--lang", "en", "--format", "json"], &stego_text);
    assert!(decode_output.status.success());

    let decoded_json = stdout_string(&decode_output);
    assert!(decoded_json.contains("\"language\":\"en\""));
    assert!(decoded_json.contains("\"payload_utf8\":\"hello world\""));
}

#[test]
fn analyze_from_english_plain_text_reports_integrity_ok() {
    let encode_output = run_lsteg(&["encode", "--lang", "en", "--message", "hello world"]);
    assert!(encode_output.status.success());
    let stego_text = stdout_string(&encode_output);

    let analyze_output = run_lsteg_with_stdin(
        &["analyze", "--lang", "en", "--format", "json"],
        &stego_text,
    );
    assert!(analyze_output.status.success());

    let analysis_json = stdout_string(&analyze_output);
    assert!(analysis_json.contains("\"mode\":\"analyze\""));
    assert!(analysis_json.contains("\"language\":\"en\""));
    assert!(analysis_json.contains("\"integrity_ok\":true"));
    assert!(analysis_json.contains("\"payload_utf8\":\"hello world\""));
}

#[test]
fn validate_from_english_plain_text_reports_integrity_ok() {
    let encode_output = run_lsteg(&["encode", "--lang", "en", "--message", "hello"]);
    assert!(encode_output.status.success());
    let stego_text = stdout_string(&encode_output);

    let validate_output = run_lsteg_with_stdin(
        &["validate", "--lang", "en", "--format", "json"],
        &stego_text,
    );
    assert!(validate_output.status.success());

    let validation_json = stdout_string(&validate_output);
    assert!(validation_json.contains("\"mode\":\"validate\""));
    assert!(validation_json.contains("\"language\":\"en\""));
    assert!(validation_json.contains("\"integrity_ok\":true"));
}

#[test]
fn decode_roundtrip_from_english_trace_works() {
    let encode_output = run_lsteg(&[
        "encode",
        "--lang",
        "en",
        "--message",
        "hello",
        "--emit-trace",
    ]);
    assert!(encode_output.status.success());
    let trace_text = stdout_string(&encode_output);

    let decode_output = run_lsteg_with_stdin(&["decode", "--format", "json"], &trace_text);
    assert!(decode_output.status.success());

    let decoded_json = stdout_string(&decode_output);
    assert!(decoded_json.contains("\"language\":\"en\""));
    assert!(decoded_json.contains("\"payload_utf8\":\"hello\""));
}

#[test]
fn decode_rejects_explicit_language_mismatch() {
    let encode_output = run_lsteg(&[
        "encode",
        "--lang",
        "en",
        "--message",
        "hello",
        "--emit-trace",
    ]);
    assert!(encode_output.status.success());
    let trace_text = stdout_string(&encode_output);

    let decode_output =
        run_lsteg_with_stdin(&["decode", "--lang", "fa", "--format", "json"], &trace_text);
    assert_eq!(decode_output.status.code(), Some(1));

    let stderr = stderr_string(&decode_output);
    assert!(stderr.contains("LSTEG-CLI-CFG-001"));
    assert!(stderr.contains("trace language 'en' does not match requested --lang 'fa'"));
}

#[test]
fn decode_rejects_mixed_language_trace() {
    let encode_output = run_lsteg(&[
        "encode",
        "--lang",
        "en",
        "--message",
        "hello",
        "--emit-trace",
    ]);
    assert!(encode_output.status.success());
    let trace_text = stdout_string(&encode_output);
    let mixed_trace = trace_text.replacen("[en-", "[fa-", 1);

    let decode_output = run_lsteg_with_stdin(&["decode", "--format", "json"], &mixed_trace);
    assert_eq!(decode_output.status.code(), Some(1));

    let stderr = stderr_string(&decode_output);
    assert!(stderr.contains("LSTEG-CLI-CFG-001"));
    assert!(stderr.contains("trace contains mixed language templates"));
}

#[test]
fn analyze_from_trace_reports_integrity_ok() {
    let encode_output = run_lsteg(&["encode", "--message", "salam donya", "--emit-trace"]);
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
fn analyze_auto_detects_english_trace_language() {
    let encode_output = run_lsteg(&[
        "encode",
        "--lang",
        "en",
        "--message",
        "hello world",
        "--emit-trace",
    ]);
    assert!(encode_output.status.success());
    let trace_text = stdout_string(&encode_output);

    let analyze_output = run_lsteg_with_stdin(&["analyze", "--format", "json"], &trace_text);
    assert!(analyze_output.status.success());

    let analysis_json = stdout_string(&analyze_output);
    assert!(analysis_json.contains("\"mode\":\"analyze\""));
    assert!(analysis_json.contains("\"language\":\"en\""));
    assert!(analysis_json.contains("\"integrity_ok\":true"));
    assert!(analysis_json.contains("\"payload_utf8\":\"hello world\""));
}

#[test]
fn analyze_rejects_explicit_language_mismatch() {
    let encode_output = run_lsteg(&[
        "encode",
        "--lang",
        "en",
        "--message",
        "hello world",
        "--emit-trace",
    ]);
    assert!(encode_output.status.success());
    let trace_text = stdout_string(&encode_output);

    let analyze_output = run_lsteg_with_stdin(
        &["analyze", "--lang", "fa", "--format", "json"],
        &trace_text,
    );
    assert_eq!(analyze_output.status.code(), Some(1));

    let stderr = stderr_string(&analyze_output);
    assert!(stderr.contains("LSTEG-CLI-CFG-001"));
    assert!(stderr.contains("trace language 'en' does not match requested --lang 'fa'"));
}

#[test]
fn analyze_rejects_mixed_language_trace() {
    let encode_output = run_lsteg(&[
        "encode",
        "--lang",
        "en",
        "--message",
        "hello world",
        "--emit-trace",
    ]);
    assert!(encode_output.status.success());
    let trace_text = stdout_string(&encode_output);
    let mixed_trace = trace_text.replacen("[en-", "[fa-", 1);

    let analyze_output = run_lsteg_with_stdin(&["analyze", "--format", "json"], &mixed_trace);
    assert_eq!(analyze_output.status.code(), Some(1));

    let stderr = stderr_string(&analyze_output);
    assert!(stderr.contains("LSTEG-CLI-CFG-001"));
    assert!(stderr.contains("trace contains mixed language templates"));
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
    let stderr = stderr_string(&output);
    assert!(stderr.contains("LSTEG-CLI-INP-001"));
    assert!(
        stderr.contains(
            "decode requires input from proto-encode trace output or canonical stego text"
        )
    );
}

#[test]
fn decode_rejects_non_canonical_plain_text_with_input_error() {
    let decode_output =
        run_lsteg_with_stdin(&["decode", "--format", "json"], "this is not stego text");
    assert_eq!(decode_output.status.code(), Some(1));

    let stderr = stderr_string(&decode_output);
    assert!(stderr.contains("LSTEG-CLI-INP-001"));
    assert!(stderr.contains("decode requires parseable trace frames or canonical stego text"));
}

#[test]
fn analyze_rejects_non_canonical_plain_text_with_input_error() {
    let analyze_output =
        run_lsteg_with_stdin(&["analyze", "--format", "json"], "this is not stego text");
    assert_eq!(analyze_output.status.code(), Some(1));

    let stderr = stderr_string(&analyze_output);
    assert!(stderr.contains("LSTEG-CLI-INP-001"));
    assert!(stderr.contains("analyze requires parseable trace frames or canonical stego text"));
}

#[test]
fn validate_rejects_non_canonical_plain_text_with_input_error() {
    let validate_output =
        run_lsteg_with_stdin(&["validate", "--format", "json"], "this is not stego text");
    assert_eq!(validate_output.status.code(), Some(1));

    let stderr = stderr_string(&validate_output);
    assert!(stderr.contains("LSTEG-CLI-INP-001"));
    assert!(stderr.contains("validate requires parseable trace frames or canonical stego text"));
}

#[test]
fn decode_rejects_non_contiguous_trace_ranges() {
    let encode_output = run_lsteg(&["encode", "--message", "salam", "--emit-trace"]);
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
    let encode_output = run_lsteg(&["encode", "--message", "salam", "--emit-trace"]);
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
fn encode_farsi_profile_changes_output_and_keeps_roundtrip_lossless() {
    let literary_output = run_lsteg(&[
        "encode",
        "--lang",
        "fa",
        "--message",
        "salam",
        "--emit-trace",
        "--profile",
        "fa-literary-classic-inspired",
    ]);
    assert!(literary_output.status.success());
    let literary_text = stdout_string(&literary_output);

    assert!(literary_text.contains("style profile: fa-literary-classic-inspired"));

    let decode_output = run_lsteg_with_stdin(
        &["decode", "--lang", "fa", "--text-input", "--format", "json"],
        &literary_text,
    );
    assert!(decode_output.status.success());
    let decoded_json = stdout_string(&decode_output);
    assert!(decoded_json.contains("\"payload_utf8\":\"salam\""));
}

#[test]
fn encode_rejects_unknown_profile_with_config_error() {
    let output = run_lsteg(&[
        "encode",
        "--lang",
        "fa",
        "--message",
        "salam",
        "--profile",
        "fa-unknown-style",
    ]);
    assert_eq!(output.status.code(), Some(1));

    let stderr = stderr_string(&output);
    assert!(stderr.contains("LSTEG-CLI-CFG-001"));
    assert!(stderr.contains("unsupported profile 'fa-unknown-style'"));
}

#[test]
fn encode_uses_profile_from_env_when_flag_missing() {
    let output = run_lsteg_with_env(
        &[
            "encode",
            "--lang",
            "fa",
            "--message",
            "salam",
            "--emit-trace",
        ],
        &[("LSTEG_PROFILE", "fa-literary-classic-inspired")],
    );
    assert!(output.status.success());

    let stdout = stdout_string(&output);
    assert!(stdout.contains("style profile: fa-literary-classic-inspired"));
}

#[test]
fn encode_profile_flag_overrides_env_profile() {
    let output = run_lsteg_with_env(
        &[
            "encode",
            "--lang",
            "fa",
            "--message",
            "salam",
            "--emit-trace",
            "--profile",
            "fa-neutral-formal",
        ],
        &[("LSTEG_PROFILE", "fa-literary-classic-inspired")],
    );
    assert!(output.status.success());

    let stdout = stdout_string(&output);
    assert!(stdout.contains("style profile: fa-neutral-formal"));
    assert!(!stdout.contains("style profile: fa-literary-classic-inspired"));
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
    let encode_output = run_lsteg(&["encode", "--message", "salam", "--emit-trace"]);
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
    let good_trace = stdout_string(&run_lsteg(&[
        "encode",
        "--message",
        "salam",
        "--emit-trace",
    ]));
    let bad_trace = stdout_string(&run_lsteg(&[
        "encode",
        "--message",
        "kharab",
        "--emit-trace",
    ]));

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
    let encode_output = run_lsteg(&["encode", "--message", "salam", "--emit-trace"]);
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
    assert!(stderr.contains("LSTEG-CLI-SEC-001"));
    assert!(stderr.contains("failed to decrypt payload"));
}

#[test]
fn analyze_without_secret_reports_structural_only() {
    let encode_output = run_lsteg(&["encode", "--message", "salam", "--emit-trace"]);
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
    assert!(analysis_json.contains("\"envelope_present\":true"));
    assert!(analysis_json.contains("\"envelope_kdf\":\"argon2id\""));
}

#[test]
fn secret_file_is_used_for_encode_and_decode() {
    let secret_file = TempSecretFile::create("from-file-secret");

    let encode_output = run_lsteg(&[
        "encode",
        "--message",
        "salam",
        "--emit-trace",
        "--secret-file",
        secret_file.as_str(),
    ]);
    assert!(encode_output.status.success());
    let trace_text = stdout_string(&encode_output);

    let decode_with_env_output = run_lsteg_with_env(
        &["decode", "--format", "json"],
        &[("LSTEG_TRACE", &trace_text)],
    );
    assert_eq!(decode_with_env_output.status.code(), Some(1));
    let stderr = stderr_string(&decode_with_env_output);
    assert!(stderr.contains("LSTEG-CLI-SEC-001"));
    assert!(stderr.contains("failed to decrypt payload"));

    let decode_with_file_output = run_lsteg_with_env(
        &[
            "decode",
            "--format",
            "json",
            "--secret-file",
            secret_file.as_str(),
        ],
        &[("LSTEG_TRACE", &trace_text)],
    );
    assert!(decode_with_file_output.status.success());
    assert!(stdout_string(&decode_with_file_output).contains("\"payload_utf8\":\"salam\""));
}

#[test]
fn cli_secret_overrides_env_secret() {
    let encode_output = run_lsteg(&[
        "encode",
        "--message",
        "salam",
        "--emit-trace",
        "--secret",
        "cli-secret",
    ]);
    assert!(encode_output.status.success());
    let trace_text = stdout_string(&encode_output);

    let decode_with_env_output = run_lsteg_with_env(
        &["decode", "--format", "json"],
        &[("LSTEG_TRACE", &trace_text)],
    );
    assert_eq!(decode_with_env_output.status.code(), Some(1));
    let stderr = stderr_string(&decode_with_env_output);
    assert!(stderr.contains("LSTEG-CLI-SEC-001"));
    assert!(stderr.contains("failed to decrypt payload"));

    let decode_with_cli_output = run_lsteg_with_env(
        &["decode", "--format", "json", "--secret", "cli-secret"],
        &[("LSTEG_TRACE", &trace_text)],
    );
    assert!(decode_with_cli_output.status.success());
    assert!(stdout_string(&decode_with_cli_output).contains("\"payload_utf8\":\"salam\""));
}

#[test]
fn analyze_with_wrong_secret_reports_decrypt_integrity_error() {
    let encode_output = run_lsteg(&[
        "encode",
        "--message",
        "salam",
        "--emit-trace",
        "--secret",
        "right-secret",
    ]);
    assert!(encode_output.status.success());
    let trace_text = stdout_string(&encode_output);

    let analyze_output = run_lsteg_with_env(
        &["analyze", "--format", "json", "--secret", "wrong-secret"],
        &[("LSTEG_TRACE", &trace_text)],
    );
    assert!(analyze_output.status.success());
    let analysis_json = stdout_string(&analyze_output);
    assert!(analysis_json.contains("\"integrity_ok\":false"));
    assert!(analysis_json.contains("\"envelope_present\":true"));
    assert!(analysis_json.contains("failed to decrypt payload; verify provided secret"));
}
