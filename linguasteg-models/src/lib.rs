pub mod en;
pub mod fa;
pub mod gateway;

use linguasteg_core::{
    CoreError, CoreResult, ModelAdapter, ModelCapability, RealizationPlan, TextExtractor,
};

pub use en::{
    EnglishPrototypeConstraintChecker, EnglishPrototypeLanguagePack, EnglishPrototypeRealizer,
    EnglishPrototypeSymbolicMapper,
};
pub use fa::{
    FarsiPrototypeConstraintChecker, FarsiPrototypeLanguagePack, FarsiPrototypeLexicon,
    FarsiPrototypeRealizer, FarsiPrototypeSymbolicMapper,
};
pub use gateway::{InMemoryGatewayRegistry, StubGateway};

#[derive(Debug, Default)]
pub struct StubModelAdapter;

impl ModelAdapter for StubModelAdapter {
    fn id(&self) -> &str {
        "stub"
    }

    fn supports(&self, _capability: ModelCapability) -> bool {
        false
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct FarsiPrototypeTextExtractor;

impl TextExtractor for FarsiPrototypeTextExtractor {
    fn extract_plans(&self, _stego_text: &str) -> CoreResult<Vec<RealizationPlan>> {
        Err(CoreError::NotImplemented(
            "farsi text extraction pipeline is not wired yet",
        ))
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct EnglishPrototypeTextExtractor;

impl TextExtractor for EnglishPrototypeTextExtractor {
    fn extract_plans(&self, _stego_text: &str) -> CoreResult<Vec<RealizationPlan>> {
        Err(CoreError::NotImplemented(
            "english text extraction pipeline is not wired yet",
        ))
    }
}
