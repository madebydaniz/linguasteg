pub mod en;
pub mod fa;
pub mod gateway;

use linguasteg_core::{ModelAdapter, ModelCapability};

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
