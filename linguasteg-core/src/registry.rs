use crate::{
    LanguageDescriptor, LanguageTag, ModelDescriptor, ModelId, ProviderId, StrategyDescriptor,
    StrategyId, StyleProfileDescriptor, StyleProfileId, RealizationTemplateDescriptor, TemplateId,
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

pub trait TemplateRegistry: Send + Sync {
    fn all_templates(&self) -> &[RealizationTemplateDescriptor];

    fn template(&self, id: &TemplateId) -> Option<&RealizationTemplateDescriptor> {
        self.all_templates()
            .iter()
            .find(|descriptor| &descriptor.id == id)
    }

    fn templates_for_language(&self, language: &LanguageTag) -> Vec<&RealizationTemplateDescriptor> {
        self.all_templates()
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
        StyleProfileRegistry, StyleStrength, TextDirection, WritingRegister, RealizationTemplateDescriptor,
        SlotId, SlotRole, TemplateId, TemplateRegistry, TemplateSlotDescriptor, TemplateToken,
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

    struct InMemoryTemplateRegistry {
        templates: Vec<RealizationTemplateDescriptor>,
    }

    impl TemplateRegistry for InMemoryTemplateRegistry {
        fn all_templates(&self) -> &[RealizationTemplateDescriptor] {
            &self.templates
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

    #[test]
    fn template_registry_supports_lookup_and_language_filter() {
        let registry = InMemoryTemplateRegistry {
            templates: vec![
                RealizationTemplateDescriptor {
                    id: TemplateId::new("fa-simple").expect("valid template id"),
                    language: LanguageTag::new("fa").expect("valid tag"),
                    display_name: "Simple Persian Template".to_string(),
                    slots: vec![
                        TemplateSlotDescriptor {
                            id: SlotId::new("subject").expect("valid slot"),
                            role: SlotRole::Subject,
                            required: true,
                        },
                        TemplateSlotDescriptor {
                            id: SlotId::new("verb").expect("valid slot"),
                            role: SlotRole::Verb,
                            required: true,
                        },
                    ],
                    tokens: vec![
                        TemplateToken::Slot(SlotId::new("subject").expect("valid slot")),
                        TemplateToken::Slot(SlotId::new("verb").expect("valid slot")),
                    ],
                },
                RealizationTemplateDescriptor {
                    id: TemplateId::new("de-simple").expect("valid template id"),
                    language: LanguageTag::new("de").expect("valid tag"),
                    display_name: "Simple German Template".to_string(),
                    slots: vec![],
                    tokens: vec![],
                },
            ],
        };

        let template_id = TemplateId::new("fa-simple").expect("valid template id");
        let template = registry.template(&template_id).expect("template should exist");
        assert_eq!(template.display_name, "Simple Persian Template");

        let fa = LanguageTag::new("fa").expect("valid tag");
        let fa_templates = registry.templates_for_language(&fa);
        assert_eq!(fa_templates.len(), 1);
        assert_eq!(fa_templates[0].id.as_str(), "fa-simple");
    }
}
