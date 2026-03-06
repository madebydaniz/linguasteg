use linguasteg_core::{
    CoreError, CoreResult, FixedWidthBitPlanner, GrammarConstraintChecker, LanguageDescriptor,
    LanguageRealizer, LanguageRegistry, LanguageTag, ModelCapability, ModelDescriptor, ModelId,
    ModelRegistry, ModelSelection, PipelineOptions, PipelineOrchestrator, ProviderId,
    RealizationPlan, RealizationTemplateDescriptor, StrategyDescriptor, StrategyId,
    StrategyRegistry, StyleProfileDescriptor, StyleProfileId, StyleProfileRegistry,
    SymbolicFramePlan, SymbolicFrameSchema, SymbolicPayloadPlan, TemplateRegistry, TextExtractor,
};
use linguasteg_models::{
    EnglishPrototypeConstraintChecker, EnglishPrototypeLanguagePack, EnglishPrototypeRealizer,
    EnglishPrototypeSymbolicMapper, EnglishPrototypeTextExtractor, FarsiPrototypeConstraintChecker,
    FarsiPrototypeLanguagePack, FarsiPrototypeRealizer, FarsiPrototypeSymbolicMapper,
    FarsiPrototypeTextExtractor, InMemoryGatewayRegistry,
};

use super::types::{CliError, ProtoTarget};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct SupportedLanguageInfo {
    pub(crate) code: &'static str,
    pub(crate) display: &'static str,
    pub(crate) direction: &'static str,
}

pub(crate) struct SupportedStrategyInfo {
    pub(crate) id: &'static str,
    pub(crate) display: &'static str,
    pub(crate) required_capabilities: &'static [&'static str],
}

pub(crate) struct SupportedModelInfo {
    pub(crate) provider: &'static str,
    pub(crate) id: &'static str,
    pub(crate) display: &'static str,
    pub(crate) languages: &'static [&'static str],
    pub(crate) capabilities: &'static [&'static str],
}

pub(crate) fn supported_languages() -> Vec<SupportedLanguageInfo> {
    runtime_providers()
        .iter()
        .map(|provider| SupportedLanguageInfo {
            code: provider.language_code(),
            display: provider.language_display(),
            direction: provider.direction(),
        })
        .collect()
}

pub(crate) fn runtime_supports_language_code(language_code: &str) -> bool {
    runtime_provider_for_code(language_code).is_some()
}

pub(crate) fn supported_language_codes_csv() -> String {
    runtime_providers()
        .iter()
        .map(|provider| provider.language_code())
        .collect::<Vec<_>>()
        .join(", ")
}

pub(crate) fn initialize_runtime(target: ProtoTarget) -> Result<PrototypeRuntime, CliError> {
    let language_code = target.as_str().to_string();
    if !runtime_supports_language_code(&language_code) {
        return Err(CliError::config(format!(
            "language '{}' is not supported by runtime providers (supported: {})",
            language_code,
            supported_language_codes_csv()
        )));
    }

    PrototypeRuntime::new(target).map_err(|error| {
        CliError::config(format!(
            "failed to initialize '{}' runtime: {error}",
            language_code
        ))
    })
}

const SUPPORTED_STRATEGIES: [SupportedStrategyInfo; 1] = [SupportedStrategyInfo {
    id: "symbolic-stub",
    display: "Symbolic Stub",
    required_capabilities: &["deterministic-seed"],
}];

pub(crate) fn supported_strategies() -> &'static [SupportedStrategyInfo] {
    &SUPPORTED_STRATEGIES
}

const SUPPORTED_MODELS: [SupportedModelInfo; 1] = [SupportedModelInfo {
    provider: "stub",
    id: "stub-local",
    display: "Stub Local",
    languages: &["fa", "en"],
    capabilities: &["deterministic-seed"],
}];

pub(crate) fn supported_models() -> &'static [SupportedModelInfo] {
    &SUPPORTED_MODELS
}

struct InMemoryStrategyRegistry {
    strategies: Vec<StrategyDescriptor>,
}

impl StrategyRegistry for InMemoryStrategyRegistry {
    fn all_strategies(&self) -> &[StrategyDescriptor] {
        &self.strategies
    }
}

struct InMemoryModelRegistry {
    models: Vec<ModelDescriptor>,
}

impl ModelRegistry for InMemoryModelRegistry {
    fn all_models(&self) -> &[ModelDescriptor] {
        &self.models
    }
}

trait RuntimeLanguagePack: LanguageRegistry + StyleProfileRegistry + TemplateRegistry {}

impl<T> RuntimeLanguagePack for T where T: LanguageRegistry + StyleProfileRegistry + TemplateRegistry
{}

pub(crate) struct RuntimeLanguagePackHandle {
    inner: Box<dyn RuntimeLanguagePack>,
}

impl RuntimeLanguagePackHandle {
    fn new(inner: Box<dyn RuntimeLanguagePack>) -> Self {
        Self { inner }
    }
}

impl LanguageRegistry for RuntimeLanguagePackHandle {
    fn all_languages(&self) -> &[LanguageDescriptor] {
        self.inner.all_languages()
    }
}

impl StyleProfileRegistry for RuntimeLanguagePackHandle {
    fn all_style_profiles(&self) -> &[StyleProfileDescriptor] {
        self.inner.all_style_profiles()
    }
}

impl TemplateRegistry for RuntimeLanguagePackHandle {
    fn all_templates(&self) -> &[RealizationTemplateDescriptor] {
        self.inner.all_templates()
    }
}

pub(crate) trait RuntimeSymbolicMapper: Send + Sync {
    fn frame_schemas(&self) -> Vec<SymbolicFrameSchema>;

    fn map_payload_to_plans_with_profile(
        &self,
        payload_plan: &SymbolicPayloadPlan,
        profile_id: Option<&StyleProfileId>,
    ) -> CoreResult<Vec<RealizationPlan>>;

    fn map_plans_to_frames(&self, plans: &[RealizationPlan]) -> CoreResult<Vec<SymbolicFramePlan>>;
}

impl RuntimeSymbolicMapper for FarsiPrototypeSymbolicMapper {
    fn frame_schemas(&self) -> Vec<SymbolicFrameSchema> {
        FarsiPrototypeSymbolicMapper::frame_schemas(self)
    }

    fn map_payload_to_plans_with_profile(
        &self,
        payload_plan: &SymbolicPayloadPlan,
        profile_id: Option<&StyleProfileId>,
    ) -> CoreResult<Vec<RealizationPlan>> {
        FarsiPrototypeSymbolicMapper::map_payload_to_plans_with_profile(
            self,
            payload_plan,
            profile_id,
        )
    }

    fn map_plans_to_frames(&self, plans: &[RealizationPlan]) -> CoreResult<Vec<SymbolicFramePlan>> {
        FarsiPrototypeSymbolicMapper::map_plans_to_frames(self, plans)
    }
}

impl RuntimeSymbolicMapper for EnglishPrototypeSymbolicMapper {
    fn frame_schemas(&self) -> Vec<SymbolicFrameSchema> {
        EnglishPrototypeSymbolicMapper::frame_schemas(self)
    }

    fn map_payload_to_plans_with_profile(
        &self,
        payload_plan: &SymbolicPayloadPlan,
        profile_id: Option<&StyleProfileId>,
    ) -> CoreResult<Vec<RealizationPlan>> {
        EnglishPrototypeSymbolicMapper::map_payload_to_plans_with_profile(
            self,
            payload_plan,
            profile_id,
        )
    }

    fn map_plans_to_frames(&self, plans: &[RealizationPlan]) -> CoreResult<Vec<SymbolicFramePlan>> {
        EnglishPrototypeSymbolicMapper::map_plans_to_frames(self, plans)
    }
}

trait RuntimeProvider: Send + Sync {
    fn language_code(&self) -> &'static str;
    fn language_display(&self) -> &'static str;
    fn direction(&self) -> &'static str;
    fn build_components(&self) -> RuntimeComponents;
}

#[derive(Debug, Clone, Copy)]
struct FarsiRuntimeProvider;

impl RuntimeProvider for FarsiRuntimeProvider {
    fn language_code(&self) -> &'static str {
        "fa"
    }

    fn language_display(&self) -> &'static str {
        "Farsi"
    }

    fn direction(&self) -> &'static str {
        "rtl"
    }

    fn build_components(&self) -> RuntimeComponents {
        RuntimeComponents {
            language_code: self.language_code(),
            language_display: self.language_display(),
            text_decode_lossless: true,
            pack: Box::new(FarsiPrototypeLanguagePack::default()),
            checker: Box::new(FarsiPrototypeConstraintChecker),
            realizer: Box::new(FarsiPrototypeRealizer),
            extractor: Box::new(FarsiPrototypeTextExtractor),
            mapper: Box::new(FarsiPrototypeSymbolicMapper),
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct EnglishRuntimeProvider;

impl RuntimeProvider for EnglishRuntimeProvider {
    fn language_code(&self) -> &'static str {
        "en"
    }

    fn language_display(&self) -> &'static str {
        "English"
    }

    fn direction(&self) -> &'static str {
        "ltr"
    }

    fn build_components(&self) -> RuntimeComponents {
        RuntimeComponents {
            language_code: self.language_code(),
            language_display: self.language_display(),
            text_decode_lossless: true,
            pack: Box::new(EnglishPrototypeLanguagePack::default()),
            checker: Box::new(EnglishPrototypeConstraintChecker),
            realizer: Box::new(EnglishPrototypeRealizer),
            extractor: Box::new(EnglishPrototypeTextExtractor),
            mapper: Box::new(EnglishPrototypeSymbolicMapper),
        }
    }
}

static FARSI_RUNTIME_PROVIDER: FarsiRuntimeProvider = FarsiRuntimeProvider;
static ENGLISH_RUNTIME_PROVIDER: EnglishRuntimeProvider = EnglishRuntimeProvider;
static RUNTIME_PROVIDERS: [&dyn RuntimeProvider; 2] =
    [&FARSI_RUNTIME_PROVIDER, &ENGLISH_RUNTIME_PROVIDER];

fn runtime_providers() -> &'static [&'static dyn RuntimeProvider] {
    &RUNTIME_PROVIDERS
}

fn runtime_provider_for_code(language_code: &str) -> Option<&'static dyn RuntimeProvider> {
    runtime_providers()
        .iter()
        .copied()
        .find(|provider| provider.language_code() == language_code)
}

struct RuntimeComponents {
    language_code: &'static str,
    language_display: &'static str,
    text_decode_lossless: bool,
    pack: Box<dyn RuntimeLanguagePack>,
    checker: Box<dyn GrammarConstraintChecker>,
    realizer: Box<dyn LanguageRealizer>,
    extractor: Box<dyn TextExtractor>,
    mapper: Box<dyn RuntimeSymbolicMapper>,
}

pub(crate) struct PrototypeRuntime {
    pub(crate) language_code: &'static str,
    pub(crate) language_display: &'static str,
    pub(crate) text_decode_lossless: bool,
    pub(crate) pack: RuntimeLanguagePackHandle,
    pub(crate) checker: Box<dyn GrammarConstraintChecker>,
    pub(crate) realizer: Box<dyn LanguageRealizer>,
    #[allow(dead_code)]
    pub(crate) extractor: Box<dyn TextExtractor>,
    pub(crate) mapper: Box<dyn RuntimeSymbolicMapper>,
    planner: FixedWidthBitPlanner,
    strategy_registry: InMemoryStrategyRegistry,
    model_registry: InMemoryModelRegistry,
    gateway_registry: InMemoryGatewayRegistry,
}

impl PrototypeRuntime {
    pub(crate) fn new(target: ProtoTarget) -> Result<Self, Box<dyn std::error::Error>> {
        Self::new_for_language_code(target.as_str())
    }

    pub(crate) fn new_for_language_code(
        language_code: &str,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let strategy_id = StrategyId::new("symbolic-stub")?;
        let provider = ProviderId::new("stub")?;
        let model = ModelId::new("stub-local")?;

        let provider_impl = runtime_provider_for_code(language_code)
            .ok_or_else(|| CoreError::UnsupportedLanguage(language_code.to_string()))?;
        let components = provider_impl.build_components();
        let language = LanguageTag::new(components.language_code)?;

        Ok(Self {
            language_code: components.language_code,
            language_display: components.language_display,
            text_decode_lossless: components.text_decode_lossless,
            pack: RuntimeLanguagePackHandle::new(components.pack),
            checker: components.checker,
            realizer: components.realizer,
            extractor: components.extractor,
            mapper: components.mapper,
            planner: FixedWidthBitPlanner::default(),
            strategy_registry: InMemoryStrategyRegistry {
                strategies: vec![StrategyDescriptor {
                    id: strategy_id,
                    display_name: "Symbolic Stub".to_string(),
                    required_capabilities: vec![ModelCapability::DeterministicSeed],
                }],
            },
            model_registry: InMemoryModelRegistry {
                models: vec![ModelDescriptor {
                    provider,
                    model,
                    display_name: "Stub Local".to_string(),
                    supported_languages: vec![language],
                    capabilities: vec![ModelCapability::DeterministicSeed],
                }],
            },
            gateway_registry: InMemoryGatewayRegistry::with_stub(),
        })
    }

    pub(crate) fn orchestrator(&self) -> PipelineOrchestrator<'_> {
        PipelineOrchestrator::new(
            &self.pack,
            &self.strategy_registry,
            &self.model_registry,
            &self.gateway_registry,
            &self.planner,
        )
    }

    #[allow(dead_code)]
    pub(crate) fn extract_plans(&self, stego_text: &str) -> CoreResult<Vec<RealizationPlan>> {
        self.extractor.extract_plans(stego_text)
    }

    pub(crate) fn pipeline_options(&self) -> Result<PipelineOptions, Box<dyn std::error::Error>> {
        Ok(PipelineOptions {
            language: LanguageTag::new(self.language_code)?,
            strategy: StrategyId::new("symbolic-stub")?,
            model_selection: Some(ModelSelection {
                provider: ProviderId::new("stub")?,
                model: ModelId::new("stub-local")?,
            }),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{
        PrototypeRuntime, initialize_runtime, supported_language_codes_csv, supported_languages,
    };
    use crate::app::types::ProtoTarget;

    #[test]
    fn supported_languages_are_provided_by_runtime_registry() {
        let languages = supported_languages();
        assert!(languages.iter().any(|item| item.code == "fa"));
        assert!(languages.iter().any(|item| item.code == "en"));
    }

    #[test]
    fn runtime_can_be_initialized_from_language_code() {
        let fa_runtime =
            PrototypeRuntime::new_for_language_code("fa").expect("fa runtime should initialize");
        assert_eq!(fa_runtime.language_code, "fa");

        let en_runtime =
            PrototypeRuntime::new_for_language_code("en").expect("en runtime should initialize");
        assert_eq!(en_runtime.language_code, "en");
    }

    #[test]
    fn runtime_initialization_rejects_unknown_language_code() {
        let error = match PrototypeRuntime::new_for_language_code("de") {
            Ok(_) => panic!("unknown language runtime should fail"),
            Err(error) => error,
        };
        assert!(error.to_string().contains("language is not supported: de"));
    }

    #[test]
    fn initialize_runtime_reports_supported_language_codes_for_unknown_target() {
        let error = match initialize_runtime(ProtoTarget::Other("de".to_string())) {
            Ok(_) => panic!("unknown runtime should fail"),
            Err(error) => error,
        };
        assert_eq!(error.code(), "LSTEG-CLI-CFG-001");
        assert!(
            error
                .message()
                .contains("language 'de' is not supported by runtime providers")
        );
        assert!(error.message().contains("supported: fa, en"));
    }

    #[test]
    fn supported_language_codes_csv_lists_registered_providers() {
        let csv = supported_language_codes_csv();
        assert!(csv.contains("fa"));
        assert!(csv.contains("en"));
    }
}
