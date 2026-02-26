use crate::{
    LanguageDescriptor, LanguageTag, ModelDescriptor, ModelId, ProviderId, StrategyDescriptor,
    StrategyId,
};

pub trait LanguageRegistry: Send + Sync {
    fn all_languages(&self) -> &[LanguageDescriptor];

    fn language(&self, tag: &LanguageTag) -> Option<&LanguageDescriptor> {
        self.all_languages().iter().find(|descriptor| &descriptor.tag == tag)
    }
}

pub trait StrategyRegistry: Send + Sync {
    fn all_strategies(&self) -> &[StrategyDescriptor];

    fn strategy(&self, id: &StrategyId) -> Option<&StrategyDescriptor> {
        self.all_strategies()
            .iter()
            .find(|descriptor| &descriptor.id == id)
    }
}

pub trait ModelRegistry: Send + Sync {
    fn all_models(&self) -> &[ModelDescriptor];

    fn model(&self, provider: &ProviderId, model: &ModelId) -> Option<&ModelDescriptor> {
        self.all_models()
            .iter()
            .find(|descriptor| &descriptor.provider == provider && &descriptor.model == model)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        LanguageDescriptor, LanguageRegistry, LanguageTag, ModelCapability, StrategyDescriptor,
        StrategyId, StrategyRegistry, TextDirection,
    };

    struct InMemoryLanguageRegistry {
        languages: Vec<LanguageDescriptor>,
    }

    impl LanguageRegistry for InMemoryLanguageRegistry {
        fn all_languages(&self) -> &[LanguageDescriptor] {
            &self.languages
        }
    }

    struct InMemoryStrategyRegistry {
        strategies: Vec<StrategyDescriptor>,
    }

    impl StrategyRegistry for InMemoryStrategyRegistry {
        fn all_strategies(&self) -> &[StrategyDescriptor] {
            &self.strategies
        }
    }

    #[test]
    fn language_registry_supports_lookup_by_tag() {
        let registry = InMemoryLanguageRegistry {
            languages: vec![LanguageDescriptor {
                tag: LanguageTag::new("fa").expect("valid tag"),
                display_name: "Persian".to_string(),
                direction: TextDirection::RightToLeft,
            }],
        };

        let tag = LanguageTag::new("fa").expect("valid tag");
        let descriptor = registry.language(&tag).expect("language should exist");
        assert_eq!(descriptor.display_name, "Persian");
    }

    #[test]
    fn strategy_registry_supports_lookup_by_id() {
        let registry = InMemoryStrategyRegistry {
            strategies: vec![StrategyDescriptor {
                id: StrategyId::new("synonym").expect("valid strategy"),
                display_name: "Synonym".to_string(),
                required_capabilities: vec![ModelCapability::TokenLogProbabilities],
            }],
        };

        let id = StrategyId::new("synonym").expect("valid strategy");
        let descriptor = registry.strategy(&id).expect("strategy should exist");
        assert_eq!(descriptor.display_name, "Synonym");
    }
}
