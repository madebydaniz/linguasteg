use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process;

use sigstore_verify::trust_root::TrustedRoot;
use sigstore_verify::types::Bundle;
use sigstore_verify::{VerificationPolicy, verify};

#[derive(Debug)]
struct AddLangOptions {
    code: String,
    name: Option<String>,
    dry_run: bool,
    force: bool,
}

#[derive(Debug)]
struct VerifyReleaseOptions {
    artifact_path: PathBuf,
    bundle_path: PathBuf,
    tag: String,
    repo: String,
    workflow_path: String,
    issuer: String,
}

const DEFAULT_REPOSITORY: &str = "madebydaniz/linguasteg";
const DEFAULT_WORKFLOW_PATH: &str = ".github/workflows/release-binaries.yml";
const DEFAULT_OIDC_ISSUER: &str = "https://token.actions.githubusercontent.com";

fn main() {
    if let Err(error) = run() {
        eprintln!("error: {error}");
        process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let mut args = env::args().skip(1);
    let command = args.next();
    match command.as_deref() {
        Some("add-lang") => run_add_lang(parse_add_lang_options(args)?),
        Some("verify-release") => run_verify_release(parse_verify_release_options(args)?),
        Some("--help") | Some("-h") | None => {
            print_usage();
            Ok(())
        }
        Some(other) => Err(format!("unknown xtask command '{other}'")),
    }
}

fn parse_add_lang_options(
    mut args: impl Iterator<Item = String>,
) -> Result<AddLangOptions, String> {
    let mut code = None;
    let mut name = None;
    let mut dry_run = false;
    let mut force = false;

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--code" => code = Some(next_value(&mut args, "--code")?),
            "--name" => name = Some(next_value(&mut args, "--name")?),
            "--dry-run" => dry_run = true,
            "--force" => force = true,
            "--help" | "-h" => {
                print_add_lang_usage();
                process::exit(0);
            }
            _ => return Err(format!("unknown add-lang argument: {arg}")),
        }
    }

    let code = code.ok_or_else(|| "add-lang requires --code <lang-code>".to_string())?;
    Ok(AddLangOptions {
        code,
        name,
        dry_run,
        force,
    })
}

fn parse_verify_release_options(
    mut args: impl Iterator<Item = String>,
) -> Result<VerifyReleaseOptions, String> {
    let mut artifact_path = None;
    let mut bundle_path = None;
    let mut tag = None;
    let mut repo = DEFAULT_REPOSITORY.to_string();
    let mut workflow_path = DEFAULT_WORKFLOW_PATH.to_string();
    let mut issuer = DEFAULT_OIDC_ISSUER.to_string();

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--artifact" => {
                artifact_path = Some(PathBuf::from(next_value(&mut args, "--artifact")?))
            }
            "--bundle" => bundle_path = Some(PathBuf::from(next_value(&mut args, "--bundle")?)),
            "--tag" => tag = Some(next_value(&mut args, "--tag")?),
            "--repo" => repo = next_value(&mut args, "--repo")?,
            "--workflow-path" => workflow_path = next_value(&mut args, "--workflow-path")?,
            "--issuer" => issuer = next_value(&mut args, "--issuer")?,
            "--help" | "-h" => {
                print_verify_release_usage();
                process::exit(0);
            }
            _ => return Err(format!("unknown verify-release argument: {arg}")),
        }
    }

    let artifact_path =
        artifact_path.ok_or_else(|| "verify-release requires --artifact <path>".to_string())?;
    let bundle_path =
        bundle_path.ok_or_else(|| "verify-release requires --bundle <path>".to_string())?;
    let tag = tag.ok_or_else(|| "verify-release requires --tag <vX.Y.Z>".to_string())?;

    Ok(VerifyReleaseOptions {
        artifact_path,
        bundle_path,
        tag,
        repo,
        workflow_path,
        issuer,
    })
}

fn run_add_lang(options: AddLangOptions) -> Result<(), String> {
    let workspace_root = discover_workspace_root()?;
    let code = normalize_language_code(&options.code)?;
    let module_name = code.replace('-', "_");
    let pascal_name = options
        .name
        .as_deref()
        .map(to_pascal_case)
        .unwrap_or_else(|| to_pascal_case(&module_name.replace('_', " ")));
    let snake_name = to_snake_case(&pascal_name);
    let display_name = options
        .name
        .clone()
        .unwrap_or_else(|| pascal_to_words(&pascal_name));

    let models_module_path = workspace_root
        .join("linguasteg-models")
        .join("src")
        .join(format!("{module_name}.rs"));
    let models_lib_path = workspace_root
        .join("linguasteg-models")
        .join("src")
        .join("lib.rs");
    let facade_lib_path = workspace_root.join("linguasteg").join("src").join("lib.rs");

    let module_source = build_module_source(&code, &pascal_name, &snake_name, &display_name);

    if models_module_path.exists() && !options.force {
        return Err(format!(
            "module already exists at '{}'; rerun with --force to overwrite",
            models_module_path.to_string_lossy()
        ));
    }

    let original_models_lib = fs::read_to_string(&models_lib_path).map_err(|error| {
        format!(
            "failed to read models lib '{}': {error}",
            models_lib_path.to_string_lossy()
        )
    })?;
    let updated_models_lib = patch_models_lib(
        &original_models_lib,
        &module_name,
        &pascal_name,
        &snake_name,
    )?;

    let original_facade_lib = fs::read_to_string(&facade_lib_path).map_err(|error| {
        format!(
            "failed to read facade lib '{}': {error}",
            facade_lib_path.to_string_lossy()
        )
    })?;
    let updated_facade_lib = patch_facade_lib(&original_facade_lib, &pascal_name)?;

    if options.dry_run {
        println!("dry-run: add language scaffold");
        println!("- code: {code}");
        println!("- display: {display_name}");
        println!("- module file: {}", models_module_path.to_string_lossy());
        println!("- update: {}", models_lib_path.to_string_lossy());
        println!("- update: {}", facade_lib_path.to_string_lossy());
        print_manual_followup(&code, &pascal_name);
        return Ok(());
    }

    if let Some(parent) = models_module_path.parent() {
        fs::create_dir_all(parent).map_err(|error| {
            format!(
                "failed to create module directory '{}': {error}",
                parent.to_string_lossy()
            )
        })?;
    }
    fs::write(&models_module_path, module_source).map_err(|error| {
        format!(
            "failed to write module file '{}': {error}",
            models_module_path.to_string_lossy()
        )
    })?;
    fs::write(&models_lib_path, updated_models_lib).map_err(|error| {
        format!(
            "failed to update models lib '{}': {error}",
            models_lib_path.to_string_lossy()
        )
    })?;
    fs::write(&facade_lib_path, updated_facade_lib).map_err(|error| {
        format!(
            "failed to update facade lib '{}': {error}",
            facade_lib_path.to_string_lossy()
        )
    })?;

    println!("language scaffold created");
    println!("- code: {code}");
    println!("- display: {display_name}");
    println!("- module file: {}", models_module_path.to_string_lossy());
    println!("- updated: {}", models_lib_path.to_string_lossy());
    println!("- updated: {}", facade_lib_path.to_string_lossy());
    print_manual_followup(&code, &pascal_name);
    Ok(())
}

fn run_verify_release(options: VerifyReleaseOptions) -> Result<(), String> {
    if !options.artifact_path.exists() {
        return Err(format!(
            "artifact file does not exist: {}",
            options.artifact_path.to_string_lossy()
        ));
    }
    if !options.bundle_path.exists() {
        return Err(format!(
            "bundle file does not exist: {}",
            options.bundle_path.to_string_lossy()
        ));
    }

    let artifact = fs::read(&options.artifact_path).map_err(|error| {
        format!(
            "failed to read artifact '{}': {error}",
            options.artifact_path.to_string_lossy()
        )
    })?;
    let bundle_raw = fs::read_to_string(&options.bundle_path).map_err(|error| {
        format!(
            "failed to read bundle '{}': {error}",
            options.bundle_path.to_string_lossy()
        )
    })?;
    let bundle = Bundle::from_json(&bundle_raw).map_err(|error| {
        format!(
            "failed to parse bundle '{}': {error}",
            options.bundle_path.to_string_lossy()
        )
    })?;

    let expected_identity = build_workflow_identity(
        &options.repo,
        &options.workflow_path,
        &normalize_tag(&options.tag),
    );
    let policy = VerificationPolicy::default()
        .require_issuer(options.issuer.clone())
        .require_identity(expected_identity.clone());
    let trusted_root = TrustedRoot::production()
        .map_err(|error| format!("failed to load trusted root: {error}"))?;

    let result = verify(&artifact, &bundle, &policy, &trusted_root)
        .map_err(|error| format!("sigstore verification failed: {error}"))?;
    if !result.success {
        return Err("sigstore verification returned unsuccessful result".to_string());
    }

    println!("release verification passed");
    println!("- artifact: {}", options.artifact_path.to_string_lossy());
    println!("- bundle: {}", options.bundle_path.to_string_lossy());
    println!("- expected issuer: {}", options.issuer);
    println!("- expected identity: {expected_identity}");
    Ok(())
}

fn normalize_tag(tag: &str) -> String {
    let trimmed = tag.trim();
    if trimmed.starts_with('v') {
        trimmed.to_string()
    } else {
        format!("v{trimmed}")
    }
}

fn build_workflow_identity(repo: &str, workflow_path: &str, tag: &str) -> String {
    let normalized_workflow = workflow_path.trim_start_matches('/');
    format!("https://github.com/{repo}/{normalized_workflow}@refs/tags/{tag}")
}

fn patch_models_lib(
    original: &str,
    module_name: &str,
    pascal_name: &str,
    snake_name: &str,
) -> Result<String, String> {
    let mod_line = format!("pub mod {module_name};\n");
    let use_line = format!("use {module_name}::parse_{snake_name}_prototype_text;\n");
    let reexport_line = format!(
        "pub use {module_name}::{{{pascal_name}PrototypeConstraintChecker, {pascal_name}PrototypeLanguagePack, {pascal_name}PrototypeRealizer, {pascal_name}PrototypeSymbolicMapper}};\n"
    );
    let extractor_struct = format!(
        r#"
#[derive(Debug, Default, Clone, Copy)]
pub struct {pascal_name}PrototypeTextExtractor;

impl TextExtractor for {pascal_name}PrototypeTextExtractor {{
    fn extract_plans(&self, stego_text: &str) -> CoreResult<Vec<RealizationPlan>> {{
        parse_{snake_name}_prototype_text(stego_text)
    }}
}}

"#
    );

    let mut updated = original.to_string();
    updated = insert_before_once(&updated, "pub mod gateway;", &mod_line)?;
    updated = insert_before_once(&updated, "use linguasteg_core::{", &use_line)?;
    updated = insert_before_once(&updated, "pub use gateway::{", &reexport_line)?;

    if !updated.contains(&format!("pub struct {pascal_name}PrototypeTextExtractor")) {
        updated = insert_before_once(&updated, "fn select_farsi_text_body(", &extractor_struct)?;
    }

    Ok(updated)
}

fn patch_facade_lib(original: &str, pascal_name: &str) -> Result<String, String> {
    let language_exports = format!(
        "    {pascal_name}PrototypeConstraintChecker, {pascal_name}PrototypeLanguagePack, {pascal_name}PrototypeRealizer,\n    {pascal_name}PrototypeSymbolicMapper, {pascal_name}PrototypeTextExtractor,\n"
    );
    insert_before_once(original, "InMemoryGatewayRegistry", &language_exports)
}

fn insert_before_once(original: &str, marker: &str, snippet: &str) -> Result<String, String> {
    if original.contains(snippet.trim()) {
        return Ok(original.to_string());
    }

    let position = original
        .find(marker)
        .ok_or_else(|| format!("marker '{marker}' not found"))?;
    let mut updated = String::with_capacity(original.len() + snippet.len() + 1);
    updated.push_str(&original[..position]);
    if !updated.ends_with('\n') {
        updated.push('\n');
    }
    updated.push_str(snippet);
    updated.push_str(&original[position..]);
    Ok(updated)
}

fn build_module_source(
    code: &str,
    pascal_name: &str,
    snake_name: &str,
    display_name: &str,
) -> String {
    format!(
        r#"use linguasteg_core::{{CoreResult, RealizationPlan}};

/// {display_name} scaffold module generated by `cargo xtask add-lang`.
///
/// TODO:
/// - replace type aliases with native language pack implementation
/// - provide native templates, symbolic schemas, and parser
/// - update runtime registry in `linguasteg-cli/src/app/runtime.rs`
pub type {pascal_name}PrototypeLanguagePack = crate::en::EnglishPrototypeLanguagePack;
pub type {pascal_name}PrototypeConstraintChecker = crate::en::EnglishPrototypeConstraintChecker;
pub type {pascal_name}PrototypeRealizer = crate::en::EnglishPrototypeRealizer;
pub type {pascal_name}PrototypeSymbolicMapper = crate::en::EnglishPrototypeSymbolicMapper;

pub fn parse_{snake_name}_prototype_text(stego_text: &str) -> CoreResult<Vec<RealizationPlan>> {{
    crate::en::parse_english_prototype_text(stego_text)
}}

#[cfg(test)]
mod tests {{
    use super::parse_{snake_name}_prototype_text;

    #[test]
    fn parser_scaffold_delegates_to_english_prototype() {{
        let text =
            "the manager labels clear report. the architect in winter at the office, records manual.";
        let plans = parse_{snake_name}_prototype_text(text)
            .expect("scaffold parser should delegate successfully");
        assert_eq!(plans.len(), 2);
    }}
}}
"#
    )
    .replace("LANG_CODE_PLACEHOLDER", code)
}

fn normalize_language_code(input: &str) -> Result<String, String> {
    let code = input.trim().to_ascii_lowercase();
    if code.is_empty() {
        return Err("language code must not be empty".to_string());
    }
    if code.starts_with('-') || code.ends_with('-') || code.contains("--") {
        return Err(format!(
            "invalid language code '{input}' (expected lowercase code like 'fa', 'en', 'de')"
        ));
    }
    if !code
        .bytes()
        .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
    {
        return Err(format!(
            "invalid language code '{input}' (only lowercase letters, digits, and '-' are allowed)"
        ));
    }
    Ok(code)
}

fn to_pascal_case(input: &str) -> String {
    let mut out = String::new();
    for token in input
        .split(|ch: char| !ch.is_ascii_alphanumeric())
        .filter(|item| !item.is_empty())
    {
        let mut chars = token.chars();
        if let Some(first) = chars.next() {
            out.push(first.to_ascii_uppercase());
            for ch in chars {
                out.push(ch.to_ascii_lowercase());
            }
        }
    }
    if out.is_empty() {
        "Language".to_string()
    } else {
        out
    }
}

fn to_snake_case(input: &str) -> String {
    let mut out = String::new();
    for (index, ch) in input.chars().enumerate() {
        if ch.is_ascii_uppercase() && index > 0 {
            out.push('_');
        }
        out.push(ch.to_ascii_lowercase());
    }
    out
}

fn pascal_to_words(input: &str) -> String {
    let mut out = String::new();
    for (index, ch) in input.chars().enumerate() {
        if ch.is_ascii_uppercase() && index > 0 {
            out.push(' ');
        }
        out.push(ch);
    }
    out
}

fn next_value(args: &mut impl Iterator<Item = String>, flag: &str) -> Result<String, String> {
    args.next()
        .ok_or_else(|| format!("missing value for {flag}"))
}

fn discover_workspace_root() -> Result<PathBuf, String> {
    let cwd = env::current_dir().map_err(|error| format!("failed to read current dir: {error}"))?;
    let mut current = Some(cwd.as_path());
    while let Some(dir) = current {
        if is_workspace_root(dir) {
            return Ok(dir.to_path_buf());
        }
        current = dir.parent();
    }
    Err("unable to find workspace root (expected linguasteg-models/src/lib.rs)".to_string())
}

fn is_workspace_root(path: &Path) -> bool {
    path.join("Cargo.toml").exists()
        && path
            .join("linguasteg-models")
            .join("src")
            .join("lib.rs")
            .exists()
        && path.join("linguasteg").join("src").join("lib.rs").exists()
}

fn print_manual_followup(code: &str, pascal_name: &str) {
    println!();
    println!("manual follow-up checklist:");
    println!(
        "1) implement native model in linguasteg-models/src/{}.rs (replace english aliases)",
        code.replace('-', "_")
    );
    println!(
        "2) wire runtime provider in linguasteg-cli/src/app/runtime.rs for code '{}'",
        code
    );
    println!(
        "3) register data source in linguasteg-cli/assets/data_sources.json for '{}'",
        code
    );
    println!("4) add integration/golden tests for '{}'", code);
    println!(
        "5) optionally add '{}' exports wherever API surface needs it",
        pascal_name
    );
}

fn print_usage() {
    println!("LinguaSteg xtask");
    println!("Usage: cargo xtask <command> [options]");
    println!("Commands:");
    println!("  add-lang --code <lang-code> [--name <display-name>] [--dry-run] [--force]");
    println!(
        "  verify-release --artifact <path> --bundle <path> --tag <vX.Y.Z> [--repo <owner/repo>] [--workflow-path <path>] [--issuer <url>]"
    );
}

fn print_add_lang_usage() {
    println!(
        "Usage: cargo xtask add-lang --code <lang-code> [--name <display-name>] [--dry-run] [--force]"
    );
}

fn print_verify_release_usage() {
    println!(
        "Usage: cargo xtask verify-release --artifact <path> --bundle <path> --tag <vX.Y.Z> [--repo <owner/repo>] [--workflow-path <path>] [--issuer <url>]"
    );
}
