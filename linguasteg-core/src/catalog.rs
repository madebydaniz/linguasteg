use crate::{LanguageTag, ModelCapability, ModelId, ProviderId, StrategyId};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TextDirection {
    LeftToRight,
    RightToLeft,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LanguageDescriptor {
    pub tag: LanguageTag,
    pub display_name: String,
    pub direction: TextDirection,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StrategyDescriptor {
    pub id: StrategyId,
    pub display_name: String,
    pub required_capabilities: Vec<ModelCapability>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelDescriptor {
    pub provider: ProviderId,
    pub model: ModelId,
    pub display_name: String,
    pub supported_languages: Vec<LanguageTag>,
    pub capabilities: Vec<ModelCapability>,
}
