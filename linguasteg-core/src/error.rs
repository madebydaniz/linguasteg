pub type CoreResult<T> = Result<T, CoreError>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CoreError {
    InvalidIdentifier(String),
    UnsupportedLanguage(String),
    UnsupportedStrategy(String),
    NotImplemented(&'static str),
}

impl core::fmt::Display for CoreError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::InvalidIdentifier(value) => write!(f, "invalid identifier: {value}"),
            Self::UnsupportedLanguage(value) => write!(f, "language is not supported: {value}"),
            Self::UnsupportedStrategy(value) => write!(f, "strategy is not supported: {value}"),
            Self::NotImplemented(feature) => write!(f, "feature is not implemented yet: {feature}"),
        }
    }
}

impl std::error::Error for CoreError {}
