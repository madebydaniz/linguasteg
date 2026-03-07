use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Output, Stdio};
use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::Value;

const TEST_SECRET: &str = "linguasteg-test-secret";
const ENV_KEYS: [&str; 10] = [
    "LSTEG_LANG",
    "LSTEG_FORMAT",
    "LSTEG_INPUT",
    "LSTEG_OUTPUT",
    "LSTEG_ENCODE_MESSAGE",
    "LSTEG_PROFILE",
    "LSTEG_TRACE",
    "LSTEG_SECRET",
    "LSTEG_SECRET_FILE",
    "LSTEG_DATA_DIR",
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

fn prepare_encoded_english_text_with_dataset_variant() -> (TempDataDir, TempDataDir, String) {
    let data_dir = TempDataDir::create();
    let env_data_dir = TempDataDir::create();
    let artifact = TempArtifactFile::create(
        br#"{
            "kind":"linguasteg-lexicon-v1",
            "schema_version":1,
            "language":"en",
            "entries":[{"slot":"object","canonical":"letter","variants":["epistle"]}]
        }"#,
    );
    let artifact_url = artifact.as_file_url();

    let install_output = run_lsteg_with_env(
        &[
            "data",
            "install",
            "--lang",
            "en",
            "--source",
            "en-wordlist-wordnik",
            "--artifact-url",
            &artifact_url,
            "--format",
            "json",
            "--data-dir",
            data_dir.as_str(),
        ],
        &[],
    );
    assert!(install_output.status.success());

    let encode_output = run_lsteg_with_env(
        &[
            "encode",
            "--lang",
            "en",
            "--message",
            "hello world",
            "--source",
            "en-wordlist-wordnik",
            "--data-dir",
            data_dir.as_str(),
            "--format",
            "json",
        ],
        &[("LSTEG_SECRET", "1234")],
    );
    assert!(encode_output.status.success());
    let encoded_json: Value =
        serde_json::from_str(&stdout_string(&encode_output)).expect("encode output should be json");
    let final_text = encoded_json
        .get("final_text")
        .and_then(Value::as_str)
        .expect("final_text should be present")
        .to_string();
    assert!(final_text.contains("epistle"));

    (data_dir, env_data_dir, final_text)
}

fn as_legacy_proto_encode_json(trace_json: &str) -> String {
    let mut value: Value =
        serde_json::from_str(trace_json).expect("proto-encode json should be valid");
    let object = value
        .as_object_mut()
        .expect("proto-encode json root should be object");
    object.remove("style_profile");
    object.remove("frame_count");
    serde_json::to_string(&value).expect("legacy proto-encode json should serialize")
}

fn as_extended_proto_encode_json(trace_json: &str) -> String {
    let mut value: Value =
        serde_json::from_str(trace_json).expect("proto-encode json should be valid");
    let object = value
        .as_object_mut()
        .expect("proto-encode json root should be object");
    object.insert(
        "contract_version".to_string(),
        Value::String("compat-vnext".to_string()),
    );
    serde_json::to_string(&value).expect("extended proto-encode json should serialize")
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

struct TempDataDir {
    path: PathBuf,
}

impl TempDataDir {
    fn create() -> Self {
        let mut path = std::env::temp_dir();
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock should be after unix epoch")
            .as_nanos();
        path.push(format!("linguasteg-data-{nanos}"));
        std::fs::create_dir_all(&path).expect("failed to create temp data dir");
        Self { path }
    }

    fn as_str(&self) -> &str {
        self.path
            .to_str()
            .expect("temp data dir path must be valid utf8")
    }
}

impl Drop for TempDataDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.path);
    }
}

struct TempArtifactFile {
    path: PathBuf,
}

impl TempArtifactFile {
    fn create(contents: &[u8]) -> Self {
        let mut path = std::env::temp_dir();
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock should be after unix epoch")
            .as_nanos();
        path.push(format!("linguasteg-artifact-{nanos}.bin"));
        std::fs::write(&path, contents).expect("failed to write temp artifact file");
        Self { path }
    }

    fn as_file_url(&self) -> String {
        format!(
            "file://{}",
            self.path
                .to_str()
                .expect("temp artifact path must be valid utf8")
        )
    }
}

impl Drop for TempArtifactFile {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}

struct TempOutputFile {
    path: PathBuf,
}

impl TempOutputFile {
    fn create() -> Self {
        let mut path = std::env::temp_dir();
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock should be after unix epoch")
            .as_nanos();
        path.push(format!("linguasteg-export-{nanos}.json"));
        Self { path }
    }

    fn as_str(&self) -> &str {
        self.path
            .to_str()
            .expect("temp output path must be valid utf8")
    }
}

impl Drop for TempOutputFile {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}

struct TempTextFile {
    path: PathBuf,
}

impl TempTextFile {
    fn create(contents: &str) -> Self {
        let mut path = std::env::temp_dir();
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock should be after unix epoch")
            .as_nanos();
        path.push(format!("linguasteg-text-{nanos}.txt"));
        std::fs::write(&path, contents).expect("failed to write temp text file");
        Self { path }
    }

    fn empty() -> Self {
        let mut path = std::env::temp_dir();
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock should be after unix epoch")
            .as_nanos();
        path.push(format!("linguasteg-output-{nanos}.txt"));
        Self { path }
    }

    fn as_str(&self) -> &str {
        self.path
            .to_str()
            .expect("temp text file path must be valid utf8")
    }
}

impl Drop for TempTextFile {
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
fn encode_emits_dataset_notice_when_no_source_is_installed() {
    let data_dir = TempDataDir::create();
    let output = run_lsteg_with_env(
        &[
            "encode",
            "--lang",
            "en",
            "--message",
            "hello",
            "--format",
            "json",
            "--data-dir",
            data_dir.as_str(),
        ],
        &[],
    );
    assert!(output.status.success());

    let stderr = stderr_string(&output);
    assert!(stderr.contains("notice: no dataset source is installed for language 'en'"));
    assert!(stderr.contains("lsteg data install --lang en"));
}

#[test]
fn encode_english_varies_output_across_secrets() {
    let output_a = run_lsteg_with_env(
        &[
            "encode",
            "--lang",
            "en",
            "--message",
            "hello world",
            "--format",
            "json",
        ],
        &[("LSTEG_SECRET", "1234")],
    );
    assert!(output_a.status.success());
    let json_a: Value =
        serde_json::from_str(&stdout_string(&output_a)).expect("json output should parse");

    let output_b = run_lsteg_with_env(
        &[
            "encode",
            "--lang",
            "en",
            "--message",
            "hello world",
            "--format",
            "json",
        ],
        &[("LSTEG_SECRET", "12345")],
    );
    assert!(output_b.status.success());
    let json_b: Value =
        serde_json::from_str(&stdout_string(&output_b)).expect("json output should parse");

    assert_ne!(json_a["frames"], json_b["frames"]);
    assert_ne!(json_a["final_text"], json_b["final_text"]);
}

#[test]
fn encode_english_varies_output_across_selected_data_sources() {
    let data_dir = TempDataDir::create();
    let install_recommended = run_lsteg_with_env(
        &[
            "data",
            "install",
            "--lang",
            "en",
            "--format",
            "json",
            "--data-dir",
            data_dir.as_str(),
        ],
        &[],
    );
    assert!(install_recommended.status.success());
    let install_wordnik = run_lsteg_with_env(
        &[
            "data",
            "install",
            "--lang",
            "en",
            "--source",
            "en-wordlist-wordnik",
            "--format",
            "json",
            "--data-dir",
            data_dir.as_str(),
        ],
        &[],
    );
    assert!(install_wordnik.status.success());

    let output_a = run_lsteg_with_env(
        &[
            "encode",
            "--lang",
            "en",
            "--message",
            "hello world",
            "--source",
            "en-wordnet-princeton",
            "--data-dir",
            data_dir.as_str(),
            "--format",
            "json",
        ],
        &[("LSTEG_SECRET", "1234")],
    );
    assert!(output_a.status.success());
    let json_a: Value =
        serde_json::from_str(&stdout_string(&output_a)).expect("json output should parse");

    let output_b = run_lsteg_with_env(
        &[
            "encode",
            "--lang",
            "en",
            "--message",
            "hello world",
            "--source",
            "en-wordlist-wordnik",
            "--data-dir",
            data_dir.as_str(),
            "--format",
            "json",
        ],
        &[("LSTEG_SECRET", "1234")],
    );
    assert!(output_b.status.success());
    let json_b: Value =
        serde_json::from_str(&stdout_string(&output_b)).expect("json output should parse");

    assert_ne!(json_a["final_text"], json_b["final_text"]);

    let stego_a = json_a
        .get("final_text")
        .and_then(Value::as_str)
        .expect("encoded final_text should exist");
    let decode_a = run_lsteg_with_env(
        &[
            "decode",
            "--lang",
            "en",
            "--text-input",
            "--trace",
            stego_a,
            "--format",
            "json",
        ],
        &[("LSTEG_SECRET", "1234")],
    );
    assert!(decode_a.status.success());
    let json_decode_a: Value =
        serde_json::from_str(&stdout_string(&decode_a)).expect("decode json should parse");
    assert_eq!(
        json_decode_a["payload_utf8"],
        Value::String("hello world".to_string())
    );
}

#[test]
fn encode_english_first_frame_sentence_varies_across_secrets() {
    let output_a = run_lsteg_with_env(
        &[
            "encode",
            "--lang",
            "en",
            "--message",
            "hello world",
            "--format",
            "json",
        ],
        &[("LSTEG_SECRET", "1234")],
    );
    assert!(output_a.status.success());
    let json_a: Value =
        serde_json::from_str(&stdout_string(&output_a)).expect("json output should parse");
    let first_sentence_a = json_a
        .get("frames")
        .and_then(Value::as_array)
        .and_then(|frames| frames.first())
        .and_then(|frame| frame.get("sentence"))
        .and_then(Value::as_str)
        .expect("first frame sentence should exist");

    let output_b = run_lsteg_with_env(
        &[
            "encode",
            "--lang",
            "en",
            "--message",
            "hello world",
            "--format",
            "json",
        ],
        &[("LSTEG_SECRET", "12345")],
    );
    assert!(output_b.status.success());
    let json_b: Value =
        serde_json::from_str(&stdout_string(&output_b)).expect("json output should parse");
    let first_sentence_b = json_b
        .get("frames")
        .and_then(Value::as_array)
        .and_then(|frames| frames.first())
        .and_then(|frame| frame.get("sentence"))
        .and_then(Value::as_str)
        .expect("first frame sentence should exist");

    assert_ne!(first_sentence_a, first_sentence_b);
}

#[test]
fn encode_farsi_first_frame_sentence_varies_across_secrets() {
    let output_a = run_lsteg_with_env(
        &[
            "encode",
            "--lang",
            "fa",
            "--message",
            "hello world",
            "--format",
            "json",
        ],
        &[("LSTEG_SECRET", "1234")],
    );
    assert!(output_a.status.success());
    let json_a: Value =
        serde_json::from_str(&stdout_string(&output_a)).expect("json output should parse");
    let first_sentence_a = json_a
        .get("frames")
        .and_then(Value::as_array)
        .and_then(|frames| frames.first())
        .and_then(|frame| frame.get("sentence"))
        .and_then(Value::as_str)
        .expect("first frame sentence should exist");

    let output_b = run_lsteg_with_env(
        &[
            "encode",
            "--lang",
            "fa",
            "--message",
            "hello world",
            "--format",
            "json",
        ],
        &[("LSTEG_SECRET", "12345")],
    );
    assert!(output_b.status.success());
    let json_b: Value =
        serde_json::from_str(&stdout_string(&output_b)).expect("json output should parse");
    let first_sentence_b = json_b
        .get("frames")
        .and_then(Value::as_array)
        .and_then(|frames| frames.first())
        .and_then(|frame| frame.get("sentence"))
        .and_then(Value::as_str)
        .expect("first frame sentence should exist");

    assert_ne!(first_sentence_a, first_sentence_b);
}

#[test]
fn languages_text_lists_supported_languages() {
    let output = run_lsteg(&["languages"]);
    assert!(output.status.success());

    let stdout = stdout_string(&output);
    assert!(stdout.contains("supported languages:"));
    assert!(stdout.contains("- fa (Farsi, rtl)"));
    assert!(stdout.contains("- en (English, ltr)"));
    assert!(stdout.contains("- de (German, ltr)"));
    assert!(stdout.contains("- it (Italian, ltr)"));
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
    assert!(stdout.contains("\"code\":\"de\""));
    assert!(stdout.contains("\"code\":\"it\""));
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
        "- stub/stub-local (Stub Local) languages: fa,en,de,it capabilities: deterministic-seed"
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
    assert!(stdout.contains("\"languages\":[\"fa\",\"en\",\"de\",\"it\"]"));
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
    assert!(stdout.contains("- de (German, ltr)"));
    assert!(stdout.contains("- it (Italian, ltr)"));
    assert!(stdout.contains("- symbolic-stub (Symbolic Stub) capabilities: deterministic-seed"));
    assert!(stdout.contains(
        "- stub/stub-local (Stub Local) languages: fa,en,de,it capabilities: deterministic-seed"
    ));
    assert!(stdout.contains("- fa/fa-basic-sov (Basic SOV) slots: 4"));
    assert!(stdout.contains("- de/de-basic-svo (German Basic SVO) slots: 4"));
    assert!(stdout.contains(
        "- en/en-neutral-prototype (Neutral English Prototype) register: neutral strength: light inspiration: register-only (<none>)"
    ));
    assert!(stdout.contains(
        "- en/en-shakespeare-inspired-light (Shakespeare-inspired (Light)) register: literary strength: light inspiration: author-inspired (William Shakespeare)"
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
    assert!(stdout.contains("- de/de-basic-svo (German Basic SVO) slots: 4"));
    assert!(stdout.contains("- de/de-time-location-svo (German Time Location SVO) slots: 5"));
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
    assert!(stdout.contains("- de/de-neutral-prototype (Neutral German Prototype)"));
    assert!(stdout.contains("- en/en-shakespeare-inspired-light (Shakespeare-inspired (Light))"));
    assert!(stdout.contains("- en/en-dickens-inspired-light (Dickens-inspired (Light))"));
    assert!(stdout.contains("- en/en-austen-inspired-light (Austen-inspired (Light))"));
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
    assert!(stdout.contains("\"id\":\"en-shakespeare-inspired-light\""));
    assert!(stdout.contains("\"inspiration_label\":\"William Shakespeare\""));
}

#[test]
fn profiles_lang_filter_limits_output() {
    let output = run_lsteg(&["profiles", "--lang", "en", "--format", "json"]);
    assert!(output.status.success());

    let stdout = stdout_string(&output);
    assert!(stdout.contains("\"mode\":\"profiles\""));
    assert!(stdout.contains("\"language\":\"en\""));
    assert!(stdout.contains("\"id\":\"en-neutral-prototype\""));
    assert!(stdout.contains("\"id\":\"en-shakespeare-inspired-light\""));
    assert!(stdout.contains("\"id\":\"en-dickens-inspired-light\""));
    assert!(stdout.contains("\"id\":\"en-austen-inspired-light\""));
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
    assert!(stdout.contains("- de/de-time-location-svo total_bits: 21"));
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
fn data_list_json_exposes_sources() {
    let output = run_lsteg(&["data", "list", "--format", "json"]);
    assert!(output.status.success());

    let stdout = stdout_string(&output);
    assert!(stdout.contains("\"mode\":\"data-list\""));
    assert!(stdout.contains("\"language\":\"en\""));
    assert!(stdout.contains("\"language\":\"fa\""));
    assert!(stdout.contains("\"source_id\":\"en-wordnet-princeton\""));
    assert!(stdout.contains("\"source_id\":\"fa-wordlist-cc0\""));
}

#[test]
fn data_install_creates_state_and_source_manifest() {
    let data_dir = TempDataDir::create();
    let output = run_lsteg_with_env(
        &[
            "data",
            "install",
            "--lang",
            "en,fa",
            "--format",
            "json",
            "--data-dir",
            data_dir.as_str(),
        ],
        &[],
    );
    assert!(output.status.success());

    let stdout = stdout_string(&output);
    assert!(stdout.contains("\"mode\":\"data-install\""));
    assert!(stdout.contains("\"status\":\"installed\""));

    let state_path = std::path::Path::new(data_dir.as_str()).join("state.json");
    let en_manifest_path = std::path::Path::new(data_dir.as_str())
        .join("en")
        .join("en-wordnet-princeton")
        .join("manifest.json");
    let fa_manifest_path = std::path::Path::new(data_dir.as_str())
        .join("fa")
        .join("fa-wordlist-cc0")
        .join("manifest.json");
    let en_starter_dataset_path = std::path::Path::new(data_dir.as_str())
        .join("en")
        .join("en-wordnet-princeton")
        .join("dataset.json");
    let fa_starter_dataset_path = std::path::Path::new(data_dir.as_str())
        .join("fa")
        .join("fa-wordlist-cc0")
        .join("dataset.json");

    assert!(state_path.exists());
    assert!(en_manifest_path.exists());
    assert!(fa_manifest_path.exists());
    assert!(en_starter_dataset_path.exists());
    assert!(fa_starter_dataset_path.exists());
}

#[test]
fn data_update_marks_existing_install_as_updated() {
    let data_dir = TempDataDir::create();
    let install_output = run_lsteg_with_env(
        &[
            "data",
            "install",
            "--lang",
            "en",
            "--format",
            "json",
            "--data-dir",
            data_dir.as_str(),
        ],
        &[],
    );
    assert!(install_output.status.success());

    let update_output = run_lsteg_with_env(
        &[
            "data",
            "update",
            "--lang",
            "en",
            "--format",
            "json",
            "--data-dir",
            data_dir.as_str(),
        ],
        &[],
    );
    assert!(update_output.status.success());
    let stdout = stdout_string(&update_output);
    assert!(stdout.contains("\"mode\":\"data-update\""));
    assert!(stdout.contains("\"status\":\"updated\""));
    assert!(stdout.contains("\"artifact_path\":\""));
}

#[test]
fn data_install_accepts_explicit_source_for_single_language() {
    let data_dir = TempDataDir::create();
    let output = run_lsteg_with_env(
        &[
            "data",
            "install",
            "--lang",
            "fa",
            "--source",
            "fa-wordlist-mit",
            "--format",
            "json",
            "--data-dir",
            data_dir.as_str(),
        ],
        &[],
    );
    assert!(output.status.success());
    let stdout = stdout_string(&output);
    assert!(stdout.contains("\"source_id\":\"fa-wordlist-mit\""));
}

#[test]
fn data_install_rejects_source_with_multi_language_target() {
    let output = run_lsteg(&[
        "data",
        "install",
        "--lang",
        "fa,en",
        "--source",
        "fa-wordlist-mit",
        "--format",
        "json",
    ]);
    assert_eq!(output.status.code(), Some(2));
    let stderr = stderr_string(&output);
    assert!(stderr.contains("LSTEG-CLI-ARG-001"));
    assert!(stderr.contains("--source can be used only with a single language in --lang"));
}

#[test]
fn data_install_with_artifact_url_stores_artifact_and_hash() {
    let data_dir = TempDataDir::create();
    let artifact = TempArtifactFile::create(b"linguasteg-dataset");
    let artifact_url = artifact.as_file_url();
    let output = run_lsteg_with_env(
        &[
            "data",
            "install",
            "--lang",
            "en",
            "--source",
            "en-wordlist-wordnik",
            "--artifact-url",
            &artifact_url,
            "--format",
            "json",
            "--data-dir",
            data_dir.as_str(),
        ],
        &[],
    );
    assert!(output.status.success());
    let stdout = stdout_string(&output);
    assert!(stdout.contains("\"source_id\":\"en-wordlist-wordnik\""));
    assert!(stdout.contains("\"artifact_sha256\":\""));

    let artifact_path = std::path::Path::new(data_dir.as_str())
        .join("en")
        .join("en-wordlist-wordnik")
        .join("artifact.bin");
    assert!(artifact_path.exists());
}

#[test]
fn data_install_with_lexicon_dataset_artifact_records_dataset_metadata() {
    let data_dir = TempDataDir::create();
    let artifact = TempArtifactFile::create(
        br#"{
            "kind":"linguasteg-lexicon-v1",
            "schema_version":1,
            "language":"en",
            "entries":[
                {"slot":"object","canonical":"letter","variants":["missive","epistle"]},
                {"slot":"verb","canonical":"writes","variants":["composes"]}
            ]
        }"#,
    );
    let artifact_url = artifact.as_file_url();
    let output = run_lsteg_with_env(
        &[
            "data",
            "install",
            "--lang",
            "en",
            "--source",
            "en-wordlist-wordnik",
            "--artifact-url",
            &artifact_url,
            "--format",
            "json",
            "--data-dir",
            data_dir.as_str(),
        ],
        &[],
    );
    assert!(output.status.success());

    let manifest_path = std::path::Path::new(data_dir.as_str())
        .join("en")
        .join("en-wordlist-wordnik")
        .join("manifest.json");
    let manifest_raw = std::fs::read_to_string(&manifest_path).expect("manifest should exist");
    let manifest: Value = serde_json::from_str(&manifest_raw).expect("manifest should be json");

    assert_eq!(
        manifest["artifact_dataset_kind"].as_str(),
        Some("linguasteg-lexicon-v1")
    );
    assert_eq!(
        manifest["artifact_dataset_schema_version"].as_u64(),
        Some(1)
    );
    assert_eq!(manifest["artifact_dataset_language"].as_str(), Some("en"));
    assert_eq!(manifest["artifact_dataset_entry_count"].as_u64(), Some(2));
}

#[test]
fn data_install_rejects_lexicon_dataset_artifact_with_mismatched_language() {
    let data_dir = TempDataDir::create();
    let artifact = TempArtifactFile::create(
        br#"{
            "kind":"linguasteg-lexicon-v1",
            "schema_version":1,
            "language":"fa",
            "entries":[{"slot":"object","canonical":"letter","variants":["missive"]}]
        }"#,
    );
    let artifact_url = artifact.as_file_url();
    let output = run_lsteg_with_env(
        &[
            "data",
            "install",
            "--lang",
            "en",
            "--source",
            "en-wordlist-wordnik",
            "--artifact-url",
            &artifact_url,
            "--format",
            "json",
            "--data-dir",
            data_dir.as_str(),
        ],
        &[],
    );
    assert_eq!(output.status.code(), Some(1));
    let stderr = stderr_string(&output);
    assert!(stderr.contains("LSTEG-CLI-CFG-001"));
    assert!(stderr.contains("artifact validation failed"));
    assert!(stderr.contains("does not match selected language"));
}

#[test]
fn encode_uses_dataset_variants_from_selected_source_artifact() {
    let data_dir = TempDataDir::create();
    let artifact = TempArtifactFile::create(
        br#"{
            "kind":"linguasteg-lexicon-v1",
            "schema_version":1,
            "language":"en",
            "entries":[{"slot":"object","canonical":"letter","variants":["epistle"]}]
        }"#,
    );
    let artifact_url = artifact.as_file_url();
    let install_output = run_lsteg_with_env(
        &[
            "data",
            "install",
            "--lang",
            "en",
            "--source",
            "en-wordlist-wordnik",
            "--artifact-url",
            &artifact_url,
            "--format",
            "json",
            "--data-dir",
            data_dir.as_str(),
        ],
        &[],
    );
    assert!(install_output.status.success());

    let encode_output = run_lsteg_with_env(
        &[
            "encode",
            "--lang",
            "en",
            "--message",
            "hello world",
            "--source",
            "en-wordlist-wordnik",
            "--data-dir",
            data_dir.as_str(),
            "--format",
            "json",
        ],
        &[("LSTEG_SECRET", "1234")],
    );
    assert!(encode_output.status.success());
    let encoded_json: Value =
        serde_json::from_str(&stdout_string(&encode_output)).expect("encode output should be json");

    let final_text = encoded_json
        .get("final_text")
        .and_then(Value::as_str)
        .expect("final_text should be present");
    assert!(final_text.contains("epistle"));

    let decode_output = run_lsteg_with_env(
        &[
            "decode",
            "--lang",
            "en",
            "--text-input",
            "--trace",
            final_text,
            "--format",
            "json",
        ],
        &[
            ("LSTEG_SECRET", "1234"),
            ("LSTEG_DATA_DIR", data_dir.as_str()),
        ],
    );
    assert!(decode_output.status.success());
    let decoded_json: Value =
        serde_json::from_str(&stdout_string(&decode_output)).expect("decode output should be json");
    assert_eq!(
        decoded_json["payload_utf8"],
        Value::String("hello world".to_string())
    );
}

#[test]
fn decode_data_dir_flag_overrides_env_for_text_input() {
    let (data_dir, env_data_dir, final_text) = prepare_encoded_english_text_with_dataset_variant();
    let output = run_lsteg_with_env(
        &[
            "decode",
            "--lang",
            "en",
            "--text-input",
            "--trace",
            final_text.as_str(),
            "--data-dir",
            data_dir.as_str(),
            "--format",
            "json",
        ],
        &[
            ("LSTEG_SECRET", "1234"),
            ("LSTEG_DATA_DIR", env_data_dir.as_str()),
        ],
    );
    assert!(output.status.success());
    let decoded_json: Value =
        serde_json::from_str(&stdout_string(&output)).expect("decode output should be json");
    assert_eq!(
        decoded_json["payload_utf8"],
        Value::String("hello world".to_string())
    );
}

#[test]
fn analyze_data_dir_flag_overrides_env_for_text_input() {
    let (data_dir, env_data_dir, final_text) = prepare_encoded_english_text_with_dataset_variant();
    let output = run_lsteg_with_env(
        &[
            "analyze",
            "--lang",
            "en",
            "--text-input",
            "--trace",
            final_text.as_str(),
            "--data-dir",
            data_dir.as_str(),
            "--format",
            "json",
        ],
        &[
            ("LSTEG_SECRET", "1234"),
            ("LSTEG_DATA_DIR", env_data_dir.as_str()),
        ],
    );
    assert!(output.status.success());
    let analysis_json = stdout_string(&output);
    assert!(analysis_json.contains("\"integrity_ok\":true"));
    assert!(analysis_json.contains("\"payload_utf8\":\"hello world\""));
}

#[test]
fn validate_data_dir_flag_overrides_env_for_text_input() {
    let (data_dir, env_data_dir, final_text) = prepare_encoded_english_text_with_dataset_variant();
    let output = run_lsteg_with_env(
        &[
            "validate",
            "--lang",
            "en",
            "--text-input",
            "--trace",
            final_text.as_str(),
            "--data-dir",
            data_dir.as_str(),
            "--format",
            "json",
        ],
        &[
            ("LSTEG_SECRET", "1234"),
            ("LSTEG_DATA_DIR", env_data_dir.as_str()),
        ],
    );
    assert!(output.status.success());
    let validate_json = stdout_string(&output);
    assert!(validate_json.contains("\"integrity_ok\":true"));
}

#[test]
fn data_install_rejects_artifact_url_with_multi_language_target() {
    let output = run_lsteg(&[
        "data",
        "install",
        "--lang",
        "fa,en",
        "--artifact-url",
        "file:///tmp/anything.bin",
    ]);
    assert_eq!(output.status.code(), Some(2));
    let stderr = stderr_string(&output);
    assert!(stderr.contains("LSTEG-CLI-ARG-001"));
    assert!(stderr.contains("--artifact-url can be used only with a single language in --lang"));
}

#[test]
fn data_artifact_validate_reports_ok_for_valid_lexicon_dataset() {
    let artifact = TempArtifactFile::create(
        br#"{
            "kind":"linguasteg-lexicon-v1",
            "schema_version":1,
            "language":"en",
            "entries":[{"slot":"object","canonical":"letter","variants":["epistle"]}]
        }"#,
    );
    let output = run_lsteg(&[
        "data",
        "artifact",
        "validate",
        "--lang",
        "en",
        "--input",
        &artifact.as_file_url(),
        "--format",
        "json",
    ]);
    assert!(output.status.success());
    let stdout = stdout_string(&output);
    assert!(stdout.contains("\"mode\":\"data-artifact-validate\""));
    assert!(stdout.contains("\"valid\":true"));
    assert!(stdout.contains("\"entry_count\":1"));
}

#[test]
fn data_artifact_validate_fails_for_non_dataset_input() {
    let artifact = TempArtifactFile::create(b"not-a-dataset");
    let output = run_lsteg(&[
        "data",
        "artifact",
        "validate",
        "--lang",
        "en",
        "--input",
        &artifact.as_file_url(),
        "--format",
        "json",
    ]);
    assert_eq!(output.status.code(), Some(1));
    let stdout = stdout_string(&output);
    assert!(stdout.contains("\"mode\":\"data-artifact-validate\""));
    assert!(stdout.contains("\"valid\":false"));

    let stderr = stderr_string(&output);
    assert!(stderr.contains("LSTEG-CLI-CFG-001"));
    assert!(stderr.contains("artifact validation failed"));
}

#[test]
fn data_status_reports_installed_manifest_state() {
    let data_dir = TempDataDir::create();
    let install_output = run_lsteg_with_env(
        &[
            "data",
            "install",
            "--lang",
            "en",
            "--format",
            "json",
            "--data-dir",
            data_dir.as_str(),
        ],
        &[],
    );
    assert!(install_output.status.success());

    let status_output = run_lsteg_with_env(
        &[
            "data",
            "status",
            "--lang",
            "en",
            "--format",
            "json",
            "--data-dir",
            data_dir.as_str(),
        ],
        &[],
    );
    assert!(status_output.status.success());
    let stdout = stdout_string(&status_output);
    assert!(stdout.contains("\"mode\":\"data-status\""));
    assert!(stdout.contains("\"source_id\":\"en-wordnet-princeton\""));
    assert!(stdout.contains("\"manifest_exists\":true"));
    assert!(stdout.contains("\"status\":\"ok\""));
}

#[test]
fn data_status_reports_missing_artifact_when_removed() {
    let data_dir = TempDataDir::create();
    let artifact = TempArtifactFile::create(b"linguasteg-dataset");
    let artifact_url = artifact.as_file_url();
    let install_output = run_lsteg_with_env(
        &[
            "data",
            "install",
            "--lang",
            "en",
            "--source",
            "en-wordlist-wordnik",
            "--artifact-url",
            &artifact_url,
            "--format",
            "json",
            "--data-dir",
            data_dir.as_str(),
        ],
        &[],
    );
    assert!(install_output.status.success());

    let artifact_path = std::path::Path::new(data_dir.as_str())
        .join("en")
        .join("en-wordlist-wordnik")
        .join("artifact.bin");
    std::fs::remove_file(&artifact_path).expect("artifact file should be removable");

    let status_output = run_lsteg_with_env(
        &[
            "data",
            "status",
            "--lang",
            "en",
            "--format",
            "json",
            "--data-dir",
            data_dir.as_str(),
        ],
        &[],
    );
    assert!(status_output.status.success());
    let stdout = stdout_string(&status_output);
    assert!(stdout.contains("\"source_id\":\"en-wordlist-wordnik\""));
    assert!(stdout.contains("\"artifact_exists\":false"));
    assert!(stdout.contains("\"status\":\"missing-artifact\""));
}

#[test]
fn data_verify_reports_ok_for_matching_artifact() {
    let data_dir = TempDataDir::create();
    let artifact = TempArtifactFile::create(b"linguasteg-dataset");
    let artifact_url = artifact.as_file_url();
    let install_output = run_lsteg_with_env(
        &[
            "data",
            "install",
            "--lang",
            "en",
            "--source",
            "en-wordlist-wordnik",
            "--artifact-url",
            &artifact_url,
            "--format",
            "json",
            "--data-dir",
            data_dir.as_str(),
        ],
        &[],
    );
    assert!(install_output.status.success());

    let verify_output = run_lsteg_with_env(
        &[
            "data",
            "verify",
            "--lang",
            "en",
            "--source",
            "en-wordlist-wordnik",
            "--format",
            "json",
            "--data-dir",
            data_dir.as_str(),
        ],
        &[],
    );
    assert!(verify_output.status.success());
    let stdout = stdout_string(&verify_output);
    assert!(stdout.contains("\"mode\":\"data-verify\""));
    assert!(stdout.contains("\"integrity_ok\":true"));
    assert!(stdout.contains("\"passed\":1"));
    assert!(stdout.contains("\"failed\":0"));
    assert!(stdout.contains("\"status\":\"ok\""));
}

#[test]
fn data_verify_fails_for_checksum_mismatch() {
    let data_dir = TempDataDir::create();
    let artifact = TempArtifactFile::create(b"linguasteg-dataset");
    let artifact_url = artifact.as_file_url();
    let install_output = run_lsteg_with_env(
        &[
            "data",
            "install",
            "--lang",
            "en",
            "--source",
            "en-wordlist-wordnik",
            "--artifact-url",
            &artifact_url,
            "--format",
            "json",
            "--data-dir",
            data_dir.as_str(),
        ],
        &[],
    );
    assert!(install_output.status.success());

    let artifact_path = std::path::Path::new(data_dir.as_str())
        .join("en")
        .join("en-wordlist-wordnik")
        .join("artifact.bin");
    std::fs::write(&artifact_path, b"tampered-content").expect("artifact should be writable");

    let verify_output = run_lsteg_with_env(
        &[
            "data",
            "verify",
            "--lang",
            "en",
            "--source",
            "en-wordlist-wordnik",
            "--format",
            "json",
            "--data-dir",
            data_dir.as_str(),
        ],
        &[],
    );
    assert_eq!(verify_output.status.code(), Some(1));
    let stdout = stdout_string(&verify_output);
    assert!(stdout.contains("\"integrity_ok\":false"));
    assert!(stdout.contains("\"failed\":1"));
    assert!(stdout.contains("\"status\":\"checksum-mismatch\""));
    let stderr = stderr_string(&verify_output);
    assert!(stderr.contains("LSTEG-CLI-DOM-001"));
    assert!(stderr.contains("data verification failed"));
}

#[test]
fn data_verify_skips_when_no_artifact_is_present() {
    let data_dir = TempDataDir::create();
    let install_output = run_lsteg_with_env(
        &[
            "data",
            "install",
            "--lang",
            "en",
            "--format",
            "json",
            "--data-dir",
            data_dir.as_str(),
        ],
        &[],
    );
    assert!(install_output.status.success());

    let verify_output = run_lsteg_with_env(
        &[
            "data",
            "verify",
            "--lang",
            "en",
            "--format",
            "json",
            "--data-dir",
            data_dir.as_str(),
        ],
        &[],
    );
    assert!(verify_output.status.success());
    let stdout = stdout_string(&verify_output);
    assert!(stdout.contains("\"integrity_ok\":true"));
    assert!(stdout.contains("\"passed\":1"));
    assert!(stdout.contains("\"status\":\"ok\""));
}

#[test]
fn data_install_source_list_prints_available_ids_for_language() {
    let data_dir = TempDataDir::create();
    let output = run_lsteg_with_env(
        &[
            "data",
            "install",
            "--lang",
            "en",
            "--source",
            "list",
            "--format",
            "json",
            "--data-dir",
            data_dir.as_str(),
        ],
        &[],
    );
    assert!(output.status.success());
    let stdout = stdout_string(&output);
    assert!(stdout.contains("\"mode\":\"data-list\""));
    assert!(stdout.contains("\"language\":\"en\""));
    assert!(stdout.contains("\"source_id\":\"en-wordnet-princeton\""));
    assert!(stdout.contains("\"source_id\":\"en-wordlist-wordnik\""));
}

#[test]
fn data_pin_sets_manifest_checksum_from_artifact() {
    let data_dir = TempDataDir::create();
    let artifact = TempArtifactFile::create(b"linguasteg-dataset");
    let artifact_url = artifact.as_file_url();
    let install_output = run_lsteg_with_env(
        &[
            "data",
            "install",
            "--lang",
            "en",
            "--source",
            "en-wordlist-wordnik",
            "--artifact-url",
            &artifact_url,
            "--format",
            "json",
            "--data-dir",
            data_dir.as_str(),
        ],
        &[],
    );
    assert!(install_output.status.success());

    let pin_output = run_lsteg_with_env(
        &[
            "data",
            "pin",
            "--lang",
            "en",
            "--source",
            "en-wordlist-wordnik",
            "--format",
            "json",
            "--data-dir",
            data_dir.as_str(),
        ],
        &[],
    );
    assert!(pin_output.status.success());
    let stdout = stdout_string(&pin_output);
    assert!(stdout.contains("\"mode\":\"data-pin\""));
    assert!(stdout.contains("\"updated\":1"));
    assert!(stdout.contains("\"status\":\"pinned\""));

    let manifest_path = std::path::Path::new(data_dir.as_str())
        .join("en")
        .join("en-wordlist-wordnik")
        .join("manifest.json");
    let manifest = std::fs::read_to_string(manifest_path).expect("manifest should be readable");
    assert!(manifest.contains("\"artifact_sha256\": \""));
    assert!(!manifest.contains("\"artifact_sha256\": null"));
}

#[test]
fn data_pin_accepts_explicit_checksum() {
    let data_dir = TempDataDir::create();
    let artifact = TempArtifactFile::create(b"linguasteg-dataset");
    let artifact_url = artifact.as_file_url();
    let install_output = run_lsteg_with_env(
        &[
            "data",
            "install",
            "--lang",
            "en",
            "--source",
            "en-wordlist-wordnik",
            "--artifact-url",
            &artifact_url,
            "--format",
            "json",
            "--data-dir",
            data_dir.as_str(),
        ],
        &[],
    );
    assert!(install_output.status.success());

    let checksum = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
    let pin_output = run_lsteg_with_env(
        &[
            "data",
            "pin",
            "--lang",
            "en",
            "--source",
            "en-wordlist-wordnik",
            "--checksum",
            checksum,
            "--format",
            "json",
            "--data-dir",
            data_dir.as_str(),
        ],
        &[],
    );
    assert!(pin_output.status.success());
    let stdout = stdout_string(&pin_output);
    assert!(stdout.contains("\"status\":\"pinned\""));
    assert!(stdout.contains(checksum));
}

#[test]
fn data_pin_rejects_checksum_for_multiple_selected_sources() {
    let data_dir = TempDataDir::create();
    let install_output = run_lsteg_with_env(
        &[
            "data",
            "install",
            "--lang",
            "en,fa",
            "--format",
            "json",
            "--data-dir",
            data_dir.as_str(),
        ],
        &[],
    );
    assert!(install_output.status.success());

    let pin_output = run_lsteg_with_env(
        &[
            "data",
            "pin",
            "--checksum",
            "cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc",
            "--format",
            "json",
            "--data-dir",
            data_dir.as_str(),
        ],
        &[],
    );
    assert_eq!(pin_output.status.code(), Some(2));
    let stderr = stderr_string(&pin_output);
    assert!(stderr.contains("LSTEG-CLI-ARG-001"));
    assert!(
        stderr
            .contains("--checksum can be used only when exactly one installed source is selected")
    );
}

#[test]
fn data_export_manifest_prints_json_snapshot() {
    let data_dir = TempDataDir::create();
    let artifact = TempArtifactFile::create(b"linguasteg-dataset");
    let artifact_url = artifact.as_file_url();
    let install_output = run_lsteg_with_env(
        &[
            "data",
            "install",
            "--lang",
            "en",
            "--source",
            "en-wordlist-wordnik",
            "--artifact-url",
            &artifact_url,
            "--format",
            "json",
            "--data-dir",
            data_dir.as_str(),
        ],
        &[],
    );
    assert!(install_output.status.success());

    let export_output = run_lsteg_with_env(
        &[
            "data",
            "export-manifest",
            "--lang",
            "en",
            "--source",
            "en-wordlist-wordnik",
            "--data-dir",
            data_dir.as_str(),
        ],
        &[],
    );
    assert!(export_output.status.success());
    let stdout = stdout_string(&export_output);
    assert!(stdout.contains("\"schema_version\": 1"));
    assert!(stdout.contains("\"source_id\": \"en-wordlist-wordnik\""));
    assert!(stdout.contains("\"artifact_sha256\":"));
}

#[test]
fn data_export_manifest_writes_output_file_when_requested() {
    let data_dir = TempDataDir::create();
    let output_file = TempOutputFile::create();
    let install_output = run_lsteg_with_env(
        &[
            "data",
            "install",
            "--lang",
            "fa",
            "--source",
            "fa-wordlist-mit",
            "--format",
            "json",
            "--data-dir",
            data_dir.as_str(),
        ],
        &[],
    );
    assert!(install_output.status.success());

    let export_output = run_lsteg_with_env(
        &[
            "data",
            "export-manifest",
            "--lang",
            "fa",
            "--source",
            "fa-wordlist-mit",
            "--output",
            output_file.as_str(),
            "--format",
            "json",
            "--data-dir",
            data_dir.as_str(),
        ],
        &[],
    );
    assert!(export_output.status.success());
    let stdout = stdout_string(&export_output);
    assert!(stdout.contains("\"mode\":\"data-export-manifest\""));
    assert!(stdout.contains("\"entry_count\":1"));

    let exported = std::fs::read_to_string(output_file.as_str())
        .expect("exported manifest should be readable");
    assert!(exported.contains("\"source_id\": \"fa-wordlist-mit\""));
}

#[test]
fn data_import_manifest_restores_state_and_manifests() {
    let source_data_dir = TempDataDir::create();
    let target_data_dir = TempDataDir::create();
    let export_file = TempOutputFile::create();

    let install_output = run_lsteg_with_env(
        &[
            "data",
            "install",
            "--lang",
            "en",
            "--source",
            "en-wordlist-wordnik",
            "--format",
            "json",
            "--data-dir",
            source_data_dir.as_str(),
        ],
        &[],
    );
    assert!(install_output.status.success());

    let export_output = run_lsteg_with_env(
        &[
            "data",
            "export-manifest",
            "--lang",
            "en",
            "--source",
            "en-wordlist-wordnik",
            "--output",
            export_file.as_str(),
            "--format",
            "json",
            "--data-dir",
            source_data_dir.as_str(),
        ],
        &[],
    );
    assert!(export_output.status.success());

    let import_output = run_lsteg_with_env(
        &[
            "data",
            "import-manifest",
            "--input",
            export_file.as_str(),
            "--format",
            "json",
            "--data-dir",
            target_data_dir.as_str(),
        ],
        &[],
    );
    assert!(import_output.status.success());
    let stdout = stdout_string(&import_output);
    assert!(stdout.contains("\"mode\":\"data-import-manifest\""));
    assert!(stdout.contains("\"source_id\":\"en-wordlist-wordnik\""));

    let status_output = run_lsteg_with_env(
        &[
            "data",
            "status",
            "--lang",
            "en",
            "--format",
            "json",
            "--data-dir",
            target_data_dir.as_str(),
        ],
        &[],
    );
    assert!(status_output.status.success());
    let status_stdout = stdout_string(&status_output);
    assert!(status_stdout.contains("\"source_id\":\"en-wordlist-wordnik\""));
    assert!(status_stdout.contains("\"manifest_exists\":true"));
}

#[test]
fn data_import_manifest_respects_filters() {
    let source_data_dir = TempDataDir::create();
    let target_data_dir = TempDataDir::create();
    let export_file = TempOutputFile::create();

    let install_output = run_lsteg_with_env(
        &[
            "data",
            "install",
            "--lang",
            "en,fa",
            "--format",
            "json",
            "--data-dir",
            source_data_dir.as_str(),
        ],
        &[],
    );
    assert!(install_output.status.success());

    let export_output = run_lsteg_with_env(
        &[
            "data",
            "export-manifest",
            "--output",
            export_file.as_str(),
            "--format",
            "json",
            "--data-dir",
            source_data_dir.as_str(),
        ],
        &[],
    );
    assert!(export_output.status.success());

    let import_output = run_lsteg_with_env(
        &[
            "data",
            "import-manifest",
            "--input",
            export_file.as_str(),
            "--lang",
            "fa",
            "--format",
            "json",
            "--data-dir",
            target_data_dir.as_str(),
        ],
        &[],
    );
    assert!(import_output.status.success());
    let stdout = stdout_string(&import_output);
    assert!(stdout.contains("\"language\":\"fa\""));
    assert!(!stdout.contains("\"language\":\"en\""));
}

#[test]
fn data_doctor_reports_missing_manifest_issue() {
    let data_dir = TempDataDir::create();
    let install_output = run_lsteg_with_env(
        &[
            "data",
            "install",
            "--lang",
            "en",
            "--source",
            "en-wordlist-wordnik",
            "--format",
            "json",
            "--data-dir",
            data_dir.as_str(),
        ],
        &[],
    );
    assert!(install_output.status.success());

    let manifest_path = std::path::Path::new(data_dir.as_str())
        .join("en")
        .join("en-wordlist-wordnik")
        .join("manifest.json");
    std::fs::remove_file(&manifest_path).expect("manifest should be removable");

    let doctor_output = run_lsteg_with_env(
        &[
            "data",
            "doctor",
            "--lang",
            "en",
            "--source",
            "en-wordlist-wordnik",
            "--format",
            "json",
            "--data-dir",
            data_dir.as_str(),
        ],
        &[],
    );
    assert_eq!(doctor_output.status.code(), Some(1));
    let stdout = stdout_string(&doctor_output);
    assert!(stdout.contains("\"mode\":\"data-doctor\""));
    assert!(stdout.contains("\"issue\":\"missing-manifest-for-state\""));
    assert!(stdout.contains("\"status\":\"unresolved\""));
}

#[test]
fn data_doctor_fix_removes_state_record_for_missing_manifest() {
    let data_dir = TempDataDir::create();
    let install_output = run_lsteg_with_env(
        &[
            "data",
            "install",
            "--lang",
            "en",
            "--source",
            "en-wordlist-wordnik",
            "--format",
            "json",
            "--data-dir",
            data_dir.as_str(),
        ],
        &[],
    );
    assert!(install_output.status.success());

    let manifest_path = std::path::Path::new(data_dir.as_str())
        .join("en")
        .join("en-wordlist-wordnik")
        .join("manifest.json");
    std::fs::remove_file(&manifest_path).expect("manifest should be removable");

    let doctor_output = run_lsteg_with_env(
        &[
            "data",
            "doctor",
            "--lang",
            "en",
            "--source",
            "en-wordlist-wordnik",
            "--fix",
            "--format",
            "json",
            "--data-dir",
            data_dir.as_str(),
        ],
        &[],
    );
    assert!(doctor_output.status.success());
    let stdout = stdout_string(&doctor_output);
    assert!(stdout.contains("\"status\":\"fixed\""));
    assert!(stdout.contains("\"unresolved\":0"));

    let status_output = run_lsteg_with_env(
        &[
            "data",
            "status",
            "--lang",
            "en",
            "--format",
            "json",
            "--data-dir",
            data_dir.as_str(),
        ],
        &[],
    );
    assert!(status_output.status.success());
    let status_stdout = stdout_string(&status_output);
    assert!(status_stdout.contains("\"items\":[]"));
}

#[test]
fn data_doctor_fix_imports_orphan_manifest_into_state() {
    let data_dir = TempDataDir::create();
    let install_output = run_lsteg_with_env(
        &[
            "data",
            "install",
            "--lang",
            "en",
            "--source",
            "en-wordlist-wordnik",
            "--format",
            "json",
            "--data-dir",
            data_dir.as_str(),
        ],
        &[],
    );
    assert!(install_output.status.success());

    let state_path = std::path::Path::new(data_dir.as_str()).join("state.json");
    std::fs::write(&state_path, "{\"schema_version\":1,\"installs\":[]}")
        .expect("state file should be writable");

    let doctor_output = run_lsteg_with_env(
        &[
            "data",
            "doctor",
            "--lang",
            "en",
            "--source",
            "en-wordlist-wordnik",
            "--fix",
            "--format",
            "json",
            "--data-dir",
            data_dir.as_str(),
        ],
        &[],
    );
    assert!(doctor_output.status.success());
    let stdout = stdout_string(&doctor_output);
    assert!(stdout.contains("\"issue\":\"orphan-manifest\""));
    assert!(stdout.contains("\"status\":\"fixed\""));

    let status_output = run_lsteg_with_env(
        &[
            "data",
            "status",
            "--lang",
            "en",
            "--format",
            "json",
            "--data-dir",
            data_dir.as_str(),
        ],
        &[],
    );
    assert!(status_output.status.success());
    let status_stdout = stdout_string(&status_output);
    assert!(status_stdout.contains("\"source_id\":\"en-wordlist-wordnik\""));
}

#[test]
fn data_clean_preview_keeps_state_and_files() {
    let data_dir = TempDataDir::create();
    let install_output = run_lsteg_with_env(
        &[
            "data",
            "install",
            "--lang",
            "en",
            "--source",
            "en-wordlist-wordnik",
            "--format",
            "json",
            "--data-dir",
            data_dir.as_str(),
        ],
        &[],
    );
    assert!(install_output.status.success());

    let source_dir = std::path::Path::new(data_dir.as_str())
        .join("en")
        .join("en-wordlist-wordnik");
    assert!(source_dir.exists());

    let clean_output = run_lsteg_with_env(
        &[
            "data",
            "clean",
            "--lang",
            "en",
            "--source",
            "en-wordlist-wordnik",
            "--format",
            "json",
            "--data-dir",
            data_dir.as_str(),
        ],
        &[],
    );
    assert!(clean_output.status.success());
    let clean_stdout = stdout_string(&clean_output);
    assert!(clean_stdout.contains("\"mode\":\"data-clean\""));
    assert!(clean_stdout.contains("\"apply\":false"));
    assert!(clean_stdout.contains("\"status\":\"would-remove\""));

    assert!(source_dir.exists());

    let status_output = run_lsteg_with_env(
        &[
            "data",
            "status",
            "--lang",
            "en",
            "--format",
            "json",
            "--data-dir",
            data_dir.as_str(),
        ],
        &[],
    );
    assert!(status_output.status.success());
    let status_stdout = stdout_string(&status_output);
    assert!(status_stdout.contains("\"source_id\":\"en-wordlist-wordnik\""));
}

#[test]
fn data_clean_apply_removes_state_and_source_directory() {
    let data_dir = TempDataDir::create();
    let install_output = run_lsteg_with_env(
        &[
            "data",
            "install",
            "--lang",
            "en",
            "--source",
            "en-wordlist-wordnik",
            "--format",
            "json",
            "--data-dir",
            data_dir.as_str(),
        ],
        &[],
    );
    assert!(install_output.status.success());

    let source_dir = std::path::Path::new(data_dir.as_str())
        .join("en")
        .join("en-wordlist-wordnik");
    assert!(source_dir.exists());

    let clean_output = run_lsteg_with_env(
        &[
            "data",
            "clean",
            "--lang",
            "en",
            "--source",
            "en-wordlist-wordnik",
            "--apply",
            "--format",
            "json",
            "--data-dir",
            data_dir.as_str(),
        ],
        &[],
    );
    assert!(clean_output.status.success());
    let clean_stdout = stdout_string(&clean_output);
    assert!(clean_stdout.contains("\"mode\":\"data-clean\""));
    assert!(clean_stdout.contains("\"apply\":true"));
    assert!(clean_stdout.contains("\"status\":\"removed\""));
    assert!(clean_stdout.contains("\"removed\":1"));
    assert!(clean_stdout.contains("\"state_removed\":1"));

    assert!(!source_dir.exists());

    let status_output = run_lsteg_with_env(
        &[
            "data",
            "status",
            "--lang",
            "en",
            "--format",
            "json",
            "--data-dir",
            data_dir.as_str(),
        ],
        &[],
    );
    assert!(status_output.status.success());
    let status_stdout = stdout_string(&status_output);
    assert!(status_stdout.contains("\"items\":[]"));
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
fn decode_accepts_proto_encode_json_contract_matrix() {
    let encode_output = run_lsteg(&["encode", "--message", "salam", "--format", "json"]);
    assert!(encode_output.status.success());
    let current_json = stdout_string(&encode_output);
    let legacy_json = as_legacy_proto_encode_json(&current_json);
    let extended_json = as_extended_proto_encode_json(&current_json);

    for candidate in [current_json, legacy_json, extended_json] {
        let decode_output = run_lsteg_with_stdin(&["decode", "--format", "json"], &candidate);
        assert!(decode_output.status.success());
        let decoded_json = stdout_string(&decode_output);
        assert!(decoded_json.contains("\"mode\":\"proto-decode\""));
        assert!(decoded_json.contains("\"payload_utf8\":\"salam\""));
    }
}

#[test]
fn analyze_accepts_proto_encode_json_contract_matrix() {
    let encode_output = run_lsteg(&["encode", "--message", "salam donya", "--format", "json"]);
    assert!(encode_output.status.success());
    let current_json = stdout_string(&encode_output);
    let legacy_json = as_legacy_proto_encode_json(&current_json);
    let extended_json = as_extended_proto_encode_json(&current_json);

    for candidate in [current_json, legacy_json, extended_json] {
        let analyze_output = run_lsteg_with_stdin(&["analyze", "--format", "json"], &candidate);
        assert!(analyze_output.status.success());
        let analysis_json = stdout_string(&analyze_output);
        assert!(analysis_json.contains("\"mode\":\"analyze\""));
        assert!(analysis_json.contains("\"integrity_ok\":true"));
        assert!(analysis_json.contains("\"payload_utf8\":\"salam donya\""));
    }
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
fn decode_text_input_roundtrip_from_german_plain_text_works() {
    let encode_output = run_lsteg(&["encode", "--lang", "de", "--message", "hallo welt"]);
    assert!(encode_output.status.success());
    let stego_text = stdout_string(&encode_output);

    let decode_output = run_lsteg_with_stdin(
        &["decode", "--lang", "de", "--text-input", "--format", "json"],
        &stego_text,
    );
    assert!(decode_output.status.success());

    let decoded_json = stdout_string(&decode_output);
    assert!(decoded_json.contains("\"language\":\"de\""));
    assert!(decoded_json.contains("\"payload_utf8\":\"hallo welt\""));
}

#[test]
fn decode_text_input_roundtrip_from_italian_plain_text_works() {
    let encode_output = run_lsteg(&["encode", "--lang", "it", "--message", "ciao mondo"]);
    assert!(encode_output.status.success());
    let stego_text = stdout_string(&encode_output);

    let decode_output = run_lsteg_with_stdin(
        &["decode", "--lang", "it", "--text-input", "--format", "json"],
        &stego_text,
    );
    assert!(decode_output.status.success());

    let decoded_json = stdout_string(&decode_output);
    assert!(decoded_json.contains("\"language\":\"it\""));
    assert!(decoded_json.contains("\"payload_utf8\":\"ciao mondo\""));
}

#[test]
fn encode_decode_with_file_io_roundtrip_for_italian() {
    let input_file = TempTextFile::create("ciao mondo");
    let encoded_file = TempTextFile::empty();
    let decoded_file = TempOutputFile::create();

    let encode_output = run_lsteg(&[
        "encode",
        "--lang",
        "it",
        "--input",
        input_file.as_str(),
        "--output",
        encoded_file.as_str(),
    ]);
    assert!(encode_output.status.success());

    let encoded_text =
        std::fs::read_to_string(encoded_file.as_str()).expect("encoded output file should exist");
    assert!(!encoded_text.trim().is_empty());

    let decode_output = run_lsteg(&[
        "decode",
        "--lang",
        "it",
        "--text-input",
        "--input",
        encoded_file.as_str(),
        "--output",
        decoded_file.as_str(),
        "--format",
        "json",
    ]);
    assert!(decode_output.status.success());

    let decoded_json = std::fs::read_to_string(decoded_file.as_str())
        .expect("decoded output file should be created");
    assert!(decoded_json.contains("\"language\":\"it\""));
    assert!(decoded_json.contains("\"payload_utf8\":\"ciao mondo\""));
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
fn decode_auto_roundtrip_from_english_plain_text_without_lang_flag_works() {
    let encode_output = run_lsteg(&["encode", "--lang", "en", "--message", "hello world"]);
    assert!(encode_output.status.success());
    let stego_text = stdout_string(&encode_output);

    let decode_output = run_lsteg_with_stdin(&["decode", "--format", "json"], &stego_text);
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
fn analyze_auto_detects_english_plain_text_language_without_lang_flag() {
    let encode_output = run_lsteg(&["encode", "--lang", "en", "--message", "hello world"]);
    assert!(encode_output.status.success());
    let stego_text = stdout_string(&encode_output);

    let analyze_output = run_lsteg_with_stdin(&["analyze", "--format", "json"], &stego_text);
    assert!(analyze_output.status.success());

    let analysis_json = stdout_string(&analyze_output);
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
fn validate_auto_detects_english_plain_text_language_without_lang_flag() {
    let encode_output = run_lsteg(&["encode", "--lang", "en", "--message", "hello"]);
    assert!(encode_output.status.success());
    let stego_text = stdout_string(&encode_output);

    let validate_output = run_lsteg_with_stdin(&["validate", "--format", "json"], &stego_text);
    assert!(validate_output.status.success());

    let validation_json = stdout_string(&validate_output);
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
    let stderr = stderr_string(&output);
    assert!(stderr.contains("error [LSTEG-CLI-ARG-001]:"));
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
fn encode_unknown_runtime_language_reports_supported_list() {
    let output = run_lsteg(&["encode", "--lang", "zz", "--message", "hello world"]);
    assert_eq!(output.status.code(), Some(1));

    let stderr = stderr_string(&output);
    assert!(stderr.contains("LSTEG-CLI-CFG-001"));
    assert!(stderr.contains("language 'zz' is not supported by runtime providers"));
    assert!(stderr.contains("supported: fa, en, de"));
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
fn decode_rejects_non_envelope_proto_trace_with_security_hint() {
    let proto_output = run_lsteg(&["proto-encode", "fa", "salam", "--json"]);
    assert!(proto_output.status.success());
    let proto_trace = stdout_string(&proto_output);

    let decode_output = run_lsteg_with_stdin(&["decode", "--format", "json"], &proto_trace);
    assert_eq!(decode_output.status.code(), Some(1));
    let stderr = stderr_string(&decode_output);
    assert!(stderr.contains("LSTEG-CLI-SEC-001"));
    assert!(stderr.contains("payload is not a valid secure envelope"));
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
fn env_secret_file_is_used_for_encode_and_decode() {
    let secret_file = TempSecretFile::create("env-file-secret");
    let encode_output = run_lsteg_with_env(
        &["encode", "--message", "salam", "--emit-trace"],
        &[
            ("LSTEG_SECRET", ""),
            ("LSTEG_SECRET_FILE", secret_file.as_str()),
        ],
    );
    assert!(encode_output.status.success());
    let trace_text = stdout_string(&encode_output);

    let decode_output = run_lsteg_with_env(
        &["decode", "--format", "json"],
        &[
            ("LSTEG_TRACE", &trace_text),
            ("LSTEG_SECRET", ""),
            ("LSTEG_SECRET_FILE", secret_file.as_str()),
        ],
    );
    assert!(decode_output.status.success());
    assert!(stdout_string(&decode_output).contains("\"payload_utf8\":\"salam\""));
}

#[test]
fn encode_rejects_ambiguous_secret_env_sources() {
    let secret_file = TempSecretFile::create("secret-from-file");
    let output = run_lsteg_with_env(
        &["encode", "--message", "salam"],
        &[
            ("LSTEG_SECRET", "secret-from-env"),
            ("LSTEG_SECRET_FILE", secret_file.as_str()),
        ],
    );
    assert_eq!(output.status.code(), Some(1));
    let stderr = stderr_string(&output);
    assert!(stderr.contains("LSTEG-CLI-CFG-001"));
    assert!(stderr.contains(
        "secret source is ambiguous; set only one of LSTEG_SECRET or LSTEG_SECRET_FILE, or override with --secret/--secret-file"
    ));
}

#[test]
fn cli_secret_overrides_env_secret_file() {
    let wrong_secret_file = TempSecretFile::create("wrong-env-file-secret");
    let encode_output = run_lsteg_with_env(
        &[
            "encode",
            "--message",
            "salam",
            "--emit-trace",
            "--secret",
            "cli-secret",
        ],
        &[("LSTEG_SECRET_FILE", wrong_secret_file.as_str())],
    );
    assert!(encode_output.status.success());
    let trace_text = stdout_string(&encode_output);

    let decode_output = run_lsteg_with_env(
        &["decode", "--format", "json", "--secret", "cli-secret"],
        &[
            ("LSTEG_TRACE", &trace_text),
            ("LSTEG_SECRET_FILE", wrong_secret_file.as_str()),
        ],
    );
    assert!(decode_output.status.success());
    assert!(stdout_string(&decode_output).contains("\"payload_utf8\":\"salam\""));
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
