pub mod fa;

use linguasteg_core::{ModelAdapter, ModelCapability};

pub use fa::{FarsiPrototypeConstraintChecker, FarsiPrototypeLanguagePack, FarsiPrototypeRealizer};

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
