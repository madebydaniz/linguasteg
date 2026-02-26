use crate::{
    LanguageDescriptor, LanguageTag, ModelDescriptor, ModelId, ProviderId, StrategyDescriptor,
    StrategyId, StyleProfileDescriptor, StyleProfileId,
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

pub trait StyleProfileRegistry: Send + Sync {
    fn all_style_profiles(&self) -> &[StyleProfileDescriptor];

    fn style_profile(&self, id: &StyleProfileId) -> Option<&StyleProfileDescriptor> {
        self.all_style_profiles()
            .iter()
            .find(|descriptor| &descriptor.id == id)
    }

    fn style_profiles_for_language(&self, language: &LanguageTag) -> Vec<&StyleProfileDescriptor> {
        self.all_style_profiles()
            .iter()
            .filter(|descriptor| &descriptor.language == language)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        LanguageDescriptor, LanguageRegistry, LanguageTag, ModelCapability, StrategyDescriptor,
        StrategyId, StrategyRegistry, StyleInspiration, StyleProfileDescriptor, StyleProfileId,
        StyleProfileRegistry, StyleStrength, TextDirection, WritingRegister,
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

    struct InMemoryStyleProfileRegistry {
        profiles: Vec<StyleProfileDescriptor>,
    }

    impl StyleProfileRegistry for InMemoryStyleProfileRegistry {
        fn all_style_profiles(&self) -> &[StyleProfileDescriptor] {
            &self.profiles
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

    #[test]
    fn style_profile_registry_supports_lookup_and_language_filter() {
        let registry = InMemoryStyleProfileRegistry {
            profiles: vec![
                StyleProfileDescriptor {
                    id: StyleProfileId::new("fa-formal").expect("valid style id"),
                    language: LanguageTag::new("fa").expect("valid tag"),
                    display_name: "Formal Persian".to_string(),
                    register: WritingRegister::Formal,
                    strength: StyleStrength::Medium,
                    inspiration: StyleInspiration::RegisterOnly,
                },
                StyleProfileDescriptor {
                    id: StyleProfileId::new("de-goethe-classic").expect("valid style id"),
                    language: LanguageTag::new("de").expect("valid tag"),
                    display_name: "Goethe-inspired German".to_string(),
                    register: WritingRegister::Literary,
                    strength: StyleStrength::Light,
                    inspiration: StyleInspiration::PublicDomainAuthorInspired {
                        author_label: "Goethe".to_string(),
                    },
                },
            ],
        };

        let style_id = StyleProfileId::new("de-goethe-classic").expect("valid style id");
        let profile = registry
            .style_profile(&style_id)
            .expect("style profile should exist");
        assert_eq!(profile.display_name, "Goethe-inspired German");

        let fa = LanguageTag::new("fa").expect("valid tag");
        let fa_profiles = registry.style_profiles_for_language(&fa);
        assert_eq!(fa_profiles.len(), 1);
        assert_eq!(fa_profiles[0].display_name, "Formal Persian");
    }
}
