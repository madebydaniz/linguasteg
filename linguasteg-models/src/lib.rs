use linguasteg_core::{ModelAdapter, ModelCapability};

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
