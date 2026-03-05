use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use super::types::{
    CliError, DataCommand, DataInstallOptions, DataListOptions, OutputFormat, ProtoTarget,
};

const DATA_STATE_FILE: &str = "state.json";
const DATA_SOURCES_MANIFEST: &str = include_str!("../../assets/data_sources.json");
const MANIFEST_SCHEMA_VERSION: u8 = 1;

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
struct InstalledSourceManifest {
    schema_version: u8,
    language: String,
    source_id: String,
    source_url: String,
    version: String,
    license: String,
    checksum_sha256: Option<String>,
    installed_at_epoch_sec: u64,
}

pub(crate) fn run_data_command(command: DataCommand) -> Result<(), CliError> {
    match command {
        DataCommand::List(options) => run_data_list(options),
        DataCommand::Install(options) => run_data_install(options, false),
        DataCommand::Update(options) => run_data_install(options, true),
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
                .is_none_or(|target| source.language == target)
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
        let source = resolve_source_for_target(&sources, *target, options.source_id.as_deref())?;
        let status = upsert_install_state(&mut state, source, now, force_refresh);
        let manifest_path = write_install_manifest(&data_dir, source, now)?;
        items.push(DataInstallItem {
            language: source.language.as_str().to_string(),
            source_id: source.id.clone(),
            version: source.version.clone(),
            status: status.to_string(),
            manifest_path: manifest_path.to_string_lossy().to_string(),
        });
    }

    save_data_state(&data_dir, &state)?;

    let mode = if force_refresh {
        "data-update"
    } else {
        "data-install"
    };
    let note = "metadata and cache manifest were prepared; remote fetch pipeline is controlled by future data-fetch worker";

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
        println!(
            "- {}/{} version:{} status:{} manifest:{}",
            item.language, item.source_id, item.version, item.status, item.manifest_path
        );
    }
    Ok(())
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

fn parse_data_language(value: &str) -> Result<ProtoTarget, CliError> {
    match value {
        "fa" => Ok(ProtoTarget::Farsi),
        "en" => Ok(ProtoTarget::English),
        _ => Err(CliError::config(format!(
            "unsupported language '{value}' in data sources manifest"
        ))),
    }
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

fn write_install_manifest(
    data_dir: &Path,
    source: &DataSource,
    installed_at_epoch_sec: u64,
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
