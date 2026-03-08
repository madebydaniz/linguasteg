use crate::{
    CoreResult, DecodeRequest, FixedWidthPlanningOptions, GatewayMessage, GatewayMessageRole,
    GatewayOperation, GatewayRequest, GatewayResponse, LanguageRegistry, ModelGatewayRegistry,
    ModelRegistry, StrategyRegistry, SymbolicFramePlan, SymbolicFrameSchema, SymbolicPayloadPlan,
    SymbolicPayloadPlanner, ValidatedDecodeRequest, ValidatedEncodeRequest,
    decode_payload_from_symbolic_frames, validate_decode_request, validate_encode_request,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OrchestratedEncodeResult {
    pub validated: ValidatedEncodeRequest,
    pub symbolic_plan: SymbolicPayloadPlan,
    pub gateway_response: Option<GatewayResponse>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OrchestratedDecodeResult {
    pub validated: ValidatedDecodeRequest,
    pub payload: Vec<u8>,
    pub gateway_response: Option<GatewayResponse>,
}

pub struct PipelineOrchestrator<'a> {
    language_registry: &'a dyn LanguageRegistry,
    strategy_registry: &'a dyn StrategyRegistry,
    model_registry: &'a dyn ModelRegistry,
    gateway_registry: &'a dyn ModelGatewayRegistry,
    symbolic_planner: &'a dyn SymbolicPayloadPlanner,
    symbolic_options: FixedWidthPlanningOptions,
}

impl<'a> PipelineOrchestrator<'a> {
    pub fn new(
        language_registry: &'a dyn LanguageRegistry,
        strategy_registry: &'a dyn StrategyRegistry,
        model_registry: &'a dyn ModelRegistry,
        gateway_registry: &'a dyn ModelGatewayRegistry,
        symbolic_planner: &'a dyn SymbolicPayloadPlanner,
    ) -> Self {
        Self {
            language_registry,
            strategy_registry,
            model_registry,
            gateway_registry,
            symbolic_planner,
            symbolic_options: FixedWidthPlanningOptions::default(),
        }
    }

    pub fn with_symbolic_options(mut self, options: FixedWidthPlanningOptions) -> Self {
        self.symbolic_options = options;
        self
    }

    pub fn orchestrate_encode(
        &self,
        request: crate::EncodeRequest,
        schemas: &[SymbolicFrameSchema],
    ) -> CoreResult<OrchestratedEncodeResult> {
        let validated = validate_encode_request(
            &request,
            self.language_registry,
            self.strategy_registry,
            self.model_registry,
        )?;
        let symbolic_plan = self
            .symbolic_planner
            .plan_payload(&request.payload, schemas)?;
        let gateway_response = self
            .build_encode_gateway_request(&validated, &request)
            .map(|gateway_request| {
                let gateway = self
                    .gateway_registry
                    .route(&gateway_request.provider, &gateway_request.model)?;
                gateway.complete(gateway_request)
            })
            .transpose()?;

        Ok(OrchestratedEncodeResult {
            validated,
            symbolic_plan,
            gateway_response,
        })
    }

    pub fn orchestrate_decode(
        &self,
        request: DecodeRequest,
        frames: &[SymbolicFramePlan],
        schemas: &[SymbolicFrameSchema],
    ) -> CoreResult<OrchestratedDecodeResult> {
        let validated = validate_decode_request(
            &request,
            self.language_registry,
            self.strategy_registry,
            self.model_registry,
        )?;
        let payload = decode_payload_from_symbolic_frames(frames, schemas, &self.symbolic_options)?;
        let gateway_response = self
            .build_decode_gateway_request(&validated, &request)
            .map(|gateway_request| {
                let gateway = self
                    .gateway_registry
                    .route(&gateway_request.provider, &gateway_request.model)?;
                gateway.complete(gateway_request)
            })
            .transpose()?;

        Ok(OrchestratedDecodeResult {
            validated,
            payload,
            gateway_response,
        })
    }

    fn build_encode_gateway_request(
        &self,
        validated: &ValidatedEncodeRequest,
        request: &crate::EncodeRequest,
    ) -> Option<GatewayRequest> {
        validated.model.as_ref().map(|model| GatewayRequest {
            provider: model.provider.clone(),
            model: model.model.clone(),
            language: validated.language.tag.clone(),
            strategy: validated.strategy.id.clone(),
            operation: GatewayOperation::Encode,
            messages: vec![GatewayMessage {
                role: GatewayMessageRole::User,
                content: request.carrier_text.clone(),
            }],
            seed: None,
            max_tokens: None,
        })
    }

    fn build_decode_gateway_request(
        &self,
        validated: &ValidatedDecodeRequest,
        request: &DecodeRequest,
    ) -> Option<GatewayRequest> {
        validated.model.as_ref().map(|model| GatewayRequest {
            provider: model.provider.clone(),
            model: model.model.clone(),
            language: validated.language.tag.clone(),
            strategy: validated.strategy.id.clone(),
            operation: GatewayOperation::Decode,
            messages: vec![GatewayMessage {
                role: GatewayMessageRole::User,
                content: request.stego_text.clone(),
            }],
            seed: None,
            max_tokens: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        DecodeRequest, EncodeRequest, FixedWidthBitPlanner, GatewayFinishReason, GatewayOperation,
        GatewayRequest, GatewayResponse, LanguageDescriptor, LanguageRegistry, LanguageTag,
        ModelCapability, ModelDescriptor, ModelGateway, ModelGatewayRegistry, ModelId,
        ModelRegistry, ModelSelection, PipelineOptions, ProviderId, StrategyDescriptor, StrategyId,
        StrategyRegistry, SymbolicFieldSpec, SymbolicFrameSchema, TextDirection,
    };

    use super::PipelineOrchestrator;
    use crate::SymbolicPayloadPlanner;

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

    struct TestGateway {
        provider: ProviderId,
    }

    impl ModelGateway for TestGateway {
        fn provider(&self) -> &ProviderId {
            &self.provider
        }

        fn complete(&self, request: GatewayRequest) -> crate::CoreResult<GatewayResponse> {
            let content = format!(
                "{}:{}:{}",
                operation_name(request.operation),
                request.language,
                request.messages.len()
            );
            Ok(GatewayResponse {
                content,
                finish_reason: GatewayFinishReason::Stop,
                usage: None,
            })
        }
    }

    struct TestGatewayRegistry {
        gateways: Vec<TestGateway>,
    }

    impl ModelGatewayRegistry for TestGatewayRegistry {
        fn gateway(&self, provider: &ProviderId) -> Option<&dyn ModelGateway> {
            self.gateways
                .iter()
                .find(|gateway| gateway.provider == *provider)
                .map(|gateway| gateway as &dyn ModelGateway)
        }
    }

    #[test]
    fn orchestrate_encode_validates_routes_gateway_and_plans_symbolic_frames() {
        let planner = FixedWidthBitPlanner::default();
        let languages = language_registry();
        let strategies = strategy_registry_requires_model();
        let models = model_registry("stub");
        let gateways = gateway_registry_with("stub");
        let orchestrator =
            PipelineOrchestrator::new(&languages, &strategies, &models, &gateways, &planner);
        let request = EncodeRequest {
            carrier_text: "carrier text".to_string(),
            payload: vec![0xAA, 0xBB],
            options: PipelineOptions {
                language: LanguageTag::new("fa").expect("valid language"),
                strategy: StrategyId::new("symbolic").expect("valid strategy"),
                model_selection: Some(ModelSelection {
                    provider: ProviderId::new("stub").expect("valid provider"),
                    model: ModelId::new("test-model").expect("valid model"),
                }),
            },
        };
        let schemas = sample_schemas();

        let result = orchestrator
            .orchestrate_encode(request, &schemas)
            .expect("encode orchestration should succeed");

        assert!(!result.symbolic_plan.frames.is_empty());
        assert!(result.gateway_response.is_some());
        assert!(
            result
                .gateway_response
                .expect("gateway response should exist")
                .content
                .contains("encode:fa:1")
        );
    }

    #[test]
    fn orchestrate_decode_reconstructs_payload_without_gateway_when_model_is_absent() {
        let planner = FixedWidthBitPlanner::default();
        let payload = vec![0x10, 0x20, 0x30];
        let schemas = sample_schemas();
        let symbolic_plan = planner
            .plan_payload(&payload, &schemas)
            .expect("planning should succeed");
        let languages = language_registry();
        let strategies = strategy_registry_without_model_requirement();
        let models = model_registry("stub");
        let gateways = gateway_registry_with("stub");
        let orchestrator =
            PipelineOrchestrator::new(&languages, &strategies, &models, &gateways, &planner);
        let request = DecodeRequest {
            stego_text: "stego sample".to_string(),
            options: PipelineOptions {
                language: LanguageTag::new("fa").expect("valid language"),
                strategy: StrategyId::new("symbolic-lite").expect("valid strategy"),
                model_selection: None,
            },
        };

        let result = orchestrator
            .orchestrate_decode(request, &symbolic_plan.frames, &schemas)
            .expect("decode orchestration should succeed");

        assert_eq!(result.payload, payload);
        assert!(result.gateway_response.is_none());
    }

    #[test]
    fn orchestrate_encode_fails_when_provider_cannot_be_routed() {
        let planner = FixedWidthBitPlanner::default();
        let languages = language_registry();
        let strategies = strategy_registry_requires_model();
        let models = model_registry("unknown-provider");
        let gateways = gateway_registry_with("stub");
        let orchestrator =
            PipelineOrchestrator::new(&languages, &strategies, &models, &gateways, &planner);
        let request = EncodeRequest {
            carrier_text: "carrier text".to_string(),
            payload: vec![0xAA],
            options: PipelineOptions {
                language: LanguageTag::new("fa").expect("valid language"),
                strategy: StrategyId::new("symbolic").expect("valid strategy"),
                model_selection: Some(ModelSelection {
                    provider: ProviderId::new("unknown-provider").expect("valid provider"),
                    model: ModelId::new("test-model").expect("valid model"),
                }),
            },
        };

        let error = orchestrator
            .orchestrate_encode(request, &sample_schemas())
            .expect_err("orchestration should fail");
        assert!(error.to_string().contains("model is not supported"));
    }

    fn operation_name(operation: GatewayOperation) -> &'static str {
        match operation {
            GatewayOperation::Encode => "encode",
            GatewayOperation::Decode => "decode",
            GatewayOperation::Analyze => "analyze",
        }
    }

    fn language_registry() -> TestLanguageRegistry {
        TestLanguageRegistry {
            values: vec![LanguageDescriptor {
                tag: LanguageTag::new("fa").expect("valid language"),
                display_name: "Persian".to_string(),
                direction: TextDirection::RightToLeft,
            }],
        }
    }

    fn strategy_registry_requires_model() -> TestStrategyRegistry {
        TestStrategyRegistry {
            values: vec![StrategyDescriptor {
                id: StrategyId::new("symbolic").expect("valid strategy"),
                display_name: "Symbolic".to_string(),
                required_capabilities: vec![ModelCapability::DeterministicSeed],
            }],
        }
    }

    fn strategy_registry_without_model_requirement() -> TestStrategyRegistry {
        TestStrategyRegistry {
            values: vec![StrategyDescriptor {
                id: StrategyId::new("symbolic-lite").expect("valid strategy"),
                display_name: "Symbolic Lite".to_string(),
                required_capabilities: Vec::new(),
            }],
        }
    }

    fn model_registry(provider: &str) -> TestModelRegistry {
        TestModelRegistry {
            values: vec![ModelDescriptor {
                provider: ProviderId::new(provider).expect("valid provider"),
                model: ModelId::new("test-model").expect("valid model"),
                display_name: "Test Model".to_string(),
                supported_languages: vec![LanguageTag::new("fa").expect("valid language")],
                capabilities: vec![ModelCapability::DeterministicSeed],
            }],
        }
    }

    fn gateway_registry_with(provider: &str) -> TestGatewayRegistry {
        TestGatewayRegistry {
            gateways: vec![TestGateway {
                provider: ProviderId::new(provider).expect("valid provider"),
            }],
        }
    }

    fn sample_schemas() -> Vec<SymbolicFrameSchema> {
        vec![SymbolicFrameSchema {
            template_id: crate::TemplateId::new("fa-test").expect("valid template"),
            fields: vec![SymbolicFieldSpec {
                slot: crate::SlotId::new("payload").expect("valid slot"),
                bit_width: 8,
            }],
        }]
    }
}
