use crate::{CoreResult, LanguageTag, StrategyId};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EncodeRequest {
    pub carrier_text: String,
    pub payload: Vec<u8>,
    pub language: LanguageTag,
    pub strategy: StrategyId,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EncodeOutput {
    pub stego_text: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecodeRequest {
    pub stego_text: String,
    pub language: LanguageTag,
    pub strategy: StrategyId,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecodeOutput {
    pub payload: Vec<u8>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ModelCapability {
    TokenLogProbabilities,
    ConstrainedGeneration,
    DeterministicSeed,
    StreamingGeneration,
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
