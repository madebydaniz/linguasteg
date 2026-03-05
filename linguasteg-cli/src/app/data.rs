use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use super::types::{
    CliError, DataCommand, DataInstallOptions, DataListOptions, OutputFormat, ProtoTarget,
};

#[derive(Debug, Clone, Copy)]
struct DataSource {
    id: &'static str,
    language: ProtoTarget,
    source_url: &'static str,
    license: &'static str,
    version: &'static str,
    checksum_sha256: Option<&'static str>,
    recommended: bool,
}

const DATA_STATE_FILE: &str = "state.json";

const DATA_SOURCES: [DataSource; 5] = [
    DataSource {
        id: "en-wordnet-princeton",
        language: ProtoTarget::English,
        source_url: "https://wordnet.princeton.edu/",
        license: "Princeton WordNet License",
        version: "3.1",
        checksum_sha256: None,
        recommended: true,
    },
    DataSource {
        id: "en-wordlist-wordnik",
        language: ProtoTarget::English,
        source_url: "https://github.com/wordnik/wordlist",
        license: "MIT",
        version: "main",
        checksum_sha256: None,
        recommended: false,
    },
    DataSource {
        id: "fa-wordlist-cc0",
        language: ProtoTarget::Farsi,
        source_url: "https://github.com/jadijadi/persianwords",
        license: "CC0-1.0",
        version: "main",
        checksum_sha256: None,
        recommended: true,
    },
    DataSource {
        id: "fa-wordlist-mit",
        language: ProtoTarget::Farsi,
        source_url: "https://github.com/mvalipour/word-list-fa",
        license: "MIT",
        version: "main",
        checksum_sha256: None,
        recommended: false,
    },
    DataSource {
        id: "fa-kaikki-wiktionary",
        language: ProtoTarget::Farsi,
        source_url: "https://kaikki.org/dictionary/Persian/index.html",
        license: "CC-BY-SA-3.0 + GFDL",
        version: "latest",
        checksum_sha256: None,
        recommended: false,
    },
];

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
    let data_dir = resolve_data_dir(options.data_dir.as_deref());
    let state = load_data_state(&data_dir)?;
    let items = DATA_SOURCES
        .iter()
        .filter(|source| {
            options
                .target
                .is_none_or(|target| target == source.language)
        })
        .map(|source| {
            let installed = state.installs.iter().any(|record| {
                record.language == source.language.as_str() && record.source_id == source.id
            });
            DataListItem {
                language: source.language.as_str().to_string(),
                source_id: source.id.to_string(),
                version: source.version.to_string(),
                source_url: source.source_url.to_string(),
                license: source.license.to_string(),
                checksum_sha256: source.checksum_sha256.map(str::to_string),
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
        let source = recommended_source_for(*target).ok_or_else(|| {
            CliError::config(format!(
                "no recommended data source is registered for language '{}'",
                target.as_str()
            ))
        })?;
        let status = upsert_install_state(&mut state, source, now, force_refresh);
        let manifest_path = write_install_manifest(&data_dir, source, now)?;
        items.push(DataInstallItem {
            language: source.language.as_str().to_string(),
            source_id: source.id.to_string(),
            version: source.version.to_string(),
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

fn recommended_source_for(target: ProtoTarget) -> Option<&'static DataSource> {
    DATA_SOURCES
        .iter()
        .find(|source| source.language == target && source.recommended)
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
                record.version = source.version.to_string();
                record.installed_at_epoch_sec = now;
                "updated"
            } else {
                "up-to-date"
            }
        }
        None => {
            state.installs.push(InstallRecord {
                language: source.language.as_str().to_string(),
                source_id: source.id.to_string(),
                version: source.version.to_string(),
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
    let source_dir = language_dir.join(source.id);
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
        source_id: source.id.to_string(),
        source_url: source.source_url.to_string(),
        version: source.version.to_string(),
        license: source.license.to_string(),
        checksum_sha256: source.checksum_sha256.map(str::to_string),
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
