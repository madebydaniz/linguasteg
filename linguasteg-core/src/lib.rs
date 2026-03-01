mod catalog;
mod crypto;
mod error;
mod gateway;
mod grammar;
mod ids;
mod orchestration;
mod pipeline;
mod registry;
mod style;
mod symbolic;
mod validation;

pub use catalog::{
    LanguageDescriptor, ModelDescriptor, RealizationTemplateDescriptor, SlotRole,
    StrategyDescriptor, StyleInspiration, StyleProfileDescriptor, StyleStrength,
    TemplateSlotDescriptor, TemplateToken, TextDirection, WritingRegister,
};
pub use crypto::{
    CryptoEnvelopeConfig, CryptoEnvelopeError, CryptoEnvelopeInspection, CryptoEnvelopeMetadata,
    CryptoEnvelopeResult, KeyDerivationParams, inspect_envelope, open_payload,
    open_payload_with_config, seal_payload, seal_payload_with_config,
};
pub use error::{CoreError, CoreResult};
pub use gateway::{
    GatewayFinishReason, GatewayMessage, GatewayMessageRole, GatewayOperation, GatewayRequest,
    GatewayResponse, GatewayUsage, ModelGateway, ModelGatewayRegistry,
};
pub use grammar::{
    GrammarConstraintChecker, LanguageRealizer, RealizationPlan, SlotAssignment,
    render_realization_plan, validate_realization_plan, validate_template_descriptor,
};
pub use ids::{LanguageTag, ModelId, ProviderId, SlotId, StrategyId, StyleProfileId, TemplateId};
pub use orchestration::{OrchestratedDecodeResult, OrchestratedEncodeResult, PipelineOrchestrator};
pub use pipeline::{
    DecodeOutput, DecodeRequest, Decoder, EncodeOutput, EncodeRequest, Encoder, ModelAdapter,
    ModelCapability, ModelSelection, PipelineOptions,
};
pub use registry::TemplateRegistry;
pub use registry::{LanguageRegistry, ModelRegistry, StrategyRegistry, StyleProfileRegistry};
pub use style::{StyleCandidate, StyleRerankRequest, StyleReranker, StyleScorer, StyleSelection};
pub use symbolic::{
    BitRange, FixedWidthBitPlanner, FixedWidthPlanningOptions, SymbolicFieldSpec,
    SymbolicFramePlan, SymbolicFrameSchema, SymbolicPayloadPlan, SymbolicPayloadPlanner,
    SymbolicSlotValue, decode_payload_from_symbolic_frames, plan_payload_to_symbolic_frames,
    validate_symbolic_frame_schema,
};
pub use validation::{
    ValidatedDecodeRequest, ValidatedEncodeRequest, validate_decode_request,
    validate_encode_request,
};
