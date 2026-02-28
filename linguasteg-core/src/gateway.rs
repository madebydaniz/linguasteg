use crate::{CoreError, CoreResult, LanguageTag, ModelId, ProviderId, StrategyId};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GatewayOperation {
    Encode,
    Decode,
    Analyze,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GatewayMessageRole {
    System,
    User,
    Assistant,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GatewayMessage {
    pub role: GatewayMessageRole,
    pub content: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GatewayRequest {
    pub provider: ProviderId,
    pub model: ModelId,
    pub language: LanguageTag,
    pub strategy: StrategyId,
    pub operation: GatewayOperation,
    pub messages: Vec<GatewayMessage>,
    pub seed: Option<u64>,
    pub max_tokens: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GatewayFinishReason {
    Stop,
    Length,
    Unknown(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GatewayUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GatewayResponse {
    pub content: String,
    pub finish_reason: GatewayFinishReason,
    pub usage: Option<GatewayUsage>,
}

pub trait ModelGateway: Send + Sync {
    fn provider(&self) -> &ProviderId;
    fn complete(&self, request: GatewayRequest) -> CoreResult<GatewayResponse>;
}

pub trait ModelGatewayRegistry: Send + Sync {
    fn gateway(&self, provider: &ProviderId) -> Option<&dyn ModelGateway>;

    fn route(&self, provider: &ProviderId, model: &ModelId) -> CoreResult<&dyn ModelGateway> {
        self.gateway(provider)
            .ok_or_else(|| CoreError::UnsupportedModel {
                provider: provider.to_string(),
                model: model.to_string(),
            })
    }
}

#[cfg(test)]
mod tests {
    use crate::{LanguageTag, ModelId, ProviderId, StrategyId};

    use super::{
        GatewayFinishReason, GatewayMessage, GatewayMessageRole, GatewayOperation, GatewayRequest,
        GatewayResponse, ModelGateway, ModelGatewayRegistry,
    };

    struct TestGateway {
        provider: ProviderId,
    }

    impl ModelGateway for TestGateway {
        fn provider(&self) -> &ProviderId {
            &self.provider
        }

        fn complete(&self, request: GatewayRequest) -> crate::CoreResult<GatewayResponse> {
            Ok(GatewayResponse {
                content: format!(
                    "{}:{}",
                    request.operation_name_for_test(),
                    request.messages.len()
                ),
                finish_reason: GatewayFinishReason::Stop,
                usage: None,
            })
        }
    }

    struct TestRegistry {
        gateways: Vec<TestGateway>,
    }

    impl ModelGatewayRegistry for TestRegistry {
        fn gateway(&self, provider: &ProviderId) -> Option<&dyn ModelGateway> {
            self.gateways
                .iter()
                .find(|gateway| gateway.provider() == provider)
                .map(|gateway| gateway as &dyn ModelGateway)
        }
    }

    impl GatewayRequest {
        fn operation_name_for_test(&self) -> &'static str {
            match self.operation {
                GatewayOperation::Encode => "encode",
                GatewayOperation::Decode => "decode",
                GatewayOperation::Analyze => "analyze",
            }
        }
    }

    #[test]
    fn registry_routes_to_registered_provider() {
        let provider = ProviderId::new("stub").expect("valid provider id");
        let registry = TestRegistry {
            gateways: vec![TestGateway {
                provider: provider.clone(),
            }],
        };
        let model = ModelId::new("model-a").expect("valid model id");

        let gateway = registry
            .route(&provider, &model)
            .expect("gateway should be found");
        let response = gateway
            .complete(sample_request(provider, model, GatewayOperation::Encode))
            .expect("completion should succeed");

        assert_eq!(response.content, "encode:1");
    }

    #[test]
    fn registry_rejects_unknown_provider() {
        let registry = TestRegistry {
            gateways: Vec::new(),
        };
        let provider = ProviderId::new("missing").expect("valid provider id");
        let model = ModelId::new("model-a").expect("valid model id");

        let result = registry.route(&provider, &model);
        match result {
            Ok(_) => panic!("route should fail"),
            Err(error) => assert!(error.to_string().contains("model is not supported")),
        }
    }

    fn sample_request(
        provider: ProviderId,
        model: ModelId,
        operation: GatewayOperation,
    ) -> GatewayRequest {
        GatewayRequest {
            provider,
            model,
            language: LanguageTag::new("fa").expect("valid language tag"),
            strategy: StrategyId::new("symbolic").expect("valid strategy"),
            operation,
            messages: vec![GatewayMessage {
                role: GatewayMessageRole::User,
                content: "sample".to_string(),
            }],
            seed: Some(7),
            max_tokens: Some(32),
        }
    }
}
