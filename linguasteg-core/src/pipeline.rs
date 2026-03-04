use crate::{CoreResult, LanguageTag, ModelId, ProviderId, StrategyId};
use crate::RealizationPlan;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EncodeRequest {
    pub carrier_text: String,
    pub payload: Vec<u8>,
    pub options: PipelineOptions,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EncodeOutput {
    pub stego_text: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecodeRequest {
    pub stego_text: String,
    pub options: PipelineOptions,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecodeOutput {
    pub payload: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelSelection {
    pub provider: ProviderId,
    pub model: ModelId,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PipelineOptions {
    pub language: LanguageTag,
    pub strategy: StrategyId,
    pub model_selection: Option<ModelSelection>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ModelCapability {
    TokenLogProbabilities,
    ConstrainedGeneration,
    DeterministicSeed,
    StreamingGeneration,
}

impl ModelCapability {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::TokenLogProbabilities => "token-log-probabilities",
            Self::ConstrainedGeneration => "constrained-generation",
            Self::DeterministicSeed => "deterministic-seed",
            Self::StreamingGeneration => "streaming-generation",
        }
    }
}

pub trait ModelAdapter: Send + Sync {
    fn id(&self) -> &str;
    fn supports(&self, capability: ModelCapability) -> bool;
}

pub trait Encoder: Send + Sync {
    fn encode(&self, request: EncodeRequest) -> CoreResult<EncodeOutput>;
}

pub trait Decoder: Send + Sync {
    fn decode(&self, request: DecodeRequest) -> CoreResult<DecodeOutput>;
}

pub trait TextExtractor: Send + Sync {
    fn extract_plans(&self, stego_text: &str) -> CoreResult<Vec<RealizationPlan>>;
}
