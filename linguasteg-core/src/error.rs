pub type CoreResult<T> = Result<T, CoreError>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CoreError {
    InvalidIdentifier(String),
    UnsupportedLanguage(String),
    UnsupportedStrategy(String),
    UnsupportedModel {
        provider: String,
        model: String,
    },
    UnsupportedTemplate(String),
    StrategyRequiresModel(String),
    InvalidTemplate(String),
    InvalidSymbolicSchema(String),
    InvalidSymbolicPlan(String),
    UnknownTemplateSlot(String),
    DuplicateSlotAssignment(String),
    MissingRequiredSlot(String),
    ModelDoesNotSupportLanguage {
        provider: String,
        model: String,
        language: String,
    },
    ModelMissingCapability {
        provider: String,
        model: String,
        capability: &'static str,
    },
    NotImplemented(&'static str),
}

impl core::fmt::Display for CoreError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::InvalidIdentifier(value) => write!(f, "invalid identifier: {value}"),
            Self::UnsupportedLanguage(value) => write!(f, "language is not supported: {value}"),
            Self::UnsupportedStrategy(value) => write!(f, "strategy is not supported: {value}"),
            Self::UnsupportedModel { provider, model } => {
                write!(f, "model is not supported: {provider}/{model}")
            }
            Self::UnsupportedTemplate(template) => {
                write!(f, "template is not supported: {template}")
            }
            Self::StrategyRequiresModel(strategy) => {
                write!(f, "strategy requires a model selection: {strategy}")
            }
            Self::InvalidTemplate(message) => write!(f, "invalid template: {message}"),
            Self::InvalidSymbolicSchema(message) => write!(f, "invalid symbolic schema: {message}"),
            Self::InvalidSymbolicPlan(message) => write!(f, "invalid symbolic plan: {message}"),
            Self::UnknownTemplateSlot(slot) => write!(f, "unknown template slot: {slot}"),
            Self::DuplicateSlotAssignment(slot) => {
                write!(f, "duplicate slot assignment in realization plan: {slot}")
            }
            Self::MissingRequiredSlot(slot) => {
                write!(f, "missing required slot in realization plan: {slot}")
            }
            Self::ModelDoesNotSupportLanguage {
                provider,
                model,
                language,
            } => write!(
                f,
                "model does not support language: {provider}/{model} for {language}"
            ),
            Self::ModelMissingCapability {
                provider,
                model,
                capability,
            } => write!(
                f,
                "model is missing required capability: {provider}/{model} -> {capability}"
            ),
            Self::NotImplemented(feature) => write!(f, "feature is not implemented yet: {feature}"),
        }
    }
}

impl std::error::Error for CoreError {}
