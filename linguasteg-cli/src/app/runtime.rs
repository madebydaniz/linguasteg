use linguasteg_core::{
    CoreResult, FixedWidthBitPlanner, GrammarConstraintChecker, LanguageDescriptor,
    LanguageRealizer, LanguageRegistry, LanguageTag, ModelCapability, ModelDescriptor, ModelId,
    ModelRegistry, ModelSelection, PipelineOptions, PipelineOrchestrator, ProviderId,
    RealizationPlan, RealizationTemplateDescriptor, StrategyDescriptor, StrategyId,
    StrategyRegistry, StyleProfileDescriptor, StyleProfileRegistry, SymbolicFrameSchema,
    SymbolicPayloadPlan, TemplateRegistry, TextExtractor,
};
use linguasteg_models::{
    EnglishPrototypeConstraintChecker, EnglishPrototypeLanguagePack, EnglishPrototypeRealizer,
    EnglishPrototypeSymbolicMapper, EnglishPrototypeTextExtractor, FarsiPrototypeConstraintChecker,
    FarsiPrototypeLanguagePack, FarsiPrototypeRealizer, FarsiPrototypeSymbolicMapper,
    FarsiPrototypeTextExtractor, InMemoryGatewayRegistry,
};

use super::types::ProtoTarget;

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

const SUPPORTED_LANGUAGES: [SupportedLanguageInfo; 2] = [
    SupportedLanguageInfo {
        code: "fa",
        display: "Farsi",
        direction: "rtl",
    },
    SupportedLanguageInfo {
        code: "en",
        display: "English",
        direction: "ltr",
    },
];

pub(crate) fn supported_languages() -> &'static [SupportedLanguageInfo] {
    &SUPPORTED_LANGUAGES
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

    fn map_payload_to_plans(
        &self,
        payload_plan: &SymbolicPayloadPlan,
    ) -> CoreResult<Vec<RealizationPlan>>;
}

impl RuntimeSymbolicMapper for FarsiPrototypeSymbolicMapper {
    fn frame_schemas(&self) -> Vec<SymbolicFrameSchema> {
        FarsiPrototypeSymbolicMapper::frame_schemas(self)
    }

    fn map_payload_to_plans(
        &self,
        payload_plan: &SymbolicPayloadPlan,
    ) -> CoreResult<Vec<RealizationPlan>> {
        FarsiPrototypeSymbolicMapper::map_payload_to_plans(self, payload_plan)
    }
}

impl RuntimeSymbolicMapper for EnglishPrototypeSymbolicMapper {
    fn frame_schemas(&self) -> Vec<SymbolicFrameSchema> {
        EnglishPrototypeSymbolicMapper::frame_schemas(self)
    }

    fn map_payload_to_plans(
        &self,
        payload_plan: &SymbolicPayloadPlan,
    ) -> CoreResult<Vec<RealizationPlan>> {
        EnglishPrototypeSymbolicMapper::map_payload_to_plans(self, payload_plan)
    }
}

struct RuntimeComponents {
    language_code: &'static str,
    language_display: &'static str,
    pack: Box<dyn RuntimeLanguagePack>,
    checker: Box<dyn GrammarConstraintChecker>,
    realizer: Box<dyn LanguageRealizer>,
    extractor: Box<dyn TextExtractor>,
    mapper: Box<dyn RuntimeSymbolicMapper>,
}

pub(crate) struct PrototypeRuntime {
    pub(crate) language_code: &'static str,
    pub(crate) language_display: &'static str,
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
        let strategy_id = StrategyId::new("symbolic-stub")?;
        let provider = ProviderId::new("stub")?;
        let model = ModelId::new("stub-local")?;

        let components = runtime_components(target);
        let language = LanguageTag::new(components.language_code)?;

        Ok(Self {
            language_code: components.language_code,
            language_display: components.language_display,
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

fn runtime_components(target: ProtoTarget) -> RuntimeComponents {
    match target {
        ProtoTarget::Farsi => RuntimeComponents {
            language_code: "fa",
            language_display: "Farsi",
            pack: Box::new(FarsiPrototypeLanguagePack::default()),
            checker: Box::new(FarsiPrototypeConstraintChecker),
            realizer: Box::new(FarsiPrototypeRealizer),
            extractor: Box::new(FarsiPrototypeTextExtractor),
            mapper: Box::new(FarsiPrototypeSymbolicMapper),
        },
        ProtoTarget::English => RuntimeComponents {
            language_code: "en",
            language_display: "English",
            pack: Box::new(EnglishPrototypeLanguagePack::default()),
            checker: Box::new(EnglishPrototypeConstraintChecker),
            realizer: Box::new(EnglishPrototypeRealizer),
            extractor: Box::new(EnglishPrototypeTextExtractor),
            mapper: Box::new(EnglishPrototypeSymbolicMapper),
        },
    }
}
