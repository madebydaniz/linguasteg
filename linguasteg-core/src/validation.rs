use crate::{
    CoreError, CoreResult, DecodeRequest, EncodeRequest, LanguageDescriptor, LanguageRegistry,
    ModelCapability, ModelDescriptor, ModelRegistry, PipelineOptions, StrategyDescriptor,
    StrategyRegistry,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidatedEncodeRequest {
    pub language: LanguageDescriptor,
    pub strategy: StrategyDescriptor,
    pub model: Option<ModelDescriptor>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidatedDecodeRequest {
    pub language: LanguageDescriptor,
    pub strategy: StrategyDescriptor,
    pub model: Option<ModelDescriptor>,
}

pub fn validate_encode_request(
    request: &EncodeRequest,
    language_registry: &dyn LanguageRegistry,
    strategy_registry: &dyn StrategyRegistry,
    model_registry: &dyn ModelRegistry,
) -> CoreResult<ValidatedEncodeRequest> {
    let validated = validate_pipeline_options(
        &request.options,
        language_registry,
        strategy_registry,
        model_registry,
    )?;

    Ok(ValidatedEncodeRequest {
        language: validated.language,
        strategy: validated.strategy,
        model: validated.model,
    })
}

pub fn validate_decode_request(
    request: &DecodeRequest,
    language_registry: &dyn LanguageRegistry,
    strategy_registry: &dyn StrategyRegistry,
    model_registry: &dyn ModelRegistry,
) -> CoreResult<ValidatedDecodeRequest> {
    let validated = validate_pipeline_options(
        &request.options,
        language_registry,
        strategy_registry,
        model_registry,
    )?;

    Ok(ValidatedDecodeRequest {
        language: validated.language,
        strategy: validated.strategy,
        model: validated.model,
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ValidatedPipelineOptions {
    language: LanguageDescriptor,
    strategy: StrategyDescriptor,
    model: Option<ModelDescriptor>,
}

fn validate_pipeline_options(
    options: &PipelineOptions,
    language_registry: &dyn LanguageRegistry,
    strategy_registry: &dyn StrategyRegistry,
    model_registry: &dyn ModelRegistry,
) -> CoreResult<ValidatedPipelineOptions> {
    let language = language_registry
        .language(&options.language)
        .cloned()
        .ok_or_else(|| CoreError::UnsupportedLanguage(options.language.to_string()))?;

    let strategy = strategy_registry
        .strategy(&options.strategy)
        .cloned()
        .ok_or_else(|| CoreError::UnsupportedStrategy(options.strategy.to_string()))?;

    let model = match &options.model_selection {
        Some(selection) => {
            let model = model_registry
                .model(&selection.provider, &selection.model)
                .cloned()
                .ok_or_else(|| CoreError::UnsupportedModel {
                    provider: selection.provider.to_string(),
                    model: selection.model.to_string(),
                })?;

            ensure_model_supports_language(&model, &options.language)?;
            ensure_required_capabilities(&model, &strategy)?;

            Some(model)
        }
        None => {
            if strategy.required_capabilities.is_empty() {
                None
            } else {
                return Err(CoreError::StrategyRequiresModel(strategy.id.to_string()));
            }
        }
    };

    Ok(ValidatedPipelineOptions {
        language,
        strategy,
        model,
    })
}

fn ensure_model_supports_language(
    model: &ModelDescriptor,
    language: &crate::LanguageTag,
) -> CoreResult<()> {
    if model.supported_languages.iter().any(|tag| tag == language) {
        Ok(())
    } else {
        Err(CoreError::ModelDoesNotSupportLanguage {
            provider: model.provider.to_string(),
            model: model.model.to_string(),
            language: language.to_string(),
        })
    }
}

fn ensure_required_capabilities(
    model: &ModelDescriptor,
    strategy: &StrategyDescriptor,
) -> CoreResult<()> {
    for capability in &strategy.required_capabilities {
        if !model.capabilities.contains(capability) {
            return Err(CoreError::ModelMissingCapability {
                provider: model.provider.to_string(),
                model: model.model.to_string(),
                capability: capability_name(*capability),
            });
        }
    }

    Ok(())
}

fn capability_name(capability: ModelCapability) -> &'static str {
    capability.as_str()
}

#[cfg(test)]
mod tests {
    use crate::{
        DecodeRequest, EncodeRequest, LanguageDescriptor, LanguageRegistry, LanguageTag,
        ModelCapability, ModelDescriptor, ModelId, ModelRegistry, ModelSelection, PipelineOptions,
        ProviderId, StrategyDescriptor, StrategyId, StrategyRegistry, TextDirection,
        validate_decode_request, validate_encode_request,
    };

    struct TestLanguageRegistry {
        values: Vec<LanguageDescriptor>,
    }

    impl LanguageRegistry for TestLanguageRegistry {
        fn all_languages(&self) -> &[LanguageDescriptor] {
            &self.values
        }
    }

    struct TestStrategyRegistry {
        values: Vec<StrategyDescriptor>,
    }

    impl StrategyRegistry for TestStrategyRegistry {
        fn all_strategies(&self) -> &[StrategyDescriptor] {
            &self.values
        }
    }

    struct TestModelRegistry {
        values: Vec<ModelDescriptor>,
    }

    impl ModelRegistry for TestModelRegistry {
        fn all_models(&self) -> &[ModelDescriptor] {
            &self.values
        }
    }

    #[test]
    fn validate_encode_request_accepts_strategy_without_model_requirement() {
        let request = EncodeRequest {
            carrier_text: "hello".to_string(),
            payload: vec![1, 2, 3],
            options: PipelineOptions {
                language: LanguageTag::new("en").expect("valid language"),
                strategy: StrategyId::new("synonym").expect("valid strategy"),
                model_selection: None,
            },
        };

        let languages = TestLanguageRegistry {
            values: vec![LanguageDescriptor {
                tag: LanguageTag::new("en").expect("valid"),
                display_name: "English".to_string(),
                direction: TextDirection::LeftToRight,
            }],
        };

        let strategies = TestStrategyRegistry {
            values: vec![StrategyDescriptor {
                id: StrategyId::new("synonym").expect("valid"),
                display_name: "Synonym".to_string(),
                required_capabilities: Vec::new(),
            }],
        };

        let models = TestModelRegistry { values: Vec::new() };

        let validated = validate_encode_request(&request, &languages, &strategies, &models)
            .expect("request should validate");
        assert!(validated.model.is_none());
    }

    #[test]
    fn validate_encode_request_rejects_missing_model_when_strategy_requires_capability() {
        let request = EncodeRequest {
            carrier_text: "hello".to_string(),
            payload: vec![1],
            options: PipelineOptions {
                language: LanguageTag::new("en").expect("valid language"),
                strategy: StrategyId::new("probabilistic").expect("valid strategy"),
                model_selection: None,
            },
        };

        let languages = TestLanguageRegistry {
            values: vec![LanguageDescriptor {
                tag: LanguageTag::new("en").expect("valid"),
                display_name: "English".to_string(),
                direction: TextDirection::LeftToRight,
            }],
        };

        let strategies = TestStrategyRegistry {
            values: vec![StrategyDescriptor {
                id: StrategyId::new("probabilistic").expect("valid"),
                display_name: "Probabilistic".to_string(),
                required_capabilities: vec![ModelCapability::TokenLogProbabilities],
            }],
        };

        let models = TestModelRegistry { values: Vec::new() };

        let error = validate_encode_request(&request, &languages, &strategies, &models)
            .expect_err("request should fail");
        let message = error.to_string();
        assert!(message.contains("strategy requires a model selection"));
    }

    #[test]
    fn validate_encode_request_rejects_model_without_required_capability() {
        let request = EncodeRequest {
            carrier_text: "hello".to_string(),
            payload: vec![1],
            options: PipelineOptions {
                language: LanguageTag::new("fa").expect("valid language"),
                strategy: StrategyId::new("probabilistic").expect("valid strategy"),
                model_selection: Some(ModelSelection {
                    provider: ProviderId::new("openai").expect("valid provider"),
                    model: ModelId::new("gpt-4o-mini").expect("valid model"),
                }),
            },
        };

        let languages = TestLanguageRegistry {
            values: vec![LanguageDescriptor {
                tag: LanguageTag::new("fa").expect("valid"),
                display_name: "Persian".to_string(),
                direction: TextDirection::RightToLeft,
            }],
        };

        let strategies = TestStrategyRegistry {
            values: vec![StrategyDescriptor {
                id: StrategyId::new("probabilistic").expect("valid"),
                display_name: "Probabilistic".to_string(),
                required_capabilities: vec![ModelCapability::TokenLogProbabilities],
            }],
        };

        let models = TestModelRegistry {
            values: vec![ModelDescriptor {
                provider: ProviderId::new("openai").expect("valid provider"),
                model: ModelId::new("gpt-4o-mini").expect("valid model"),
                display_name: "GPT-4o Mini".to_string(),
                supported_languages: vec![LanguageTag::new("fa").expect("valid")],
                capabilities: vec![ModelCapability::StreamingGeneration],
            }],
        };

        let error = validate_encode_request(&request, &languages, &strategies, &models)
            .expect_err("request should fail");
        let message = error.to_string();
        assert!(message.contains("missing required capability"));
    }

    #[test]
    fn validate_encode_request_accepts_supported_model_and_capabilities() {
        let request = EncodeRequest {
            carrier_text: "hello".to_string(),
            payload: vec![1],
            options: PipelineOptions {
                language: LanguageTag::new("fa").expect("valid language"),
                strategy: StrategyId::new("probabilistic").expect("valid strategy"),
                model_selection: Some(ModelSelection {
                    provider: ProviderId::new("openai").expect("valid provider"),
                    model: ModelId::new("gpt-4o-mini").expect("valid model"),
                }),
            },
        };

        let languages = TestLanguageRegistry {
            values: vec![LanguageDescriptor {
                tag: LanguageTag::new("fa").expect("valid"),
                display_name: "Persian".to_string(),
                direction: TextDirection::RightToLeft,
            }],
        };

        let strategies = TestStrategyRegistry {
            values: vec![StrategyDescriptor {
                id: StrategyId::new("probabilistic").expect("valid"),
                display_name: "Probabilistic".to_string(),
                required_capabilities: vec![ModelCapability::TokenLogProbabilities],
            }],
        };

        let models = TestModelRegistry {
            values: vec![ModelDescriptor {
                provider: ProviderId::new("openai").expect("valid provider"),
                model: ModelId::new("gpt-4o-mini").expect("valid model"),
                display_name: "GPT-4o Mini".to_string(),
                supported_languages: vec![LanguageTag::new("fa").expect("valid")],
                capabilities: vec![
                    ModelCapability::TokenLogProbabilities,
                    ModelCapability::StreamingGeneration,
                ],
            }],
        };

        let validated = validate_encode_request(&request, &languages, &strategies, &models)
            .expect("request should validate");

        let model = validated.model.expect("validated model should exist");
        assert_eq!(model.display_name, "GPT-4o Mini");
        assert_eq!(validated.language.display_name, "Persian");
    }

    #[test]
    fn validate_decode_request_rejects_model_with_unsupported_language() {
        let request = DecodeRequest {
            stego_text: "salam".to_string(),
            options: PipelineOptions {
                language: LanguageTag::new("fa").expect("valid language"),
                strategy: StrategyId::new("probabilistic").expect("valid strategy"),
                model_selection: Some(ModelSelection {
                    provider: ProviderId::new("openai").expect("valid provider"),
                    model: ModelId::new("gpt-4o-mini").expect("valid model"),
                }),
            },
        };

        let languages = TestLanguageRegistry {
            values: vec![LanguageDescriptor {
                tag: LanguageTag::new("fa").expect("valid"),
                display_name: "Persian".to_string(),
                direction: TextDirection::RightToLeft,
            }],
        };

        let strategies = TestStrategyRegistry {
            values: vec![StrategyDescriptor {
                id: StrategyId::new("probabilistic").expect("valid"),
                display_name: "Probabilistic".to_string(),
                required_capabilities: vec![ModelCapability::TokenLogProbabilities],
            }],
        };

        let models = TestModelRegistry {
            values: vec![ModelDescriptor {
                provider: ProviderId::new("openai").expect("valid provider"),
                model: ModelId::new("gpt-4o-mini").expect("valid model"),
                display_name: "GPT-4o Mini".to_string(),
                supported_languages: vec![LanguageTag::new("en").expect("valid")],
                capabilities: vec![ModelCapability::TokenLogProbabilities],
            }],
        };

        let error = validate_decode_request(&request, &languages, &strategies, &models)
            .expect_err("request should fail");
        assert!(error.to_string().contains("does not support language"));
    }

    #[test]
    fn validate_decode_request_accepts_supported_configuration() {
        let request = DecodeRequest {
            stego_text: "salam".to_string(),
            options: PipelineOptions {
                language: LanguageTag::new("fa").expect("valid language"),
                strategy: StrategyId::new("probabilistic").expect("valid strategy"),
                model_selection: Some(ModelSelection {
                    provider: ProviderId::new("openai").expect("valid provider"),
                    model: ModelId::new("gpt-4o-mini").expect("valid model"),
                }),
            },
        };

        let languages = TestLanguageRegistry {
            values: vec![LanguageDescriptor {
                tag: LanguageTag::new("fa").expect("valid"),
                display_name: "Persian".to_string(),
                direction: TextDirection::RightToLeft,
            }],
        };

        let strategies = TestStrategyRegistry {
            values: vec![StrategyDescriptor {
                id: StrategyId::new("probabilistic").expect("valid"),
                display_name: "Probabilistic".to_string(),
                required_capabilities: vec![ModelCapability::TokenLogProbabilities],
            }],
        };

        let models = TestModelRegistry {
            values: vec![ModelDescriptor {
                provider: ProviderId::new("openai").expect("valid provider"),
                model: ModelId::new("gpt-4o-mini").expect("valid model"),
                display_name: "GPT-4o Mini".to_string(),
                supported_languages: vec![LanguageTag::new("fa").expect("valid")],
                capabilities: vec![ModelCapability::TokenLogProbabilities],
            }],
        };

        let validated = validate_decode_request(&request, &languages, &strategies, &models)
            .expect("request should validate");
        assert_eq!(validated.language.display_name, "Persian");
        assert_eq!(validated.strategy.display_name, "Probabilistic");
        assert!(validated.model.is_some());
    }
}
