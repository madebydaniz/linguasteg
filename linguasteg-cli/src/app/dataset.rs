use std::collections::HashMap;
use std::collections::HashSet;

use serde::Deserialize;
use serde_json::Value;

const LEXICON_DATASET_KIND: &str = "linguasteg-lexicon-v1";
const LEXICON_DATASET_SCHEMA_VERSION: u8 = 1;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DatasetArtifactMetadata {
    pub(crate) kind: String,
    pub(crate) schema_version: u8,
    pub(crate) language: String,
    pub(crate) entry_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct LexiconDataset {
    pub(crate) kind: String,
    pub(crate) schema_version: u8,
    pub(crate) language: String,
    pub(crate) entries: Vec<LexiconVariantEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct LexiconVariantEntry {
    pub(crate) slot: String,
    pub(crate) canonical: String,
    pub(crate) variants: Vec<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct LexiconVariantCatalog {
    entries: HashMap<String, HashMap<String, Vec<String>>>,
    normalization_rules: Vec<(String, String)>,
}

impl LexiconDataset {
    pub(crate) fn metadata(&self) -> DatasetArtifactMetadata {
        DatasetArtifactMetadata {
            kind: self.kind.clone(),
            schema_version: self.schema_version,
            language: self.language.clone(),
            entry_count: self.entries.len(),
        }
    }

    pub(crate) fn variant_catalog(&self) -> LexiconVariantCatalog {
        let mut entries: HashMap<String, HashMap<String, Vec<String>>> = HashMap::new();
        let mut normalization_rules = Vec::new();
        for entry in &self.entries {
            entries
                .entry(entry.slot.clone())
                .or_default()
                .insert(entry.canonical.clone(), entry.variants.clone());
            for variant in &entry.variants {
                normalization_rules.push((variant.clone(), entry.canonical.clone()));
            }
        }
        normalization_rules.sort_by(|left, right| right.0.len().cmp(&left.0.len()));
        LexiconVariantCatalog {
            entries,
            normalization_rules,
        }
    }
}

impl LexiconVariantCatalog {
    pub(crate) fn select_variant(
        &self,
        slot: &str,
        canonical: &str,
        selector: u64,
    ) -> Option<&str> {
        let variants = self.entries.get(slot)?.get(canonical)?;
        if variants.is_empty() {
            return None;
        }
        let index = (selector as usize) % variants.len();
        variants.get(index).map(String::as_str)
    }

    pub(crate) fn normalize_text(&self, input: &str) -> String {
        let mut normalized = input.to_string();
        for (variant, canonical) in &self.normalization_rules {
            normalized = replace_whole_surface(&normalized, variant, canonical);
        }
        normalized
    }
}

fn replace_whole_surface(input: &str, variant: &str, canonical: &str) -> String {
    if variant.is_empty() || variant == canonical {
        return input.to_string();
    }

    let mut out = String::with_capacity(input.len());
    let mut cursor = 0usize;
    for (start, _) in input.match_indices(variant) {
        if start < cursor {
            continue;
        }
        let end = start + variant.len();
        if !is_boundary(input, start, true) || !is_boundary(input, end, false) {
            continue;
        }
        out.push_str(&input[cursor..start]);
        out.push_str(canonical);
        cursor = end;
    }
    out.push_str(&input[cursor..]);
    out
}

fn is_boundary(input: &str, index: usize, before: bool) -> bool {
    if before {
        if index == 0 {
            return true;
        }
        let Some(ch) = input[..index].chars().next_back() else {
            return true;
        };
        !ch.is_alphanumeric()
    } else {
        if index >= input.len() {
            return true;
        }
        let Some(ch) = input[index..].chars().next() else {
            return true;
        };
        !ch.is_alphanumeric()
    }
}

#[derive(Debug, Deserialize)]
struct RawLexiconDataset {
    kind: String,
    schema_version: u8,
    language: String,
    entries: Vec<RawLexiconEntry>,
}

#[derive(Debug, Deserialize)]
struct RawLexiconEntry {
    slot: String,
    canonical: String,
    variants: Vec<String>,
}

pub(crate) fn load_lexicon_dataset_artifact(
    expected_language: &str,
    bytes: &[u8],
) -> Result<Option<LexiconDataset>, String> {
    if bytes.is_empty() {
        return Ok(None);
    }

    let text = match std::str::from_utf8(bytes) {
        Ok(value) => value.trim(),
        Err(_) => return Ok(None),
    };
    if text.is_empty() {
        return Ok(None);
    }

    let value: Value = match serde_json::from_str(text) {
        Ok(value) => value,
        Err(_) => return Ok(None),
    };

    let Some(kind) = value.get("kind").and_then(Value::as_str) else {
        return Ok(None);
    };
    if kind != LEXICON_DATASET_KIND {
        return Err(format!(
            "unsupported dataset kind '{kind}' (expected '{LEXICON_DATASET_KIND}')"
        ));
    }

    let raw: RawLexiconDataset = serde_json::from_value(value)
        .map_err(|error| format!("invalid lexicon dataset artifact schema: {error}"))?;
    validate_raw_lexicon_dataset(expected_language, raw).map(Some)
}

fn validate_raw_lexicon_dataset(
    expected_language: &str,
    raw: RawLexiconDataset,
) -> Result<LexiconDataset, String> {
    if raw.schema_version != LEXICON_DATASET_SCHEMA_VERSION {
        return Err(format!(
            "unsupported lexicon dataset schema_version {} (expected {})",
            raw.schema_version, LEXICON_DATASET_SCHEMA_VERSION
        ));
    }

    let expected_language = normalize_language_code(expected_language)
        .map_err(|reason| format!("invalid expected language code: {reason}"))?;
    let artifact_language = normalize_language_code(&raw.language)
        .map_err(|reason| format!("invalid dataset language code '{}': {reason}", raw.language))?;
    if artifact_language != expected_language {
        return Err(format!(
            "dataset language '{}' does not match selected language '{}'",
            artifact_language, expected_language
        ));
    }

    if raw.entries.is_empty() {
        return Err("lexicon dataset must contain at least one entry".to_string());
    }

    let mut seen_pairs: HashSet<(String, String)> = HashSet::new();
    let mut entries = Vec::with_capacity(raw.entries.len());
    for entry in raw.entries {
        let slot = normalize_non_empty_text("slot", &entry.slot)?;
        let canonical = normalize_non_empty_text("canonical", &entry.canonical)?;
        if entry.variants.is_empty() {
            return Err(format!(
                "dataset entry '{slot}:{canonical}' must contain at least one variant"
            ));
        }
        if !seen_pairs.insert((slot.clone(), canonical.clone())) {
            return Err(format!(
                "duplicate dataset entry for slot '{}' and canonical '{}'",
                slot, canonical
            ));
        }

        let mut seen_variants = HashSet::new();
        let mut variants = Vec::with_capacity(entry.variants.len());
        for variant in entry.variants {
            let normalized = normalize_non_empty_text("variant", &variant)?;
            if normalized == canonical {
                return Err(format!(
                    "dataset entry '{slot}:{canonical}' includes a variant equal to canonical surface"
                ));
            }
            if !seen_variants.insert(normalized.clone()) {
                return Err(format!(
                    "dataset entry '{slot}:{canonical}' contains duplicate variant '{}'",
                    normalized
                ));
            }
            variants.push(normalized);
        }

        entries.push(LexiconVariantEntry {
            slot,
            canonical,
            variants,
        });
    }

    Ok(LexiconDataset {
        kind: raw.kind,
        schema_version: raw.schema_version,
        language: artifact_language,
        entries,
    })
}

fn normalize_language_code(value: &str) -> Result<String, String> {
    let normalized = value.trim().to_ascii_lowercase();
    if normalized.is_empty() {
        return Err("must not be empty".to_string());
    }
    if normalized.starts_with('-') || normalized.ends_with('-') || normalized.contains("--") {
        return Err("must use lowercase letters, digits, and single '-' separators".to_string());
    }
    if !normalized
        .bytes()
        .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
    {
        return Err("must use lowercase letters, digits, and '-'".to_string());
    }
    Ok(normalized)
}

fn normalize_non_empty_text(field: &str, value: &str) -> Result<String, String> {
    let normalized = value.trim();
    if normalized.is_empty() {
        return Err(format!("dataset field '{field}' must not be empty"));
    }
    Ok(normalized.to_string())
}

#[cfg(test)]
mod tests {
    use super::load_lexicon_dataset_artifact;

    #[test]
    fn load_lexicon_dataset_artifact_parses_valid_payload() {
        let payload = br#"{
            "kind": "linguasteg-lexicon-v1",
            "schema_version": 1,
            "language": "en",
            "entries": [
                {"slot": "object", "canonical": "letter", "variants": ["missive", "epistle"]},
                {"slot": "verb", "canonical": "writes", "variants": ["composes"]}
            ]
        }"#;

        let dataset = load_lexicon_dataset_artifact("en", payload)
            .expect("dataset should parse")
            .expect("dataset should be detected");
        let metadata = dataset.metadata();

        assert_eq!(metadata.kind, "linguasteg-lexicon-v1");
        assert_eq!(metadata.schema_version, 1);
        assert_eq!(metadata.language, "en");
        assert_eq!(metadata.entry_count, 2);
    }

    #[test]
    fn load_lexicon_dataset_artifact_rejects_language_mismatch() {
        let payload = br#"{
            "kind": "linguasteg-lexicon-v1",
            "schema_version": 1,
            "language": "fa",
            "entries": [{"slot":"object","canonical":"letter","variants":["missive"]}]
        }"#;

        let error = load_lexicon_dataset_artifact("en", payload)
            .expect_err("mismatched language should fail");

        assert!(error.contains("does not match selected language"));
    }

    #[test]
    fn load_lexicon_dataset_artifact_returns_none_for_plain_text() {
        let dataset = load_lexicon_dataset_artifact("en", b"simple word list");
        assert!(
            dataset
                .expect("plain text should be treated as opaque")
                .is_none()
        );
    }

    #[test]
    fn lexicon_variant_catalog_normalizes_variants_to_canonical_surfaces() {
        let payload = br#"{
            "kind": "linguasteg-lexicon-v1",
            "schema_version": 1,
            "language": "en",
            "entries": [
                {"slot": "object", "canonical": "letter", "variants": ["missive", "epistle"]},
                {"slot": "verb", "canonical": "writes", "variants": ["composes"]}
            ]
        }"#;
        let dataset = load_lexicon_dataset_artifact("en", payload)
            .expect("dataset should parse")
            .expect("dataset should be detected");
        let catalog = dataset.variant_catalog();
        let text = "the writer composes quiet epistle. the writer composes quiet missive.";
        let normalized = catalog.normalize_text(text);

        assert_eq!(
            normalized,
            "the writer writes quiet letter. the writer writes quiet letter."
        );
    }

    #[test]
    fn lexicon_variant_catalog_normalize_text_handles_overlapping_matches() {
        let payload = br#"{
            "kind": "linguasteg-lexicon-v1",
            "schema_version": 1,
            "language": "en",
            "entries": [
                {"slot": "object", "canonical": "token", "variants": ["a-a"]}
            ]
        }"#;
        let dataset = load_lexicon_dataset_artifact("en", payload)
            .expect("dataset should parse")
            .expect("dataset should be detected");
        let catalog = dataset.variant_catalog();
        let normalized = catalog.normalize_text("a-a-a");

        assert_eq!(normalized, "token-a");
    }
}
