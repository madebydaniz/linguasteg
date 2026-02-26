use crate::{CoreError, CoreResult};

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct LanguageTag(String);

impl LanguageTag {
    pub fn new(value: impl Into<String>) -> CoreResult<Self> {
        let value = normalize_identifier(value.into())?;
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl core::fmt::Display for LanguageTag {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct StrategyId(String);

impl StrategyId {
    pub fn new(value: impl Into<String>) -> CoreResult<Self> {
        let value = normalize_identifier(value.into())?;
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl core::fmt::Display for StrategyId {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ProviderId(String);

impl ProviderId {
    pub fn new(value: impl Into<String>) -> CoreResult<Self> {
        let value = normalize_identifier(value.into())?;
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl core::fmt::Display for ProviderId {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ModelId(String);

impl ModelId {
    pub fn new(value: impl Into<String>) -> CoreResult<Self> {
        let value = normalize_identifier(value.into())?;
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl core::fmt::Display for ModelId {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct StyleProfileId(String);

impl StyleProfileId {
    pub fn new(value: impl Into<String>) -> CoreResult<Self> {
        let value = normalize_identifier(value.into())?;
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl core::fmt::Display for StyleProfileId {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(self.as_str())
    }
}

fn normalize_identifier(value: String) -> CoreResult<String> {
    let normalized = value.trim().to_ascii_lowercase();
    let is_valid = !normalized.is_empty()
        && normalized
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-');

    if is_valid {
        Ok(normalized)
    } else {
        Err(CoreError::InvalidIdentifier(value))
    }
}

#[cfg(test)]
mod tests {
    use super::{LanguageTag, ModelId, ProviderId, StrategyId, StyleProfileId};

    #[test]
    fn language_tag_normalizes_ascii_input() {
        let tag = LanguageTag::new(" FA ").expect("tag should normalize");
        assert_eq!(tag.as_str(), "fa");
    }

    #[test]
    fn strategy_id_rejects_invalid_characters() {
        let strategy = StrategyId::new("synonym_v1");
        assert!(strategy.is_err());
    }

    #[test]
    fn provider_id_normalizes_ascii_input() {
        let provider = ProviderId::new(" OpenAI ").expect("provider should normalize");
        assert_eq!(provider.as_str(), "openai");
    }

    #[test]
    fn model_id_accepts_dash_and_digits() {
        let model = ModelId::new("gpt-4o-mini").expect("model id should be valid");
        assert_eq!(model.as_str(), "gpt-4o-mini");
    }

    #[test]
    fn style_profile_id_normalizes_ascii_input() {
        let profile = StyleProfileId::new(" FA-Formal ").expect("profile id should normalize");
        assert_eq!(profile.as_str(), "fa-formal");
    }
}
