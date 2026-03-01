use linguasteg_core::{
    CoreResult, FixedWidthBitPlanner, GrammarConstraintChecker, LanguageRealizer, LanguageRegistry,
    LanguageTag, ModelCapability, ModelDescriptor, ModelId, ModelRegistry, ModelSelection,
    PipelineOptions, PipelineOrchestrator, ProviderId, RealizationPlan, StrategyDescriptor,
    StrategyId, StrategyRegistry, StyleProfileRegistry, SymbolicFrameSchema, SymbolicPayloadPlan,
    TemplateRegistry,
};
use linguasteg_models::{
    EnglishPrototypeConstraintChecker, EnglishPrototypeLanguagePack, EnglishPrototypeRealizer,
    EnglishPrototypeSymbolicMapper, FarsiPrototypeConstraintChecker, FarsiPrototypeLanguagePack,
    FarsiPrototypeRealizer, FarsiPrototypeSymbolicMapper, InMemoryGatewayRegistry,
};

use super::types::ProtoTarget;

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

pub(crate) struct PrototypeRuntime {
    pub(crate) language_code: &'static str,
    pub(crate) language_display: &'static str,
    pub(crate) pack: PackVariant,
    pub(crate) checker: CheckerVariant,
    pub(crate) realizer: RealizerVariant,
    pub(crate) mapper: MapperVariant,
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
        let language = LanguageTag::new(target.as_str())?;

        let (language_code, language_display, pack, checker, realizer, mapper) = match target {
            ProtoTarget::Farsi => (
                "fa",
                "Farsi",
                PackVariant::Farsi(FarsiPrototypeLanguagePack::default()),
                CheckerVariant::Farsi(FarsiPrototypeConstraintChecker),
                RealizerVariant::Farsi(FarsiPrototypeRealizer),
                MapperVariant::Farsi(FarsiPrototypeSymbolicMapper),
            ),
            ProtoTarget::English => (
                "en",
                "English",
                PackVariant::English(EnglishPrototypeLanguagePack::default()),
                CheckerVariant::English(EnglishPrototypeConstraintChecker),
                RealizerVariant::English(EnglishPrototypeRealizer),
                MapperVariant::English(EnglishPrototypeSymbolicMapper),
            ),
        };

        Ok(Self {
            language_code,
            language_display,
            pack,
            checker,
            realizer,
            mapper,
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

pub(crate) enum PackVariant {
    Farsi(FarsiPrototypeLanguagePack),
    English(EnglishPrototypeLanguagePack),
}

impl LanguageRegistry for PackVariant {
    fn all_languages(&self) -> &[linguasteg_core::LanguageDescriptor] {
        match self {
            Self::Farsi(pack) => pack.all_languages(),
            Self::English(pack) => pack.all_languages(),
        }
    }
}

impl StyleProfileRegistry for PackVariant {
    fn all_style_profiles(&self) -> &[linguasteg_core::StyleProfileDescriptor] {
        match self {
            Self::Farsi(pack) => pack.all_style_profiles(),
            Self::English(pack) => pack.all_style_profiles(),
        }
    }
}

impl TemplateRegistry for PackVariant {
    fn all_templates(&self) -> &[linguasteg_core::RealizationTemplateDescriptor] {
        match self {
            Self::Farsi(pack) => pack.all_templates(),
            Self::English(pack) => pack.all_templates(),
        }
    }
}

pub(crate) enum CheckerVariant {
    Farsi(FarsiPrototypeConstraintChecker),
    English(EnglishPrototypeConstraintChecker),
}

impl GrammarConstraintChecker for CheckerVariant {
    fn validate_plan(
        &self,
        template: &linguasteg_core::RealizationTemplateDescriptor,
        plan: &linguasteg_core::RealizationPlan,
    ) -> CoreResult<()> {
        match self {
            Self::Farsi(checker) => checker.validate_plan(template, plan),
            Self::English(checker) => checker.validate_plan(template, plan),
        }
    }
}

pub(crate) enum RealizerVariant {
    Farsi(FarsiPrototypeRealizer),
    English(EnglishPrototypeRealizer),
}

impl LanguageRealizer for RealizerVariant {
    fn render(
        &self,
        template: &linguasteg_core::RealizationTemplateDescriptor,
        plan: &linguasteg_core::RealizationPlan,
    ) -> CoreResult<String> {
        match self {
            Self::Farsi(realizer) => realizer.render(template, plan),
            Self::English(realizer) => realizer.render(template, plan),
        }
    }
}

pub(crate) enum MapperVariant {
    Farsi(FarsiPrototypeSymbolicMapper),
    English(EnglishPrototypeSymbolicMapper),
}

impl MapperVariant {
    pub(crate) fn frame_schemas(&self) -> Vec<SymbolicFrameSchema> {
        match self {
            Self::Farsi(mapper) => mapper.frame_schemas(),
            Self::English(mapper) => mapper.frame_schemas(),
        }
    }

    pub(crate) fn map_payload_to_plans(
        &self,
        payload_plan: &SymbolicPayloadPlan,
    ) -> CoreResult<Vec<RealizationPlan>> {
        match self {
            Self::Farsi(mapper) => mapper.map_payload_to_plans(payload_plan),
            Self::English(mapper) => mapper.map_payload_to_plans(payload_plan),
        }
    }
}
