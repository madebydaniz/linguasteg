mod catalog;
mod error;
mod grammar;
mod ids;
mod pipeline;
mod registry;
mod style;
mod validation;

pub use catalog::{
    LanguageDescriptor, ModelDescriptor, StrategyDescriptor, StyleInspiration,
    StyleProfileDescriptor, StyleStrength, TextDirection, WritingRegister, SlotRole,
    RealizationTemplateDescriptor, TemplateSlotDescriptor, TemplateToken,
};
pub use error::{CoreError, CoreResult};
pub use grammar::{
    GrammarConstraintChecker, LanguageRealizer, RealizationPlan, SlotAssignment,
    render_realization_plan, validate_realization_plan, validate_template_descriptor,
};
pub use ids::{
    LanguageTag, ModelId, ProviderId, SlotId, StrategyId, StyleProfileId, TemplateId,
};
pub use pipeline::{
    DecodeOutput, DecodeRequest, Decoder, EncodeOutput, EncodeRequest, Encoder, ModelAdapter,
    ModelCapability, ModelSelection, PipelineOptions,
};
pub use registry::{LanguageRegistry, ModelRegistry, StrategyRegistry, StyleProfileRegistry};
pub use registry::TemplateRegistry;
pub use style::{
    StyleCandidate, StyleRerankRequest, StyleReranker, StyleScorer, StyleSelection,
};
pub use validation::{
    ValidatedDecodeRequest, ValidatedEncodeRequest, validate_decode_request, validate_encode_request,
};
