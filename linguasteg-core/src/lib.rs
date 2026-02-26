mod catalog;
mod error;
mod ids;
mod pipeline;
mod registry;

pub use catalog::{LanguageDescriptor, ModelDescriptor, StrategyDescriptor, TextDirection};
pub use error::{CoreError, CoreResult};
pub use ids::{LanguageTag, ModelId, ProviderId, StrategyId};
pub use pipeline::{
    DecodeOutput, DecodeRequest, Decoder, EncodeOutput, EncodeRequest, Encoder, ModelAdapter,
    ModelCapability,
};
pub use registry::{LanguageRegistry, ModelRegistry, StrategyRegistry};
