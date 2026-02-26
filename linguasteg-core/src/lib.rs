mod catalog;
mod error;
mod ids;
mod pipeline;
mod registry;
mod style;
mod validation;

pub use catalog::{
    LanguageDescriptor, ModelDescriptor, StrategyDescriptor, StyleInspiration,
    StyleProfileDescriptor, StyleStrength, TextDirection, WritingRegister,
};
pub use error::{CoreError, CoreResult};
pub use ids::{LanguageTag, ModelId, ProviderId, StrategyId, StyleProfileId};
pub use pipeline::{
    DecodeOutput, DecodeRequest, Decoder, EncodeOutput, EncodeRequest, Encoder, ModelAdapter,
    ModelCapability, ModelSelection, PipelineOptions,
};
pub use registry::{LanguageRegistry, ModelRegistry, StrategyRegistry, StyleProfileRegistry};
pub use style::{
    StyleCandidate, StyleRerankRequest, StyleReranker, StyleScorer, StyleSelection,
};
pub use validation::{
    ValidatedDecodeRequest, ValidatedEncodeRequest, validate_decode_request, validate_encode_request,
};
