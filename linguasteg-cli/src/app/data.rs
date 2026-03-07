use std::collections::{HashMap, HashSet};
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use super::dataset::{
    DatasetArtifactMetadata, LexiconVariantCatalog, load_lexicon_dataset_artifact,
};
use super::types::{
    CliError, DataArtifactValidateOptions, DataCleanOptions, DataCommand, DataDoctorOptions,
    DataExportManifestOptions, DataImportManifestOptions, DataInstallOptions, DataListOptions,
    DataPinOptions, DataStatusOptions, DataVerifyOptions, OutputFormat, ProtoTarget,
};

const DATA_STATE_FILE: &str = "state.json";
const DATA_SOURCES_MANIFEST: &str = include_str!("../../assets/data_sources.json");
const MANIFEST_SCHEMA_VERSION: u8 = 1;
const MAX_ARTIFACT_BYTES: usize = 64 * 1024 * 1024;
const STARTER_DATASET_FILE: &str = "dataset.json";

#[derive(Debug, Clone, Deserialize)]
struct DataSourceCatalogRaw {
    schema_version: u8,
    sources: Vec<DataSourceRaw>,
}

#[derive(Debug, Clone, Deserialize)]
struct DataSourceRaw {
    id: String,
    language: String,
    source_url: String,
    license: String,
    version: String,
    checksum_sha256: Option<String>,
    recommended: bool,
}

#[derive(Debug, Clone)]
struct DataSource {
    id: String,
    language: ProtoTarget,
    source_url: String,
    license: String,
    version: String,
    checksum_sha256: Option<String>,
    recommended: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct DataState {
    schema_version: u8,
    installs: Vec<InstallRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct InstallRecord {
    language: String,
    source_id: String,
    version: String,
    installed_at_epoch_sec: u64,
}

#[derive(Debug, Clone, Serialize)]
struct DataListItem {
    language: String,
    source_id: String,
    version: String,
    source_url: String,
    license: String,
    checksum_sha256: Option<String>,
    recommended: bool,
    installed: bool,
}

#[derive(Debug, Clone, Serialize)]
struct DataInstallItem {
    language: String,
    source_id: String,
    version: String,
    status: String,
    manifest_path: String,
    starter_dataset_path: String,
    artifact_path: Option<String>,
    artifact_sha256: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
struct DataListResponse {
    mode: &'static str,
    data_dir: String,
    items: Vec<DataListItem>,
}

#[derive(Debug, Clone, Serialize)]
struct DataInstallResponse {
    mode: &'static str,
    data_dir: String,
    note: &'static str,
    items: Vec<DataInstallItem>,
}

#[derive(Debug, Clone, Serialize)]
struct DataStatusItem {
    language: String,
    source_id: String,
    version: String,
    installed_at_epoch_sec: u64,
    source_url: Option<String>,
    license: Option<String>,
    manifest_path: String,
    manifest_exists: bool,
    status: &'static str,
    artifact_path: Option<String>,
    artifact_exists: Option<bool>,
    artifact_sha256: Option<String>,
    manifest_error: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
struct DataStatusResponse {
    mode: &'static str,
    data_dir: String,
    items: Vec<DataStatusItem>,
}

#[derive(Debug, Clone, Serialize)]
struct DataVerifyItem {
    language: String,
    source_id: String,
    status: &'static str,
    verifiable: bool,
    manifest_path: String,
    artifact_path: Option<String>,
    expected_sha256: Option<String>,
    actual_sha256: Option<String>,
    reason: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
struct DataVerifyResponse {
    mode: &'static str,
    data_dir: String,
    integrity_ok: bool,
    passed: usize,
    failed: usize,
    skipped: usize,
    items: Vec<DataVerifyItem>,
}

#[derive(Debug, Clone, Serialize)]
struct DataDoctorItem {
    language: String,
    source_id: String,
    issue: &'static str,
    status: &'static str,
    path: String,
    message: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
struct DataDoctorResponse {
    mode: &'static str,
    data_dir: String,
    fix_applied: bool,
    detected: usize,
    fixed: usize,
    unresolved: usize,
    items: Vec<DataDoctorItem>,
}

#[derive(Debug, Clone, Serialize)]
struct DataCleanItem {
    language: String,
    source_id: String,
    status: &'static str,
    path: String,
}

#[derive(Debug, Clone, Serialize)]
struct DataCleanResponse {
    mode: &'static str,
    data_dir: String,
    apply: bool,
    removed: usize,
    state_removed: usize,
    items: Vec<DataCleanItem>,
}

#[derive(Debug, Clone, Serialize)]
struct DataPinItem {
    language: String,
    source_id: String,
    status: &'static str,
    manifest_path: String,
    artifact_path: Option<String>,
    pinned_sha256: Option<String>,
    reason: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
struct DataPinResponse {
    mode: &'static str,
    data_dir: String,
    updated: usize,
    failed: usize,
    items: Vec<DataPinItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct DataExportEntry {
    language: String,
    source_id: String,
    version: String,
    installed_at_epoch_sec: u64,
    source_url: Option<String>,
    license: Option<String>,
    checksum_sha256: Option<String>,
    manifest_path: String,
    manifest_exists: bool,
    artifact_path: Option<String>,
    artifact_sha256: Option<String>,
    manifest_error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct DataExportSnapshot {
    schema_version: u8,
    generated_at_epoch_sec: u64,
    data_dir: String,
    entries: Vec<DataExportEntry>,
}

#[derive(Debug, Clone, Serialize)]
struct DataExportResponse {
    mode: &'static str,
    output_path: String,
    entry_count: usize,
}

#[derive(Debug, Clone, Serialize)]
struct DataImportItem {
    language: String,
    source_id: String,
    status: &'static str,
    manifest_path: String,
}

#[derive(Debug, Clone, Serialize)]
struct DataImportResponse {
    mode: &'static str,
    data_dir: String,
    imported: usize,
    updated: usize,
    items: Vec<DataImportItem>,
}

#[derive(Debug, Clone, Serialize)]
struct DataArtifactValidateResponse {
    mode: &'static str,
    input: String,
    language: String,
    valid: bool,
    kind: Option<String>,
    schema_version: Option<u8>,
    dataset_language: Option<String>,
    entry_count: Option<usize>,
    error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct InstalledSourceManifest {
    schema_version: u8,
    language: String,
    source_id: String,
    source_url: String,
    version: String,
    license: String,
    checksum_sha256: Option<String>,
    artifact_url: Option<String>,
    artifact_path: Option<String>,
    artifact_sha256: Option<String>,
    artifact_bytes: Option<usize>,
    artifact_dataset_kind: Option<String>,
    artifact_dataset_schema_version: Option<u8>,
    artifact_dataset_language: Option<String>,
    artifact_dataset_entry_count: Option<usize>,
    installed_at_epoch_sec: u64,
}

#[derive(Debug, Clone)]
pub(crate) struct ActiveDataSourceSelection {
    pub(crate) source_id: String,
}

pub(crate) fn resolve_active_data_source_selection(
    target: ProtoTarget,
    source_id: Option<&str>,
    data_dir: Option<&str>,
) -> Result<Option<ActiveDataSourceSelection>, CliError> {
    let data_dir = resolve_data_dir(data_dir);
    let state = load_data_state(&data_dir)?;
    let mut installs = state
        .installs
        .iter()
        .filter(|record| record.language == target.as_str())
        .collect::<Vec<_>>();

    if let Some(requested_source_id) = source_id {
        let record = installs
            .iter()
            .find(|record| record.source_id == requested_source_id)
            .copied()
            .ok_or_else(|| {
                CliError::config(format!(
                    "data source '{}' is not installed for language '{}' (run 'lsteg data install --lang {} --source {}')",
                    requested_source_id,
                    target.as_str(),
                    target.as_str(),
                    requested_source_id
                ))
            })?;
        return Ok(Some(ActiveDataSourceSelection {
            source_id: record.source_id.clone(),
        }));
    }

    if installs.is_empty() {
        return Ok(None);
    }
    if installs.len() == 1 {
        let record = installs[0];
        return Ok(Some(ActiveDataSourceSelection {
            source_id: record.source_id.clone(),
        }));
    }

    let recommended_source_id = load_data_sources()?
        .into_iter()
        .find(|source| source.language == target && source.recommended)
        .map(|source| source.id);
    if let Some(recommended_source_id) = recommended_source_id {
        if let Some(record) = installs
            .iter()
            .find(|record| record.source_id == recommended_source_id)
        {
            return Ok(Some(ActiveDataSourceSelection {
                source_id: record.source_id.clone(),
            }));
        }
    }

    installs.sort_by(|left, right| {
        right
            .installed_at_epoch_sec
            .cmp(&left.installed_at_epoch_sec)
            .then_with(|| left.source_id.cmp(&right.source_id))
    });
    let record = installs[0];
    Ok(Some(ActiveDataSourceSelection {
        source_id: record.source_id.clone(),
    }))
}

pub(crate) fn resolve_effective_data_dir(explicit: Option<&str>) -> PathBuf {
    resolve_data_dir(explicit)
}

pub(crate) fn resolve_active_data_source_variant_catalog(
    target: ProtoTarget,
    source_id: Option<&str>,
    data_dir: Option<&str>,
) -> Result<Option<LexiconVariantCatalog>, CliError> {
    let Some(source_id) = source_id else {
        return Ok(None);
    };

    let data_dir = resolve_data_dir(data_dir);
    let source_dir = data_dir.join(target.as_str()).join(source_id);
    let manifest_path = source_dir.join("manifest.json");
    if !manifest_path.exists() {
        return Ok(None);
    }

    let manifest_raw = fs::read_to_string(&manifest_path).map_err(|error| {
        CliError::io(
            "failed to read source manifest",
            Some(&manifest_path.to_string_lossy()),
            error,
        )
    })?;
    let manifest: InstalledSourceManifest =
        serde_json::from_str(&manifest_raw).map_err(|error| {
            CliError::config(format!(
                "failed to parse source manifest '{}': {error}",
                manifest_path.to_string_lossy()
            ))
        })?;

    let Some(artifact_raw_path) = manifest.artifact_path.as_deref() else {
        return Ok(None);
    };
    let artifact_path = if Path::new(artifact_raw_path).is_absolute() {
        PathBuf::from(artifact_raw_path)
    } else {
        source_dir.join(artifact_raw_path)
    };
    if !artifact_path.exists() {
        return Ok(None);
    }

    let bytes = read_local_file_limited(&artifact_path)?;
    let dataset = load_lexicon_dataset_artifact(target.as_str(), &bytes).map_err(|reason| {
        CliError::config(format!(
            "installed artifact for source '{}' is not valid: {reason}",
            source_id
        ))
    })?;
    Ok(dataset.map(|item| item.variant_catalog()))
}

pub(crate) fn run_data_command(command: DataCommand) -> Result<(), CliError> {
    match command {
        DataCommand::List(options) => run_data_list(options),
        DataCommand::Status(options) => run_data_status(options),
        DataCommand::Verify(options) => run_data_verify(options),
        DataCommand::Doctor(options) => run_data_doctor(options),
        DataCommand::Clean(options) => run_data_clean(options),
        DataCommand::Pin(options) => run_data_pin(options),
        DataCommand::ArtifactValidate(options) => run_data_artifact_validate(options),
        DataCommand::ExportManifest(options) => run_data_export_manifest(options),
        DataCommand::ImportManifest(options) => run_data_import_manifest(options),
        DataCommand::Install(options) => run_data_install(options, false),
        DataCommand::Update(options) => run_data_install(options, true),
    }
}

fn run_data_artifact_validate(options: DataArtifactValidateOptions) -> Result<(), CliError> {
    let bytes = read_artifact_bytes(&options.input_path)?;
    let parsed = load_lexicon_dataset_artifact(options.target.as_str(), &bytes);
    let response = match parsed {
        Ok(Some(dataset)) => {
            let metadata = dataset.metadata();
            DataArtifactValidateResponse {
                mode: "data-artifact-validate",
                input: options.input_path.clone(),
                language: options.target.as_str().to_string(),
                valid: true,
                kind: Some(metadata.kind),
                schema_version: Some(metadata.schema_version),
                dataset_language: Some(metadata.language),
                entry_count: Some(metadata.entry_count),
                error: None,
            }
        }
        Ok(None) => DataArtifactValidateResponse {
            mode: "data-artifact-validate",
            input: options.input_path.clone(),
            language: options.target.as_str().to_string(),
            valid: false,
            kind: None,
            schema_version: None,
            dataset_language: None,
            entry_count: None,
            error: Some(
                "artifact is not a linguasteg lexicon dataset (expected kind 'linguasteg-lexicon-v1')"
                    .to_string(),
            ),
        },
        Err(reason) => DataArtifactValidateResponse {
            mode: "data-artifact-validate",
            input: options.input_path.clone(),
            language: options.target.as_str().to_string(),
            valid: false,
            kind: Some("linguasteg-lexicon-v1".to_string()),
            schema_version: None,
            dataset_language: None,
            entry_count: None,
            error: Some(reason),
        },
    };

    if matches!(options.format, OutputFormat::Json) {
        let output = serde_json::to_string(&response).map_err(|error| {
            CliError::internal(format!(
                "failed to serialize data artifact validate response: {error}"
            ))
        })?;
        println!("{output}");
    } else {
        println!("data artifact validate:");
        println!("input: {}", response.input);
        println!("language: {}", response.language);
        println!("valid: {}", if response.valid { "yes" } else { "no" });
        if let Some(kind) = &response.kind {
            println!("kind: {kind}");
        }
        if let Some(schema_version) = response.schema_version {
            println!("schema_version: {schema_version}");
        }
        if let Some(dataset_language) = &response.dataset_language {
            println!("dataset_language: {dataset_language}");
        }
        if let Some(entry_count) = response.entry_count {
            println!("entry_count: {entry_count}");
        }
        if let Some(error) = &response.error {
            println!("error: {error}");
        }
    }

    if response.valid {
        Ok(())
    } else {
        let reason = response
            .error
            .unwrap_or_else(|| "artifact validation failed".to_string());
        Err(CliError::config(format!(
            "artifact validation failed: {reason}"
        )))
    }
}

fn run_data_list(options: DataListOptions) -> Result<(), CliError> {
    let sources = load_data_sources()?;
    let data_dir = resolve_data_dir(options.data_dir.as_deref());
    let state = load_data_state(&data_dir)?;
    let items = sources
        .iter()
        .filter(|source| {
            options
                .target
                .as_ref()
                .is_none_or(|target| source.language.as_str() == target.as_str())
        })
        .map(|source| {
            let installed = state.installs.iter().any(|record| {
                record.language == source.language.as_str() && record.source_id == source.id
            });
            DataListItem {
                language: source.language.as_str().to_string(),
                source_id: source.id.clone(),
                version: source.version.clone(),
                source_url: source.source_url.clone(),
                license: source.license.clone(),
                checksum_sha256: source.checksum_sha256.clone(),
                recommended: source.recommended,
                installed,
            }
        })
        .collect::<Vec<_>>();

    if matches!(options.format, OutputFormat::Json) {
        let payload = DataListResponse {
            mode: "data-list",
            data_dir: data_dir.to_string_lossy().to_string(),
            items,
        };
        let output = serde_json::to_string(&payload).map_err(|error| {
            CliError::internal(format!("failed to serialize data list: {error}"))
        })?;
        println!("{output}");
        return Ok(());
    }

    println!("data sources:");
    println!("data dir: {}", data_dir.to_string_lossy());
    for item in items {
        println!(
            "- {}/{} version:{} license:{} recommended:{} installed:{}",
            item.language,
            item.source_id,
            item.version,
            item.license,
            if item.recommended { "yes" } else { "no" },
            if item.installed { "yes" } else { "no" }
        );
    }
    Ok(())
}

fn run_data_install(options: DataInstallOptions, force_refresh: bool) -> Result<(), CliError> {
    let sources = load_data_sources()?;
    if options.source_id.is_some() && options.targets.len() != 1 {
        return Err(CliError::usage(
            "--source can be used only with a single language in --lang".to_string(),
        ));
    }
    if options.artifact_url.is_some() && options.targets.len() != 1 {
        return Err(CliError::usage(
            "--artifact-url can be used only with a single language in --lang".to_string(),
        ));
    }

    let data_dir = resolve_data_dir(options.data_dir.as_deref());
    fs::create_dir_all(&data_dir).map_err(|error| {
        CliError::io(
            "failed to create data directory",
            Some(&data_dir.to_string_lossy()),
            error,
        )
    })?;

    let mut state = load_data_state(&data_dir)?;
    let now = unix_epoch_seconds()?;
    let mut items = Vec::with_capacity(options.targets.len());

    for target in &options.targets {
        let source =
            resolve_source_for_target(&sources, target.clone(), options.source_id.as_deref())?;
        let status = upsert_install_state(&mut state, source, now, force_refresh);
        let starter_dataset_path = ensure_starter_dataset_template_exists(&data_dir, source)?;
        let artifact = if let Some(url) = options.artifact_url.as_deref() {
            Some(fetch_and_store_artifact(&data_dir, source, url)?)
        } else if force_refresh {
            load_local_dataset_artifact(source, &starter_dataset_path)?
        } else {
            None
        };
        let manifest_path = write_install_manifest(
            &data_dir,
            source,
            now,
            options.artifact_url.as_deref(),
            artifact.as_ref(),
        )?;
        items.push(DataInstallItem {
            language: source.language.as_str().to_string(),
            source_id: source.id.clone(),
            version: source.version.clone(),
            status: status.to_string(),
            manifest_path: manifest_path.to_string_lossy().to_string(),
            starter_dataset_path: starter_dataset_path.to_string_lossy().to_string(),
            artifact_path: artifact
                .as_ref()
                .map(|item| item.path.to_string_lossy().to_string()),
            artifact_sha256: artifact.as_ref().map(|item| item.sha256.clone()),
        });
    }

    save_data_state(&data_dir, &state)?;

    let mode = if force_refresh {
        "data-update"
    } else {
        "data-install"
    };
    let note = if force_refresh {
        "metadata was refreshed; a valid local starter dataset is activated automatically when present"
    } else {
        "metadata and starter dataset template were prepared; edit starter dataset and run 'lsteg data update --lang <code>' to activate variants"
    };

    if matches!(options.format, OutputFormat::Json) {
        let payload = DataInstallResponse {
            mode,
            data_dir: data_dir.to_string_lossy().to_string(),
            note,
            items,
        };
        let output = serde_json::to_string(&payload).map_err(|error| {
            CliError::internal(format!(
                "failed to serialize data install response: {error}"
            ))
        })?;
        println!("{output}");
        return Ok(());
    }

    println!("{mode}:");
    println!("data dir: {}", data_dir.to_string_lossy());
    println!("note: {note}");
    for item in items {
        let artifact_suffix = item
            .artifact_path
            .as_ref()
            .map_or_else(String::new, |path| format!(" artifact:{path}"));
        println!(
            "- {}/{} version:{} status:{} manifest:{} starter_dataset:{}{}",
            item.language,
            item.source_id,
            item.version,
            item.status,
            item.manifest_path,
            item.starter_dataset_path,
            artifact_suffix
        );
        if let Some(sha256) = &item.artifact_sha256 {
            println!("  sha256: {sha256}");
        }
    }
    Ok(())
}

fn run_data_status(options: DataStatusOptions) -> Result<(), CliError> {
    let sources = load_data_sources()?;
    let data_dir = resolve_data_dir(options.data_dir.as_deref());
    let state = load_data_state(&data_dir)?;
    let mut items = state
        .installs
        .iter()
        .filter(|record| {
            options
                .target
                .as_ref()
                .is_none_or(|target| record.language == target.as_str())
        })
        .map(|record| {
            let source = sources.iter().find(|item| {
                item.id == record.source_id && item.language.as_str() == record.language
            });
            build_status_item(&data_dir, record, source)
        })
        .collect::<Vec<_>>();
    items.sort_by(|left, right| {
        left.language
            .cmp(&right.language)
            .then(left.source_id.cmp(&right.source_id))
    });

    if matches!(options.format, OutputFormat::Json) {
        let payload = DataStatusResponse {
            mode: "data-status",
            data_dir: data_dir.to_string_lossy().to_string(),
            items,
        };
        let output = serde_json::to_string(&payload).map_err(|error| {
            CliError::internal(format!("failed to serialize data status: {error}"))
        })?;
        println!("{output}");
        return Ok(());
    }

    println!("data status:");
    println!("data dir: {}", data_dir.to_string_lossy());
    for item in items {
        let artifact_path = item.artifact_path.as_ref().map_or("-", String::as_str);
        let artifact_exists = item
            .artifact_exists
            .map_or("-", |value| if value { "yes" } else { "no" });
        println!(
            "- {}/{} version:{} status:{} manifest:{} artifact:{} artifact_exists:{}",
            item.language,
            item.source_id,
            item.version,
            item.status,
            item.manifest_path,
            artifact_path,
            artifact_exists
        );
        if let Some(error) = &item.manifest_error {
            println!("  manifest_error: {error}");
        }
    }
    Ok(())
}

fn run_data_verify(options: DataVerifyOptions) -> Result<(), CliError> {
    let sources = load_data_sources()?;
    let data_dir = resolve_data_dir(options.data_dir.as_deref());
    let state = load_data_state(&data_dir)?;

    if let Some(source_id) = options.source_id.as_deref() {
        let has_source = state.installs.iter().any(|record| {
            record.source_id == source_id
                && options
                    .target
                    .as_ref()
                    .is_none_or(|target| record.language == target.as_str())
        });
        if !has_source {
            return Err(CliError::config(format!(
                "installed source '{}' was not found in data state",
                source_id
            )));
        }
    }

    let mut items = state
        .installs
        .iter()
        .filter(|record| {
            options
                .target
                .as_ref()
                .is_none_or(|target| record.language == target.as_str())
                && options
                    .source_id
                    .as_deref()
                    .is_none_or(|source_id| source_id == record.source_id)
        })
        .map(|record| {
            let source = sources.iter().find(|item| {
                item.id == record.source_id && item.language.as_str() == record.language
            });
            build_verify_item(&data_dir, record, source)
        })
        .collect::<Vec<_>>();
    items.sort_by(|left, right| {
        left.language
            .cmp(&right.language)
            .then(left.source_id.cmp(&right.source_id))
    });

    let mut passed = 0_usize;
    let mut failed = 0_usize;
    let mut skipped = 0_usize;
    for item in &items {
        match item.status {
            "ok" => passed += 1,
            "skipped-no-artifact" | "skipped-no-checksum" => skipped += 1,
            _ => failed += 1,
        }
    }
    let integrity_ok = failed == 0;

    if matches!(options.format, OutputFormat::Json) {
        let payload = DataVerifyResponse {
            mode: "data-verify",
            data_dir: data_dir.to_string_lossy().to_string(),
            integrity_ok,
            passed,
            failed,
            skipped,
            items,
        };
        let output = serde_json::to_string(&payload).map_err(|error| {
            CliError::internal(format!("failed to serialize data verify: {error}"))
        })?;
        println!("{output}");
    } else {
        println!("data verify:");
        println!("data dir: {}", data_dir.to_string_lossy());
        println!(
            "summary: passed:{} failed:{} skipped:{} integrity_ok:{}",
            passed,
            failed,
            skipped,
            if integrity_ok { "yes" } else { "no" }
        );
        for item in &items {
            let artifact_path = item.artifact_path.as_ref().map_or("-", String::as_str);
            println!(
                "- {}/{} status:{} verifiable:{} manifest:{} artifact:{}",
                item.language,
                item.source_id,
                item.status,
                if item.verifiable { "yes" } else { "no" },
                item.manifest_path,
                artifact_path
            );
            if let Some(reason) = &item.reason {
                println!("  reason: {reason}");
            }
            if let Some(expected) = &item.expected_sha256 {
                println!("  expected_sha256: {expected}");
            }
            if let Some(actual) = &item.actual_sha256 {
                println!("  actual_sha256: {actual}");
            }
        }
    }

    if integrity_ok {
        Ok(())
    } else {
        Err(CliError::domain(format!(
            "data verification failed for {failed} source(s)"
        )))
    }
}

fn run_data_doctor(options: DataDoctorOptions) -> Result<(), CliError> {
    let data_dir = resolve_data_dir(options.data_dir.as_deref());
    let mut state = load_data_state(&data_dir)?;
    let mut items = Vec::new();
    let mut remove_keys: HashSet<(String, String)> = HashSet::new();
    let mut upsert_records: HashMap<(String, String), InstallRecord> = HashMap::new();

    for record in &state.installs {
        if options
            .target
            .as_ref()
            .is_some_and(|target| record.language != target.as_str())
        {
            continue;
        }
        if options
            .source_id
            .as_deref()
            .is_some_and(|source_id| source_id != record.source_id)
        {
            continue;
        }

        let manifest_path = data_dir
            .join(&record.language)
            .join(&record.source_id)
            .join("manifest.json");
        if !manifest_path.exists() {
            let status = if options.fix {
                remove_keys.insert((record.language.clone(), record.source_id.clone()));
                "fixed"
            } else {
                "unresolved"
            };
            items.push(DataDoctorItem {
                language: record.language.clone(),
                source_id: record.source_id.clone(),
                issue: "missing-manifest-for-state",
                status,
                path: manifest_path.to_string_lossy().to_string(),
                message: None,
            });
            continue;
        }

        let manifest = match read_manifest_for_doctor(&manifest_path) {
            Ok(value) => value,
            Err(message) => {
                items.push(DataDoctorItem {
                    language: record.language.clone(),
                    source_id: record.source_id.clone(),
                    issue: "invalid-manifest",
                    status: "unresolved",
                    path: manifest_path.to_string_lossy().to_string(),
                    message: Some(message),
                });
                continue;
            }
        };

        if manifest.language != record.language || manifest.source_id != record.source_id {
            items.push(DataDoctorItem {
                language: record.language.clone(),
                source_id: record.source_id.clone(),
                issue: "manifest-record-mismatch",
                status: "unresolved",
                path: manifest_path.to_string_lossy().to_string(),
                message: Some(format!(
                    "manifest points to {}/{}",
                    manifest.language, manifest.source_id
                )),
            });
            continue;
        }

        if manifest.version != record.version
            || manifest.installed_at_epoch_sec != record.installed_at_epoch_sec
        {
            let status = if options.fix {
                upsert_records.insert(
                    (record.language.clone(), record.source_id.clone()),
                    InstallRecord {
                        language: record.language.clone(),
                        source_id: record.source_id.clone(),
                        version: manifest.version.clone(),
                        installed_at_epoch_sec: manifest.installed_at_epoch_sec,
                    },
                );
                "fixed"
            } else {
                "unresolved"
            };
            items.push(DataDoctorItem {
                language: record.language.clone(),
                source_id: record.source_id.clone(),
                issue: "state-drift",
                status,
                path: manifest_path.to_string_lossy().to_string(),
                message: Some("state record differs from manifest metadata".to_string()),
            });
        }
    }

    let manifest_candidates =
        collect_manifest_candidates(&data_dir, options.target, options.source_id.as_deref())?;
    for candidate in manifest_candidates {
        let key = (candidate.language.clone(), candidate.source_id.clone());
        let in_state = state.installs.iter().any(|record| {
            record.language == candidate.language
                && record.source_id == candidate.source_id
                && !remove_keys.contains(&(record.language.clone(), record.source_id.clone()))
        }) || upsert_records.contains_key(&key);
        if in_state {
            continue;
        }

        let manifest = match read_manifest_for_doctor(&candidate.manifest_path) {
            Ok(value) => value,
            Err(message) => {
                items.push(DataDoctorItem {
                    language: candidate.language.clone(),
                    source_id: candidate.source_id.clone(),
                    issue: "invalid-manifest",
                    status: "unresolved",
                    path: candidate.manifest_path.to_string_lossy().to_string(),
                    message: Some(message),
                });
                continue;
            }
        };
        if manifest.language != candidate.language || manifest.source_id != candidate.source_id {
            items.push(DataDoctorItem {
                language: candidate.language.clone(),
                source_id: candidate.source_id.clone(),
                issue: "manifest-path-mismatch",
                status: "unresolved",
                path: candidate.manifest_path.to_string_lossy().to_string(),
                message: Some(format!(
                    "manifest path suggests {}/{}, payload is {}/{}",
                    candidate.language, candidate.source_id, manifest.language, manifest.source_id
                )),
            });
            continue;
        }

        let status = if options.fix {
            upsert_records.insert(
                (manifest.language.clone(), manifest.source_id.clone()),
                InstallRecord {
                    language: manifest.language.clone(),
                    source_id: manifest.source_id.clone(),
                    version: manifest.version.clone(),
                    installed_at_epoch_sec: manifest.installed_at_epoch_sec,
                },
            );
            "fixed"
        } else {
            "unresolved"
        };
        items.push(DataDoctorItem {
            language: candidate.language,
            source_id: candidate.source_id,
            issue: "orphan-manifest",
            status,
            path: candidate.manifest_path.to_string_lossy().to_string(),
            message: None,
        });
    }

    let mut state_changed = false;
    if options.fix {
        let before_len = state.installs.len();
        state.installs.retain(|record| {
            !remove_keys.contains(&(record.language.clone(), record.source_id.clone()))
        });
        if state.installs.len() != before_len {
            state_changed = true;
        }
        for record in upsert_records.values() {
            let _ = upsert_install_record(
                &mut state,
                &record.language,
                &record.source_id,
                &record.version,
                record.installed_at_epoch_sec,
            );
            state_changed = true;
        }
        if state_changed {
            save_data_state(&data_dir, &state)?;
        }
    }

    items.sort_by(|left, right| {
        left.language
            .cmp(&right.language)
            .then(left.source_id.cmp(&right.source_id))
            .then(left.issue.cmp(right.issue))
    });
    let detected = items.len();
    let fixed = items.iter().filter(|item| item.status == "fixed").count();
    let unresolved = items
        .iter()
        .filter(|item| item.status == "unresolved")
        .count();

    if matches!(options.format, OutputFormat::Json) {
        let payload = DataDoctorResponse {
            mode: "data-doctor",
            data_dir: data_dir.to_string_lossy().to_string(),
            fix_applied: options.fix,
            detected,
            fixed,
            unresolved,
            items,
        };
        let output = serde_json::to_string(&payload).map_err(|error| {
            CliError::internal(format!("failed to serialize data doctor response: {error}"))
        })?;
        println!("{output}");
    } else {
        println!("data doctor:");
        println!("data dir: {}", data_dir.to_string_lossy());
        println!(
            "summary: detected:{} fixed:{} unresolved:{}",
            detected, fixed, unresolved
        );
        for item in &items {
            println!(
                "- {}/{} issue:{} status:{} path:{}",
                item.language, item.source_id, item.issue, item.status, item.path
            );
            if let Some(message) = &item.message {
                println!("  message: {message}");
            }
        }
    }

    if unresolved == 0 {
        Ok(())
    } else {
        Err(CliError::domain(format!(
            "data doctor found {unresolved} unresolved issue(s)"
        )))
    }
}

fn run_data_clean(options: DataCleanOptions) -> Result<(), CliError> {
    let data_dir = resolve_data_dir(options.data_dir.as_deref());
    let mut state = load_data_state(&data_dir)?;

    if let Some(source_id) = options.source_id.as_deref() {
        let has_source = state.installs.iter().any(|record| {
            record.source_id == source_id
                && options
                    .target
                    .as_ref()
                    .is_none_or(|target| record.language == target.as_str())
        });
        if !has_source {
            return Err(CliError::config(format!(
                "installed source '{}' was not found in data state",
                source_id
            )));
        }
    }

    let selected = state
        .installs
        .iter()
        .filter(|record| {
            options
                .target
                .as_ref()
                .is_none_or(|target| record.language == target.as_str())
                && options
                    .source_id
                    .as_deref()
                    .is_none_or(|source_id| source_id == record.source_id)
        })
        .cloned()
        .collect::<Vec<_>>();
    if selected.is_empty() {
        return Err(CliError::config(
            "no installed sources matched the selected filters".to_string(),
        ));
    }

    let mut items = Vec::with_capacity(selected.len());
    let selected_keys = selected
        .iter()
        .map(|record| (record.language.clone(), record.source_id.clone()))
        .collect::<HashSet<_>>();
    let mut removed = 0_usize;
    for record in &selected {
        let source_dir = data_dir.join(&record.language).join(&record.source_id);
        let status = if options.apply {
            if source_dir.exists() {
                fs::remove_dir_all(&source_dir).map_err(|error| {
                    CliError::io(
                        "failed to remove source data directory",
                        Some(&source_dir.to_string_lossy()),
                        error,
                    )
                })?;
                removed += 1;
                "removed"
            } else {
                "removed-state-only"
            }
        } else if source_dir.exists() {
            "would-remove"
        } else {
            "would-remove-missing-dir"
        };
        items.push(DataCleanItem {
            language: record.language.clone(),
            source_id: record.source_id.clone(),
            status,
            path: source_dir.to_string_lossy().to_string(),
        });
    }

    let state_removed = if options.apply {
        let before_len = state.installs.len();
        state.installs.retain(|record| {
            !selected_keys.contains(&(record.language.clone(), record.source_id.clone()))
        });
        let removed_count = before_len.saturating_sub(state.installs.len());
        if removed_count > 0 {
            save_data_state(&data_dir, &state)?;
        }
        removed_count
    } else {
        0
    };

    items.sort_by(|left, right| {
        left.language
            .cmp(&right.language)
            .then(left.source_id.cmp(&right.source_id))
    });

    if matches!(options.format, OutputFormat::Json) {
        let payload = DataCleanResponse {
            mode: "data-clean",
            data_dir: data_dir.to_string_lossy().to_string(),
            apply: options.apply,
            removed,
            state_removed,
            items,
        };
        let output = serde_json::to_string(&payload).map_err(|error| {
            CliError::internal(format!("failed to serialize data clean: {error}"))
        })?;
        println!("{output}");
        return Ok(());
    }

    println!("data clean:");
    println!("data dir: {}", data_dir.to_string_lossy());
    println!(
        "summary: apply:{} removed:{} state_removed:{}",
        if options.apply { "yes" } else { "no" },
        removed,
        state_removed
    );
    for item in items {
        println!(
            "- {}/{} status:{} path:{}",
            item.language, item.source_id, item.status, item.path
        );
    }

    Ok(())
}

fn run_data_pin(options: DataPinOptions) -> Result<(), CliError> {
    let data_dir = resolve_data_dir(options.data_dir.as_deref());
    let state = load_data_state(&data_dir)?;

    if let Some(source_id) = options.source_id.as_deref() {
        let has_source = state.installs.iter().any(|record| {
            record.source_id == source_id
                && options
                    .target
                    .as_ref()
                    .is_none_or(|target| record.language == target.as_str())
        });
        if !has_source {
            return Err(CliError::config(format!(
                "installed source '{}' was not found in data state",
                source_id
            )));
        }
    }

    let selected = state
        .installs
        .iter()
        .filter(|record| {
            options
                .target
                .as_ref()
                .is_none_or(|target| record.language == target.as_str())
                && options
                    .source_id
                    .as_deref()
                    .is_none_or(|source_id| source_id == record.source_id)
        })
        .cloned()
        .collect::<Vec<_>>();
    if selected.is_empty() {
        return Err(CliError::config(
            "no installed sources matched the selected filters".to_string(),
        ));
    }

    let explicit_checksum = options
        .checksum_sha256
        .as_deref()
        .map(normalize_sha256)
        .transpose()?;
    if explicit_checksum.is_some() && selected.len() != 1 {
        return Err(CliError::usage(
            "--checksum can be used only when exactly one installed source is selected".to_string(),
        ));
    }

    let mut items = selected
        .iter()
        .map(|record| build_pin_item(&data_dir, record, explicit_checksum.as_deref()))
        .collect::<Vec<_>>();
    items.sort_by(|left, right| {
        left.language
            .cmp(&right.language)
            .then(left.source_id.cmp(&right.source_id))
    });

    let updated = items.iter().filter(|item| item.status == "pinned").count();
    let failed = items.len().saturating_sub(updated);

    if matches!(options.format, OutputFormat::Json) {
        let payload = DataPinResponse {
            mode: "data-pin",
            data_dir: data_dir.to_string_lossy().to_string(),
            updated,
            failed,
            items,
        };
        let output = serde_json::to_string(&payload).map_err(|error| {
            CliError::internal(format!("failed to serialize data pin: {error}"))
        })?;
        println!("{output}");
    } else {
        println!("data pin:");
        println!("data dir: {}", data_dir.to_string_lossy());
        println!("summary: updated:{} failed:{}", updated, failed);
        for item in &items {
            let artifact_path = item.artifact_path.as_ref().map_or("-", String::as_str);
            let pinned_sha256 = item.pinned_sha256.as_deref().unwrap_or("-");
            println!(
                "- {}/{} status:{} manifest:{} artifact:{} pinned_sha256:{}",
                item.language,
                item.source_id,
                item.status,
                item.manifest_path,
                artifact_path,
                pinned_sha256
            );
            if let Some(reason) = &item.reason {
                println!("  reason: {reason}");
            }
        }
    }

    if failed == 0 {
        Ok(())
    } else {
        Err(CliError::domain(format!(
            "data pin failed for {failed} source(s)"
        )))
    }
}

fn run_data_export_manifest(options: DataExportManifestOptions) -> Result<(), CliError> {
    let sources = load_data_sources()?;
    let data_dir = resolve_data_dir(options.data_dir.as_deref());
    let state = load_data_state(&data_dir)?;

    if let Some(source_id) = options.source_id.as_deref() {
        let has_source = state.installs.iter().any(|record| {
            record.source_id == source_id
                && options
                    .target
                    .as_ref()
                    .is_none_or(|target| record.language == target.as_str())
        });
        if !has_source {
            return Err(CliError::config(format!(
                "installed source '{}' was not found in data state",
                source_id
            )));
        }
    }

    let mut entries = state
        .installs
        .iter()
        .filter(|record| {
            options
                .target
                .as_ref()
                .is_none_or(|target| record.language == target.as_str())
                && options
                    .source_id
                    .as_deref()
                    .is_none_or(|source_id| source_id == record.source_id)
        })
        .map(|record| {
            let source = sources.iter().find(|item| {
                item.id == record.source_id && item.language.as_str() == record.language
            });
            build_export_entry(&data_dir, record, source)
        })
        .collect::<Vec<_>>();
    entries.sort_by(|left, right| {
        left.language
            .cmp(&right.language)
            .then(left.source_id.cmp(&right.source_id))
    });
    if entries.is_empty() {
        return Err(CliError::config(
            "no installed sources matched the selected filters".to_string(),
        ));
    }

    let snapshot = DataExportSnapshot {
        schema_version: 1,
        generated_at_epoch_sec: unix_epoch_seconds()?,
        data_dir: data_dir.to_string_lossy().to_string(),
        entries,
    };
    let raw = serde_json::to_string_pretty(&snapshot).map_err(|error| {
        CliError::internal(format!("failed to encode export manifest: {error}"))
    })?;

    if let Some(output_path) = options.output_path.as_deref() {
        fs::write(output_path, &raw).map_err(|error| {
            CliError::io(
                "failed to write export manifest file",
                Some(output_path),
                error,
            )
        })?;
        if matches!(options.format, OutputFormat::Json) {
            let response = DataExportResponse {
                mode: "data-export-manifest",
                output_path: output_path.to_string(),
                entry_count: snapshot.entries.len(),
            };
            let payload = serde_json::to_string(&response).map_err(|error| {
                CliError::internal(format!(
                    "failed to encode export-manifest response json: {error}"
                ))
            })?;
            println!("{payload}");
            return Ok(());
        }
        println!("data export-manifest:");
        println!("output: {}", output_path);
        println!("entries: {}", snapshot.entries.len());
        return Ok(());
    }

    println!("{raw}");
    Ok(())
}

fn run_data_import_manifest(options: DataImportManifestOptions) -> Result<(), CliError> {
    let data_dir = resolve_data_dir(options.data_dir.as_deref());
    fs::create_dir_all(&data_dir).map_err(|error| {
        CliError::io(
            "failed to create data directory",
            Some(&data_dir.to_string_lossy()),
            error,
        )
    })?;
    let raw = fs::read_to_string(&options.input_path).map_err(|error| {
        CliError::io(
            "failed to read import manifest file",
            Some(&options.input_path),
            error,
        )
    })?;
    let snapshot: DataExportSnapshot = serde_json::from_str(&raw).map_err(|error| {
        CliError::config(format!(
            "failed to parse import manifest '{}': {error}",
            options.input_path
        ))
    })?;
    if snapshot.schema_version != 1 {
        return Err(CliError::config(format!(
            "unsupported import manifest schema_version {} (expected 1)",
            snapshot.schema_version
        )));
    }

    let mut state = load_data_state(&data_dir)?;
    let mut items = Vec::new();
    let mut imported = 0_usize;
    let mut updated = 0_usize;
    for entry in snapshot.entries {
        if options
            .target
            .as_ref()
            .is_some_and(|target| entry.language != target.as_str())
        {
            continue;
        }
        if options
            .source_id
            .as_deref()
            .is_some_and(|source_id| source_id != entry.source_id)
        {
            continue;
        }

        let target = parse_data_language(&entry.language)?;
        let source_dir = data_dir.join(target.as_str()).join(&entry.source_id);
        fs::create_dir_all(&source_dir).map_err(|error| {
            CliError::io(
                "failed to create source data directory",
                Some(&source_dir.to_string_lossy()),
                error,
            )
        })?;

        let manifest = InstalledSourceManifest {
            schema_version: 1,
            language: entry.language.clone(),
            source_id: entry.source_id.clone(),
            source_url: entry
                .source_url
                .unwrap_or_else(|| format!("imported://{}", entry.source_id)),
            version: entry.version.clone(),
            license: entry.license.unwrap_or_else(|| "unknown".to_string()),
            checksum_sha256: entry.checksum_sha256.clone(),
            artifact_url: None,
            artifact_path: entry.artifact_path.clone(),
            artifact_sha256: entry.artifact_sha256.clone(),
            artifact_bytes: None,
            artifact_dataset_kind: None,
            artifact_dataset_schema_version: None,
            artifact_dataset_language: None,
            artifact_dataset_entry_count: None,
            installed_at_epoch_sec: entry.installed_at_epoch_sec,
        };
        let manifest_path = source_dir.join("manifest.json");
        let manifest_raw = serde_json::to_string_pretty(&manifest).map_err(|error| {
            CliError::internal(format!(
                "failed to encode imported source manifest '{}': {error}",
                entry.source_id
            ))
        })?;
        fs::write(&manifest_path, manifest_raw).map_err(|error| {
            CliError::io(
                "failed to write imported source manifest",
                Some(&manifest_path.to_string_lossy()),
                error,
            )
        })?;

        let status = upsert_install_record(
            &mut state,
            &entry.language,
            &entry.source_id,
            &entry.version,
            entry.installed_at_epoch_sec,
        );
        if status == "imported" {
            imported += 1;
        } else {
            updated += 1;
        }
        items.push(DataImportItem {
            language: entry.language,
            source_id: entry.source_id,
            status,
            manifest_path: manifest_path.to_string_lossy().to_string(),
        });
    }

    if items.is_empty() {
        return Err(CliError::config(
            "no import-manifest entries matched the selected filters".to_string(),
        ));
    }

    save_data_state(&data_dir, &state)?;

    if matches!(options.format, OutputFormat::Json) {
        let payload = DataImportResponse {
            mode: "data-import-manifest",
            data_dir: data_dir.to_string_lossy().to_string(),
            imported,
            updated,
            items,
        };
        let output = serde_json::to_string(&payload).map_err(|error| {
            CliError::internal(format!(
                "failed to serialize data import-manifest response: {error}"
            ))
        })?;
        println!("{output}");
        return Ok(());
    }

    println!("data import-manifest:");
    println!("data dir: {}", data_dir.to_string_lossy());
    println!("imported: {imported} updated: {updated}");
    for item in items {
        println!(
            "- {}/{} status:{} manifest:{}",
            item.language, item.source_id, item.status, item.manifest_path
        );
    }
    Ok(())
}

#[derive(Debug, Clone)]
struct ManifestCandidate {
    language: String,
    source_id: String,
    manifest_path: PathBuf,
}

fn collect_manifest_candidates(
    data_dir: &Path,
    target: Option<ProtoTarget>,
    source_filter: Option<&str>,
) -> Result<Vec<ManifestCandidate>, CliError> {
    let mut items = Vec::new();
    let language_codes = match target {
        Some(value) => vec![value.as_str().to_string()],
        None => discover_data_languages(data_dir)?,
    };
    for language in language_codes {
        let language_dir = data_dir.join(&language);
        if !language_dir.exists() {
            continue;
        }
        let entries = fs::read_dir(&language_dir).map_err(|error| {
            CliError::io(
                "failed to read language data directory",
                Some(&language_dir.to_string_lossy()),
                error,
            )
        })?;
        for entry in entries {
            let entry = entry.map_err(|error| {
                CliError::io(
                    "failed to read source directory entry",
                    Some(&language_dir.to_string_lossy()),
                    error,
                )
            })?;
            let source_path = entry.path();
            if !source_path.is_dir() {
                continue;
            }
            let source_id = entry.file_name().to_string_lossy().to_string();
            if source_filter.is_some_and(|value| value != source_id.as_str()) {
                continue;
            }
            let manifest_path = source_path.join("manifest.json");
            if !manifest_path.exists() {
                continue;
            }
            items.push(ManifestCandidate {
                language: language.clone(),
                source_id,
                manifest_path,
            });
        }
    }
    Ok(items)
}

fn discover_data_languages(data_dir: &Path) -> Result<Vec<String>, CliError> {
    if !data_dir.exists() {
        return Ok(Vec::new());
    }

    let entries = fs::read_dir(data_dir).map_err(|error| {
        CliError::io(
            "failed to read data directory",
            Some(&data_dir.to_string_lossy()),
            error,
        )
    })?;

    let mut languages = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|error| {
            CliError::io(
                "failed to read language directory entry",
                Some(&data_dir.to_string_lossy()),
                error,
            )
        })?;
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }

        let language = entry.file_name().to_string_lossy().trim().to_string();
        if language.is_empty() {
            continue;
        }
        languages.push(language);
    }

    languages.sort();
    languages.dedup();
    Ok(languages)
}

fn read_manifest_for_doctor(path: &Path) -> Result<InstalledSourceManifest, String> {
    let raw = fs::read_to_string(path).map_err(|error| {
        format!(
            "failed to read manifest '{}': {error}",
            path.to_string_lossy()
        )
    })?;
    serde_json::from_str(&raw).map_err(|error| {
        format!(
            "failed to parse manifest '{}': {error}",
            path.to_string_lossy()
        )
    })
}

fn load_data_sources() -> Result<Vec<DataSource>, CliError> {
    let raw: DataSourceCatalogRaw =
        serde_json::from_str(DATA_SOURCES_MANIFEST).map_err(|error| {
            CliError::config(format!("invalid data sources manifest json: {error}"))
        })?;
    if raw.schema_version != MANIFEST_SCHEMA_VERSION {
        return Err(CliError::config(format!(
            "unsupported data sources manifest schema_version {} (expected {})",
            raw.schema_version, MANIFEST_SCHEMA_VERSION
        )));
    }
    if raw.sources.is_empty() {
        return Err(CliError::config(
            "data sources manifest does not contain any source entries".to_string(),
        ));
    }

    let mut seen_ids = HashSet::new();
    let mut source_count_by_language: HashMap<String, usize> = HashMap::new();
    let mut recommended_count_by_language: HashMap<String, usize> = HashMap::new();
    let mut sources = Vec::with_capacity(raw.sources.len());

    for entry in raw.sources {
        if !seen_ids.insert(entry.id.clone()) {
            return Err(CliError::config(format!(
                "data sources manifest has duplicate source id '{}'",
                entry.id
            )));
        }
        let language = parse_data_language(&entry.language)?;
        *source_count_by_language
            .entry(entry.language.clone())
            .or_insert(0) += 1;
        if entry.recommended {
            *recommended_count_by_language
                .entry(entry.language.clone())
                .or_insert(0) += 1;
        }
        sources.push(DataSource {
            id: entry.id,
            language,
            source_url: entry.source_url,
            license: entry.license,
            version: entry.version,
            checksum_sha256: entry.checksum_sha256,
            recommended: entry.recommended,
        });
    }

    for (language_code, source_count) in source_count_by_language {
        if source_count == 0 {
            continue;
        }
        let recommended_count = recommended_count_by_language
            .get(&language_code)
            .copied()
            .unwrap_or(0);
        if recommended_count != 1 {
            return Err(CliError::config(format!(
                "data sources manifest must contain exactly one recommended source for language '{}' (found {})",
                language_code, recommended_count
            )));
        }
    }

    Ok(sources)
}

fn build_status_item(
    data_dir: &Path,
    record: &InstallRecord,
    source: Option<&DataSource>,
) -> DataStatusItem {
    let manifest_path = data_dir
        .join(&record.language)
        .join(&record.source_id)
        .join("manifest.json");
    if !manifest_path.exists() {
        return DataStatusItem {
            language: record.language.clone(),
            source_id: record.source_id.clone(),
            version: record.version.clone(),
            installed_at_epoch_sec: record.installed_at_epoch_sec,
            source_url: source.map(|item| item.source_url.clone()),
            license: source.map(|item| item.license.clone()),
            manifest_path: manifest_path.to_string_lossy().to_string(),
            manifest_exists: false,
            status: "missing-manifest",
            artifact_path: None,
            artifact_exists: None,
            artifact_sha256: None,
            manifest_error: None,
        };
    }

    let raw = match fs::read_to_string(&manifest_path) {
        Ok(value) => value,
        Err(error) => {
            return DataStatusItem {
                language: record.language.clone(),
                source_id: record.source_id.clone(),
                version: record.version.clone(),
                installed_at_epoch_sec: record.installed_at_epoch_sec,
                source_url: source.map(|item| item.source_url.clone()),
                license: source.map(|item| item.license.clone()),
                manifest_path: manifest_path.to_string_lossy().to_string(),
                manifest_exists: true,
                status: "invalid-manifest",
                artifact_path: None,
                artifact_exists: None,
                artifact_sha256: None,
                manifest_error: Some(error.to_string()),
            };
        }
    };

    let manifest: InstalledSourceManifest = match serde_json::from_str(&raw) {
        Ok(value) => value,
        Err(error) => {
            return DataStatusItem {
                language: record.language.clone(),
                source_id: record.source_id.clone(),
                version: record.version.clone(),
                installed_at_epoch_sec: record.installed_at_epoch_sec,
                source_url: source.map(|item| item.source_url.clone()),
                license: source.map(|item| item.license.clone()),
                manifest_path: manifest_path.to_string_lossy().to_string(),
                manifest_exists: true,
                status: "invalid-manifest",
                artifact_path: None,
                artifact_exists: None,
                artifact_sha256: None,
                manifest_error: Some(error.to_string()),
            };
        }
    };

    let artifact_exists = manifest
        .artifact_path
        .as_ref()
        .map(|path| Path::new(path).exists());
    let status = if manifest
        .artifact_path
        .as_ref()
        .is_some_and(|_| artifact_exists == Some(false))
    {
        "missing-artifact"
    } else {
        "ok"
    };

    DataStatusItem {
        language: record.language.clone(),
        source_id: record.source_id.clone(),
        version: record.version.clone(),
        installed_at_epoch_sec: record.installed_at_epoch_sec,
        source_url: source.map(|item| item.source_url.clone()),
        license: source.map(|item| item.license.clone()),
        manifest_path: manifest_path.to_string_lossy().to_string(),
        manifest_exists: true,
        status,
        artifact_path: manifest.artifact_path,
        artifact_exists,
        artifact_sha256: manifest.artifact_sha256,
        manifest_error: None,
    }
}

fn build_export_entry(
    data_dir: &Path,
    record: &InstallRecord,
    source: Option<&DataSource>,
) -> DataExportEntry {
    let manifest_path = data_dir
        .join(&record.language)
        .join(&record.source_id)
        .join("manifest.json");
    if !manifest_path.exists() {
        return DataExportEntry {
            language: record.language.clone(),
            source_id: record.source_id.clone(),
            version: record.version.clone(),
            installed_at_epoch_sec: record.installed_at_epoch_sec,
            source_url: source.map(|item| item.source_url.clone()),
            license: source.map(|item| item.license.clone()),
            checksum_sha256: source.and_then(|item| item.checksum_sha256.clone()),
            manifest_path: manifest_path.to_string_lossy().to_string(),
            manifest_exists: false,
            artifact_path: None,
            artifact_sha256: None,
            manifest_error: None,
        };
    }

    let raw = match fs::read_to_string(&manifest_path) {
        Ok(value) => value,
        Err(error) => {
            return DataExportEntry {
                language: record.language.clone(),
                source_id: record.source_id.clone(),
                version: record.version.clone(),
                installed_at_epoch_sec: record.installed_at_epoch_sec,
                source_url: source.map(|item| item.source_url.clone()),
                license: source.map(|item| item.license.clone()),
                checksum_sha256: source.and_then(|item| item.checksum_sha256.clone()),
                manifest_path: manifest_path.to_string_lossy().to_string(),
                manifest_exists: true,
                artifact_path: None,
                artifact_sha256: None,
                manifest_error: Some(error.to_string()),
            };
        }
    };

    let manifest: InstalledSourceManifest = match serde_json::from_str(&raw) {
        Ok(value) => value,
        Err(error) => {
            return DataExportEntry {
                language: record.language.clone(),
                source_id: record.source_id.clone(),
                version: record.version.clone(),
                installed_at_epoch_sec: record.installed_at_epoch_sec,
                source_url: source.map(|item| item.source_url.clone()),
                license: source.map(|item| item.license.clone()),
                checksum_sha256: source.and_then(|item| item.checksum_sha256.clone()),
                manifest_path: manifest_path.to_string_lossy().to_string(),
                manifest_exists: true,
                artifact_path: None,
                artifact_sha256: None,
                manifest_error: Some(error.to_string()),
            };
        }
    };

    DataExportEntry {
        language: record.language.clone(),
        source_id: record.source_id.clone(),
        version: record.version.clone(),
        installed_at_epoch_sec: record.installed_at_epoch_sec,
        source_url: Some(manifest.source_url),
        license: Some(manifest.license),
        checksum_sha256: manifest.checksum_sha256,
        manifest_path: manifest_path.to_string_lossy().to_string(),
        manifest_exists: true,
        artifact_path: manifest.artifact_path,
        artifact_sha256: manifest.artifact_sha256,
        manifest_error: None,
    }
}

fn build_verify_item(
    data_dir: &Path,
    record: &InstallRecord,
    source: Option<&DataSource>,
) -> DataVerifyItem {
    let source_dir = data_dir.join(&record.language).join(&record.source_id);
    let manifest_path = source_dir.join("manifest.json");
    if !manifest_path.exists() {
        return DataVerifyItem {
            language: record.language.clone(),
            source_id: record.source_id.clone(),
            status: "missing-manifest",
            verifiable: false,
            manifest_path: manifest_path.to_string_lossy().to_string(),
            artifact_path: None,
            expected_sha256: None,
            actual_sha256: None,
            reason: None,
        };
    }

    let raw = match fs::read_to_string(&manifest_path) {
        Ok(value) => value,
        Err(error) => {
            return DataVerifyItem {
                language: record.language.clone(),
                source_id: record.source_id.clone(),
                status: "invalid-manifest",
                verifiable: false,
                manifest_path: manifest_path.to_string_lossy().to_string(),
                artifact_path: None,
                expected_sha256: None,
                actual_sha256: None,
                reason: Some(error.to_string()),
            };
        }
    };

    let manifest: InstalledSourceManifest = match serde_json::from_str(&raw) {
        Ok(value) => value,
        Err(error) => {
            return DataVerifyItem {
                language: record.language.clone(),
                source_id: record.source_id.clone(),
                status: "invalid-manifest",
                verifiable: false,
                manifest_path: manifest_path.to_string_lossy().to_string(),
                artifact_path: None,
                expected_sha256: None,
                actual_sha256: None,
                reason: Some(error.to_string()),
            };
        }
    };

    let Some(artifact_raw_path) = manifest.artifact_path else {
        return DataVerifyItem {
            language: record.language.clone(),
            source_id: record.source_id.clone(),
            status: "skipped-no-artifact",
            verifiable: false,
            manifest_path: manifest_path.to_string_lossy().to_string(),
            artifact_path: None,
            expected_sha256: source.and_then(|item| item.checksum_sha256.clone()),
            actual_sha256: None,
            reason: None,
        };
    };

    let artifact_path = if Path::new(&artifact_raw_path).is_absolute() {
        PathBuf::from(&artifact_raw_path)
    } else {
        source_dir.join(&artifact_raw_path)
    };
    if !artifact_path.exists() {
        return DataVerifyItem {
            language: record.language.clone(),
            source_id: record.source_id.clone(),
            status: "missing-artifact",
            verifiable: true,
            manifest_path: manifest_path.to_string_lossy().to_string(),
            artifact_path: Some(artifact_path.to_string_lossy().to_string()),
            expected_sha256: source
                .and_then(|item| item.checksum_sha256.clone())
                .or(manifest.artifact_sha256),
            actual_sha256: None,
            reason: None,
        };
    }

    let actual_sha256 = match sha256_file_limited(&artifact_path) {
        Ok(value) => value,
        Err(reason) => {
            return DataVerifyItem {
                language: record.language.clone(),
                source_id: record.source_id.clone(),
                status: "artifact-read-error",
                verifiable: true,
                manifest_path: manifest_path.to_string_lossy().to_string(),
                artifact_path: Some(artifact_path.to_string_lossy().to_string()),
                expected_sha256: source
                    .and_then(|item| item.checksum_sha256.clone())
                    .or(manifest.artifact_sha256),
                actual_sha256: None,
                reason: Some(reason),
            };
        }
    };

    if let Some(manifest_sha) = manifest.artifact_sha256.as_deref() {
        if manifest_sha != actual_sha256 {
            return DataVerifyItem {
                language: record.language.clone(),
                source_id: record.source_id.clone(),
                status: "checksum-mismatch",
                verifiable: true,
                manifest_path: manifest_path.to_string_lossy().to_string(),
                artifact_path: Some(artifact_path.to_string_lossy().to_string()),
                expected_sha256: Some(manifest_sha.to_string()),
                actual_sha256: Some(actual_sha256),
                reason: Some("manifest artifact_sha256 mismatch".to_string()),
            };
        }
    }

    if let Some(source_sha) = source.and_then(|item| item.checksum_sha256.as_deref()) {
        if source_sha != actual_sha256 {
            return DataVerifyItem {
                language: record.language.clone(),
                source_id: record.source_id.clone(),
                status: "checksum-mismatch",
                verifiable: true,
                manifest_path: manifest_path.to_string_lossy().to_string(),
                artifact_path: Some(artifact_path.to_string_lossy().to_string()),
                expected_sha256: Some(source_sha.to_string()),
                actual_sha256: Some(actual_sha256),
                reason: Some("catalog checksum_sha256 mismatch".to_string()),
            };
        }
    }

    let expected_sha256 = source
        .and_then(|item| item.checksum_sha256.clone())
        .or(manifest.artifact_sha256);
    if expected_sha256.is_none() {
        return DataVerifyItem {
            language: record.language.clone(),
            source_id: record.source_id.clone(),
            status: "skipped-no-checksum",
            verifiable: false,
            manifest_path: manifest_path.to_string_lossy().to_string(),
            artifact_path: Some(artifact_path.to_string_lossy().to_string()),
            expected_sha256: None,
            actual_sha256: Some(actual_sha256),
            reason: None,
        };
    }

    DataVerifyItem {
        language: record.language.clone(),
        source_id: record.source_id.clone(),
        status: "ok",
        verifiable: true,
        manifest_path: manifest_path.to_string_lossy().to_string(),
        artifact_path: Some(artifact_path.to_string_lossy().to_string()),
        expected_sha256,
        actual_sha256: Some(actual_sha256),
        reason: None,
    }
}

fn build_pin_item(
    data_dir: &Path,
    record: &InstallRecord,
    explicit_checksum: Option<&str>,
) -> DataPinItem {
    let source_dir = data_dir.join(&record.language).join(&record.source_id);
    let manifest_path = source_dir.join("manifest.json");
    if !manifest_path.exists() {
        return DataPinItem {
            language: record.language.clone(),
            source_id: record.source_id.clone(),
            status: "missing-manifest",
            manifest_path: manifest_path.to_string_lossy().to_string(),
            artifact_path: None,
            pinned_sha256: None,
            reason: None,
        };
    }

    let raw = match fs::read_to_string(&manifest_path) {
        Ok(value) => value,
        Err(error) => {
            return DataPinItem {
                language: record.language.clone(),
                source_id: record.source_id.clone(),
                status: "invalid-manifest",
                manifest_path: manifest_path.to_string_lossy().to_string(),
                artifact_path: None,
                pinned_sha256: None,
                reason: Some(error.to_string()),
            };
        }
    };

    let mut manifest: InstalledSourceManifest = match serde_json::from_str(&raw) {
        Ok(value) => value,
        Err(error) => {
            return DataPinItem {
                language: record.language.clone(),
                source_id: record.source_id.clone(),
                status: "invalid-manifest",
                manifest_path: manifest_path.to_string_lossy().to_string(),
                artifact_path: None,
                pinned_sha256: None,
                reason: Some(error.to_string()),
            };
        }
    };

    let checksum = match explicit_checksum {
        Some(value) => value.to_string(),
        None => {
            let Some(artifact_raw_path) = manifest.artifact_path.as_deref() else {
                return DataPinItem {
                    language: record.language.clone(),
                    source_id: record.source_id.clone(),
                    status: "missing-artifact",
                    manifest_path: manifest_path.to_string_lossy().to_string(),
                    artifact_path: None,
                    pinned_sha256: None,
                    reason: Some(
                        "artifact_path is not set in manifest; use --checksum for explicit pin"
                            .to_string(),
                    ),
                };
            };
            let artifact_path = if Path::new(artifact_raw_path).is_absolute() {
                PathBuf::from(artifact_raw_path)
            } else {
                source_dir.join(artifact_raw_path)
            };
            if !artifact_path.exists() {
                return DataPinItem {
                    language: record.language.clone(),
                    source_id: record.source_id.clone(),
                    status: "missing-artifact",
                    manifest_path: manifest_path.to_string_lossy().to_string(),
                    artifact_path: Some(artifact_path.to_string_lossy().to_string()),
                    pinned_sha256: None,
                    reason: None,
                };
            }
            match sha256_file_limited(&artifact_path) {
                Ok(value) => value,
                Err(reason) => {
                    return DataPinItem {
                        language: record.language.clone(),
                        source_id: record.source_id.clone(),
                        status: "artifact-read-error",
                        manifest_path: manifest_path.to_string_lossy().to_string(),
                        artifact_path: Some(artifact_path.to_string_lossy().to_string()),
                        pinned_sha256: None,
                        reason: Some(reason),
                    };
                }
            }
        }
    };

    manifest.artifact_sha256 = Some(checksum.clone());
    let raw = match serde_json::to_string_pretty(&manifest) {
        Ok(value) => value,
        Err(error) => {
            return DataPinItem {
                language: record.language.clone(),
                source_id: record.source_id.clone(),
                status: "write-error",
                manifest_path: manifest_path.to_string_lossy().to_string(),
                artifact_path: manifest.artifact_path.clone(),
                pinned_sha256: Some(checksum),
                reason: Some(error.to_string()),
            };
        }
    };
    if let Err(error) = fs::write(&manifest_path, raw) {
        return DataPinItem {
            language: record.language.clone(),
            source_id: record.source_id.clone(),
            status: "write-error",
            manifest_path: manifest_path.to_string_lossy().to_string(),
            artifact_path: manifest.artifact_path.clone(),
            pinned_sha256: Some(checksum),
            reason: Some(error.to_string()),
        };
    }

    DataPinItem {
        language: record.language.clone(),
        source_id: record.source_id.clone(),
        status: "pinned",
        manifest_path: manifest_path.to_string_lossy().to_string(),
        artifact_path: manifest.artifact_path,
        pinned_sha256: Some(checksum),
        reason: None,
    }
}

fn normalize_sha256(value: &str) -> Result<String, CliError> {
    let normalized = value.trim().to_ascii_lowercase();
    if normalized.len() != 64 || !normalized.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(CliError::usage(
            "--checksum must be a 64-character sha256 hex string".to_string(),
        ));
    }
    Ok(normalized)
}

fn sha256_file_limited(path: &Path) -> Result<String, String> {
    let metadata = fs::metadata(path).map_err(|error| {
        format!(
            "failed to stat artifact file '{}': {error}",
            path.to_string_lossy()
        )
    })?;
    let file_len = usize::try_from(metadata.len()).map_err(|_| {
        format!(
            "artifact file is too large to verify: {}",
            path.to_string_lossy()
        )
    })?;
    if file_len > MAX_ARTIFACT_BYTES {
        return Err(format!(
            "artifact exceeds max allowed size ({} bytes): {}",
            MAX_ARTIFACT_BYTES,
            path.to_string_lossy()
        ));
    }
    let bytes = fs::read(path).map_err(|error| {
        format!(
            "failed to read artifact file '{}': {error}",
            path.to_string_lossy()
        )
    })?;
    Ok(sha256_hex(&bytes))
}

fn parse_data_language(value: &str) -> Result<ProtoTarget, CliError> {
    let normalized = value.trim().to_ascii_lowercase();
    if normalized.is_empty() {
        return Err(CliError::config(
            "data sources manifest contains an empty language code".to_string(),
        ));
    }
    if !normalized
        .bytes()
        .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
    {
        return Err(CliError::config(format!(
            "invalid language code '{value}' in data sources manifest"
        )));
    }
    Ok(ProtoTarget::from_language_code(&normalized))
}

fn resolve_source_for_target<'a>(
    sources: &'a [DataSource],
    target: ProtoTarget,
    source_id: Option<&str>,
) -> Result<&'a DataSource, CliError> {
    if let Some(source_id) = source_id {
        let source = sources
            .iter()
            .find(|item| item.id == source_id)
            .ok_or_else(|| {
                CliError::config(format!(
                    "unknown data source id '{source_id}' (use 'lsteg data list')"
                ))
            })?;
        if source.language != target {
            return Err(CliError::config(format!(
                "source '{}' is not available for language '{}'",
                source_id,
                target.as_str()
            )));
        }
        return Ok(source);
    }

    sources
        .iter()
        .find(|item| item.language == target && item.recommended)
        .ok_or_else(|| {
            CliError::config(format!(
                "no recommended data source is registered for language '{}'",
                target.as_str()
            ))
        })
}

fn resolve_data_dir(explicit: Option<&str>) -> PathBuf {
    if let Some(path) = explicit {
        return PathBuf::from(path);
    }
    if let Ok(path) = std::env::var("LSTEG_DATA_DIR") {
        let trimmed = path.trim();
        if !trimmed.is_empty() {
            return PathBuf::from(trimmed);
        }
    }
    if let Ok(home) = std::env::var("HOME") {
        return PathBuf::from(home)
            .join(".cache")
            .join("linguasteg")
            .join("data");
    }
    PathBuf::from(".linguasteg-data")
}

fn starter_dataset_template_for_language(language: &ProtoTarget) -> &'static str {
    match language.as_str() {
        "en" => {
            r#"{
  "kind": "linguasteg-lexicon-v1",
  "schema_version": 1,
  "language": "en",
  "entries": [
    {
      "slot": "object",
      "canonical": "letter",
      "variants": ["missive", "epistle"]
    },
    {
      "slot": "adjective",
      "canonical": "quiet",
      "variants": ["concise"]
    },
    {
      "slot": "verb",
      "canonical": "writes",
      "variants": ["composes"]
    }
  ]
}
"#
        }
        "fa" => {
            r#"{
  "kind": "linguasteg-lexicon-v1",
  "schema_version": 1,
  "language": "fa",
  "entries": [
    {
      "slot": "object",
      "canonical": "نامه",
      "variants": ["مکتوب"]
    },
    {
      "slot": "adjective",
      "canonical": "زیبا",
      "variants": ["خوش"]
    },
    {
      "slot": "verb",
      "canonical": "نوشت",
      "variants": ["نگاشت"]
    }
  ]
}
"#
        }
        "de" => {
            r#"{
  "kind": "linguasteg-lexicon-v1",
  "schema_version": 1,
  "language": "de",
  "entries": [
    {
      "slot": "object",
      "canonical": "brief",
      "variants": ["schreiben", "botschaft"]
    },
    {
      "slot": "adjective",
      "canonical": "neu",
      "variants": ["frisch"]
    },
    {
      "slot": "verb",
      "canonical": "schreibt",
      "variants": ["verfasst"]
    }
  ]
}
"#
        }
        _ => {
            r#"{
  "kind": "linguasteg-lexicon-v1",
  "schema_version": 1,
  "language": "xx",
  "entries": [
    {
      "slot": "object",
      "canonical": "placeholder",
      "variants": ["placeholder-alt"]
    }
  ]
}
"#
        }
    }
}

fn ensure_starter_dataset_template_exists(
    data_dir: &Path,
    source: &DataSource,
) -> Result<PathBuf, CliError> {
    let source_dir = data_dir.join(source.language.as_str()).join(&source.id);
    fs::create_dir_all(&source_dir).map_err(|error| {
        CliError::io(
            "failed to create source data directory",
            Some(&source_dir.to_string_lossy()),
            error,
        )
    })?;

    let starter_path = source_dir.join(STARTER_DATASET_FILE);
    if starter_path.exists() {
        return Ok(starter_path);
    }

    let template_raw = starter_dataset_template_for_language(&source.language)
        .replace("\"xx\"", &format!("\"{}\"", source.language.as_str()));
    fs::write(&starter_path, template_raw).map_err(|error| {
        CliError::io(
            "failed to write starter dataset template",
            Some(&starter_path.to_string_lossy()),
            error,
        )
    })?;
    Ok(starter_path)
}

fn load_local_dataset_artifact(
    source: &DataSource,
    dataset_path: &Path,
) -> Result<Option<StoredArtifact>, CliError> {
    if !dataset_path.exists() {
        return Ok(None);
    }
    let bytes = read_local_file_limited(dataset_path)?;
    if bytes.is_empty() {
        return Ok(None);
    }

    let metadata = load_lexicon_dataset_artifact(source.language.as_str(), &bytes)
        .map_err(|reason| {
            CliError::config(format!(
                "starter dataset validation failed for source '{}': {reason}",
                source.id
            ))
        })?
        .ok_or_else(|| {
            CliError::config(format!(
                "starter dataset for source '{}' is not a linguasteg lexicon dataset",
                source.id
            ))
        })?
        .metadata();

    Ok(Some(StoredArtifact {
        path: dataset_path.to_path_buf(),
        sha256: sha256_hex(&bytes),
        byte_len: bytes.len(),
        dataset_metadata: Some(metadata),
    }))
}

fn unix_epoch_seconds() -> Result<u64, CliError> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .map_err(|error| CliError::internal(format!("system clock error: {error}")))
}

fn data_state_path(data_dir: &Path) -> PathBuf {
    data_dir.join(DATA_STATE_FILE)
}

fn load_data_state(data_dir: &Path) -> Result<DataState, CliError> {
    let path = data_state_path(data_dir);
    if !path.exists() {
        return Ok(DataState {
            schema_version: 1,
            installs: Vec::new(),
        });
    }
    let raw = fs::read_to_string(&path).map_err(|error| {
        CliError::io(
            "failed to read data state file",
            Some(&path.to_string_lossy()),
            error,
        )
    })?;
    let mut state: DataState = serde_json::from_str(&raw).map_err(|error| {
        CliError::config(format!(
            "failed to parse data state file '{}': {error}",
            path.to_string_lossy()
        ))
    })?;
    if state.schema_version == 0 {
        state.schema_version = 1;
    }
    Ok(state)
}

fn save_data_state(data_dir: &Path, state: &DataState) -> Result<(), CliError> {
    let path = data_state_path(data_dir);
    let raw = serde_json::to_string_pretty(state)
        .map_err(|error| CliError::internal(format!("failed to encode data state: {error}")))?;
    fs::write(&path, raw).map_err(|error| {
        CliError::io(
            "failed to write data state file",
            Some(&path.to_string_lossy()),
            error,
        )
    })
}

fn upsert_install_state<'a>(
    state: &'a mut DataState,
    source: &DataSource,
    now: u64,
    force_refresh: bool,
) -> &'a str {
    let existing = state
        .installs
        .iter_mut()
        .find(|item| item.language == source.language.as_str() && item.source_id == source.id);

    match existing {
        Some(record) => {
            if force_refresh {
                record.version = source.version.clone();
                record.installed_at_epoch_sec = now;
                "updated"
            } else {
                "up-to-date"
            }
        }
        None => {
            state.installs.push(InstallRecord {
                language: source.language.as_str().to_string(),
                source_id: source.id.clone(),
                version: source.version.clone(),
                installed_at_epoch_sec: now,
            });
            "installed"
        }
    }
}

fn upsert_install_record(
    state: &mut DataState,
    language: &str,
    source_id: &str,
    version: &str,
    installed_at_epoch_sec: u64,
) -> &'static str {
    let existing = state
        .installs
        .iter_mut()
        .find(|item| item.language == language && item.source_id == source_id);
    match existing {
        Some(record) => {
            record.version = version.to_string();
            record.installed_at_epoch_sec = installed_at_epoch_sec;
            "updated"
        }
        None => {
            state.installs.push(InstallRecord {
                language: language.to_string(),
                source_id: source_id.to_string(),
                version: version.to_string(),
                installed_at_epoch_sec,
            });
            "imported"
        }
    }
}

fn write_install_manifest(
    data_dir: &Path,
    source: &DataSource,
    installed_at_epoch_sec: u64,
    artifact_url: Option<&str>,
    artifact: Option<&StoredArtifact>,
) -> Result<PathBuf, CliError> {
    let language_dir = data_dir.join(source.language.as_str());
    let source_dir = language_dir.join(&source.id);
    fs::create_dir_all(&source_dir).map_err(|error| {
        CliError::io(
            "failed to create source data directory",
            Some(&source_dir.to_string_lossy()),
            error,
        )
    })?;

    let manifest = InstalledSourceManifest {
        schema_version: 1,
        language: source.language.as_str().to_string(),
        source_id: source.id.clone(),
        source_url: source.source_url.clone(),
        version: source.version.clone(),
        license: source.license.clone(),
        checksum_sha256: source.checksum_sha256.clone(),
        artifact_url: artifact_url.map(ToString::to_string),
        artifact_path: artifact
            .as_ref()
            .map(|item| item.path.to_string_lossy().to_string()),
        artifact_sha256: artifact.as_ref().map(|item| item.sha256.clone()),
        artifact_bytes: artifact.as_ref().map(|item| item.byte_len),
        artifact_dataset_kind: artifact
            .and_then(|item| item.dataset_metadata.as_ref())
            .map(|item| item.kind.clone()),
        artifact_dataset_schema_version: artifact
            .and_then(|item| item.dataset_metadata.as_ref())
            .map(|item| item.schema_version),
        artifact_dataset_language: artifact
            .and_then(|item| item.dataset_metadata.as_ref())
            .map(|item| item.language.clone()),
        artifact_dataset_entry_count: artifact
            .and_then(|item| item.dataset_metadata.as_ref())
            .map(|item| item.entry_count),
        installed_at_epoch_sec,
    };
    let manifest_path = source_dir.join("manifest.json");
    let raw = serde_json::to_string_pretty(&manifest).map_err(|error| {
        CliError::internal(format!("failed to encode source manifest: {error}"))
    })?;
    fs::write(&manifest_path, raw).map_err(|error| {
        CliError::io(
            "failed to write source manifest",
            Some(&manifest_path.to_string_lossy()),
            error,
        )
    })?;
    Ok(manifest_path)
}

#[derive(Debug, Clone)]
struct StoredArtifact {
    path: PathBuf,
    sha256: String,
    byte_len: usize,
    dataset_metadata: Option<DatasetArtifactMetadata>,
}

fn fetch_and_store_artifact(
    data_dir: &Path,
    source: &DataSource,
    artifact_url: &str,
) -> Result<StoredArtifact, CliError> {
    let bytes = read_artifact_bytes(artifact_url)?;
    let dataset_metadata = load_lexicon_dataset_artifact(source.language.as_str(), &bytes)
        .map_err(|reason| {
            CliError::config(format!(
                "artifact validation failed for source '{}': {reason}",
                source.id
            ))
        })?
        .map(|dataset| dataset.metadata());
    let sha256 = sha256_hex(&bytes);
    let source_dir = data_dir.join(source.language.as_str()).join(&source.id);
    fs::create_dir_all(&source_dir).map_err(|error| {
        CliError::io(
            "failed to create source data directory",
            Some(&source_dir.to_string_lossy()),
            error,
        )
    })?;
    let artifact_path = source_dir.join("artifact.bin");
    fs::write(&artifact_path, &bytes).map_err(|error| {
        CliError::io(
            "failed to write source artifact",
            Some(&artifact_path.to_string_lossy()),
            error,
        )
    })?;
    Ok(StoredArtifact {
        path: artifact_path,
        sha256,
        byte_len: bytes.len(),
        dataset_metadata,
    })
}

fn read_artifact_bytes(url: &str) -> Result<Vec<u8>, CliError> {
    if let Some(path) = url.strip_prefix("file://") {
        return read_local_file_limited(Path::new(path));
    }
    if let Some(path) = url.strip_prefix("path://") {
        return read_local_file_limited(Path::new(path));
    }
    if url.starts_with('/') {
        return read_local_file_limited(Path::new(url));
    }
    if url.starts_with("http://") || url.starts_with("https://") {
        return read_http_bytes(url);
    }
    Err(CliError::input(format!(
        "unsupported artifact url '{url}' (supported: file://, path://, absolute path, http://, https://)"
    )))
}

fn read_local_file_limited(path: &Path) -> Result<Vec<u8>, CliError> {
    let metadata = fs::metadata(path).map_err(|error| {
        CliError::io(
            "failed to stat artifact file",
            Some(&path.to_string_lossy()),
            error,
        )
    })?;
    let file_len = usize::try_from(metadata.len()).map_err(|_| {
        CliError::input(format!(
            "artifact file is too large to load: {}",
            path.to_string_lossy()
        ))
    })?;
    if file_len > MAX_ARTIFACT_BYTES {
        return Err(CliError::input(format!(
            "artifact exceeds max allowed size ({} bytes): {}",
            MAX_ARTIFACT_BYTES,
            path.to_string_lossy()
        )));
    }
    fs::read(path).map_err(|error| {
        CliError::io(
            "failed to read artifact file",
            Some(&path.to_string_lossy()),
            error,
        )
    })
}

fn read_http_bytes(url: &str) -> Result<Vec<u8>, CliError> {
    let response = ureq::get(url)
        .call()
        .map_err(|error| CliError::io("failed to download artifact url", Some(url), error))?;
    let mut reader = response.into_reader();
    let mut buffer = Vec::new();
    reader.read_to_end(&mut buffer).map_err(|error| {
        CliError::io("failed to read downloaded artifact bytes", Some(url), error)
    })?;
    if buffer.len() > MAX_ARTIFACT_BYTES {
        return Err(CliError::input(format!(
            "downloaded artifact exceeds max allowed size ({} bytes): {url}",
            MAX_ARTIFACT_BYTES
        )));
    }
    Ok(buffer)
}

fn sha256_hex(bytes: &[u8]) -> String {
    use sha2::{Digest, Sha256};

    let mut hasher = Sha256::new();
    hasher.update(bytes);
    let digest = hasher.finalize();
    let mut out = String::with_capacity(digest.len() * 2);
    for byte in digest {
        out.push(hex_char(byte >> 4));
        out.push(hex_char(byte & 0x0f));
    }
    out
}

fn hex_char(value: u8) -> char {
    match value {
        0..=9 => (b'0' + value) as char,
        10..=15 => (b'a' + (value - 10)) as char,
        _ => '0',
    }
}
