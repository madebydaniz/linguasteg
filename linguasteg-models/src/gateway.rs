use linguasteg_core::{
    CoreError, CoreResult, GatewayFinishReason, GatewayMessageRole, GatewayOperation,
    GatewayRequest, GatewayResponse, GatewayUsage, ModelGateway, ModelGatewayRegistry, ProviderId,
};

#[derive(Debug, Clone)]
pub struct StubGateway {
    provider: ProviderId,
}

impl StubGateway {
    pub fn new(provider: ProviderId) -> Self {
        Self { provider }
    }
}

impl Default for StubGateway {
    fn default() -> Self {
        Self {
            provider: ProviderId::new("stub").expect("valid provider id"),
        }
    }
}

impl ModelGateway for StubGateway {
    fn provider(&self) -> &ProviderId {
        &self.provider
    }

    fn complete(&self, request: GatewayRequest) -> CoreResult<GatewayResponse> {
        if request.provider != self.provider {
            return Err(CoreError::UnsupportedModel {
                provider: request.provider.to_string(),
                model: request.model.to_string(),
            });
        }

        let last_user_message = request
            .messages
            .iter()
            .rev()
            .find(|message| message.role == GatewayMessageRole::User)
            .map(|message| message.content.as_str())
            .unwrap_or("");
        let operation = operation_name(request.operation);
        let content = format!(
            "stub:{operation}:{}:{}:{last_user_message}",
            request.language, request.strategy
        );
        let prompt_tokens = saturating_u32(
            request
                .messages
                .iter()
                .map(|message| message.content.len())
                .sum(),
        );
        let completion_tokens = saturating_u32(content.len());

        Ok(GatewayResponse {
            content,
            finish_reason: GatewayFinishReason::Stop,
            usage: Some(GatewayUsage {
                prompt_tokens,
                completion_tokens,
                total_tokens: prompt_tokens.saturating_add(completion_tokens),
            }),
        })
    }
}

#[derive(Default)]
pub struct InMemoryGatewayRegistry {
    gateways: Vec<Box<dyn ModelGateway>>,
}

impl InMemoryGatewayRegistry {
    pub fn new() -> Self {
        Self {
            gateways: Vec::new(),
        }
    }

    pub fn with_stub() -> Self {
        let mut registry = Self::new();
        registry.register(StubGateway::default());
        registry
    }

    pub fn register<G>(&mut self, gateway: G)
    where
        G: ModelGateway + 'static,
    {
        let provider = gateway.provider().clone();
        if let Some(index) = self
            .gateways
            .iter()
            .position(|existing| existing.provider() == &provider)
        {
            self.gateways[index] = Box::new(gateway);
        } else {
            self.gateways.push(Box::new(gateway));
        }
    }

    pub fn len(&self) -> usize {
        self.gateways.len()
    }

    pub fn is_empty(&self) -> bool {
        self.gateways.is_empty()
    }
}

impl ModelGatewayRegistry for InMemoryGatewayRegistry {
    fn gateway(&self, provider: &ProviderId) -> Option<&dyn ModelGateway> {
        self.gateways
            .iter()
            .find(|gateway| gateway.provider() == provider)
            .map(|gateway| gateway.as_ref())
    }
}

fn operation_name(operation: GatewayOperation) -> &'static str {
    match operation {
        GatewayOperation::Encode => "encode",
        GatewayOperation::Decode => "decode",
        GatewayOperation::Analyze => "analyze",
    }
}

fn saturating_u32(value: usize) -> u32 {
    if value > u32::MAX as usize {
        u32::MAX
    } else {
        value as u32
    }
}

#[cfg(test)]
mod tests {
    use linguasteg_core::{
        GatewayMessage, GatewayMessageRole, GatewayOperation, GatewayRequest, ModelGatewayRegistry,
        StrategyId,
    };

    use super::{InMemoryGatewayRegistry, StubGateway};
    use linguasteg_core::{LanguageTag, ModelGateway, ModelId, ProviderId};

    #[test]
    fn stub_gateway_returns_deterministic_response() {
        let gateway = StubGateway::default();
        let request = sample_request(
            ProviderId::new("stub").expect("valid provider"),
            ModelId::new("local-model").expect("valid model"),
            GatewayOperation::Encode,
            "salam",
        );

        let response = gateway
            .complete(request)
            .expect("stub completion should work");
        assert_eq!(
            response.finish_reason,
            linguasteg_core::GatewayFinishReason::Stop
        );
        assert!(response.content.contains("stub:encode:fa:symbolic:salam"));
        assert!(response.usage.is_some());
    }

    #[test]
    fn registry_routes_registered_stub_gateway() {
        let registry = InMemoryGatewayRegistry::with_stub();
        let provider = ProviderId::new("stub").expect("valid provider");
        let model = ModelId::new("local-model").expect("valid model");
        let gateway = registry
            .route(&provider, &model)
            .expect("registry should route stub provider");
        let response = gateway
            .complete(sample_request(
                provider,
                model,
                GatewayOperation::Decode,
                "trace",
            ))
            .expect("completion should work");

        assert!(response.content.contains("stub:decode"));
    }

    #[test]
    fn registry_replaces_gateway_for_same_provider() {
        let mut registry = InMemoryGatewayRegistry::new();
        let provider = ProviderId::new("stub").expect("valid provider");
        registry.register(StubGateway::new(provider.clone()));
        registry.register(StubGateway::new(provider));

        assert_eq!(registry.len(), 1);
        assert!(!registry.is_empty());
    }

    fn sample_request(
        provider: ProviderId,
        model: ModelId,
        operation: GatewayOperation,
        user_message: &str,
    ) -> GatewayRequest {
        GatewayRequest {
            provider,
            model,
            language: LanguageTag::new("fa").expect("valid language"),
            strategy: StrategyId::new("symbolic").expect("valid strategy"),
            operation,
            messages: vec![GatewayMessage {
                role: GatewayMessageRole::User,
                content: user_message.to_string(),
            }],
            seed: Some(7),
            max_tokens: Some(64),
        }
    }
}
