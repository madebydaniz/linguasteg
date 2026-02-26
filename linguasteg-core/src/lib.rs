mod catalog;
mod error;
mod ids;
mod pipeline;
mod registry;
mod validation;

pub use catalog::{LanguageDescriptor, ModelDescriptor, StrategyDescriptor, TextDirection};
pub use error::{CoreError, CoreResult};
pub use ids::{LanguageTag, ModelId, ProviderId, StrategyId};
pub use pipeline::{
    DecodeOutput, DecodeRequest, Decoder, EncodeOutput, EncodeRequest, Encoder, ModelAdapter,
    ModelCapability, ModelSelection,
};
pub use registry::{LanguageRegistry, ModelRegistry, StrategyRegistry};
pub use validation::{ValidatedEncodeRequest, validate_encode_request};
