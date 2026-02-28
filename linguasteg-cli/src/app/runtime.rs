use linguasteg_core::{
    FixedWidthBitPlanner, LanguageTag, ModelCapability, ModelDescriptor, ModelId, ModelRegistry,
    ModelSelection, PipelineOptions, PipelineOrchestrator, ProviderId, StrategyDescriptor,
    StrategyId, StrategyRegistry,
};
use linguasteg_models::{
    FarsiPrototypeConstraintChecker, FarsiPrototypeLanguagePack, FarsiPrototypeRealizer,
    FarsiPrototypeSymbolicMapper, InMemoryGatewayRegistry,
};

use super::types::DynError;

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

pub(crate) struct FarsiProtoRuntime {
    pub(crate) pack: FarsiPrototypeLanguagePack,
    pub(crate) checker: FarsiPrototypeConstraintChecker,
    pub(crate) realizer: FarsiPrototypeRealizer,
    pub(crate) mapper: FarsiPrototypeSymbolicMapper,
    planner: FixedWidthBitPlanner,
    strategy_registry: InMemoryStrategyRegistry,
    model_registry: InMemoryModelRegistry,
    gateway_registry: InMemoryGatewayRegistry,
}

impl FarsiProtoRuntime {
    pub(crate) fn new() -> Result<Self, DynError> {
        let strategy_id = StrategyId::new("symbolic-stub")?;
        let provider = ProviderId::new("stub")?;
        let model = ModelId::new("stub-local")?;
        let fa = LanguageTag::new("fa")?;

        Ok(Self {
            pack: FarsiPrototypeLanguagePack::default(),
            checker: FarsiPrototypeConstraintChecker,
            realizer: FarsiPrototypeRealizer,
            mapper: FarsiPrototypeSymbolicMapper,
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
                    supported_languages: vec![fa],
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

    pub(crate) fn pipeline_options(&self) -> Result<PipelineOptions, DynError> {
        Ok(PipelineOptions {
            language: LanguageTag::new("fa")?,
            strategy: StrategyId::new("symbolic-stub")?,
            model_selection: Some(ModelSelection {
                provider: ProviderId::new("stub")?,
                model: ModelId::new("stub-local")?,
            }),
        })
    }
}
