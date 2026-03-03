use std::fmt::{Display, Formatter};

pub(crate) enum Command {
    Encode(EncodeOptions),
    Decode(DecodeOptions),
    Analyze(AnalyzeOptions),
    Validate(ValidateOptions),
    Languages(OutputFormat),
    Strategies(OutputFormat),
    Models(OutputFormat),
    Catalog(CatalogQueryOptions),
    Templates(TemplateQueryOptions),
    Profiles(ProfileQueryOptions),
    Schemas(SchemaQueryOptions),
    Demo(DemoTarget),
    ProtoEncode(ProtoTarget, String, bool),
    ProtoDecode(ProtoTarget, Option<String>, bool),
}

pub(crate) struct EncodeOptions {
    pub(crate) target: ProtoTarget,
    pub(crate) message: Option<String>,
    pub(crate) input_path: Option<String>,
    pub(crate) output_path: Option<String>,
    pub(crate) secret: Option<String>,
    pub(crate) secret_file: Option<String>,
    pub(crate) format: OutputFormat,
}

pub(crate) struct DecodeOptions {
    pub(crate) target: ProtoTarget,
    pub(crate) auto_detect_target: bool,
    pub(crate) trace: Option<String>,
    pub(crate) input_path: Option<String>,
    pub(crate) output_path: Option<String>,
    pub(crate) secret: Option<String>,
    pub(crate) secret_file: Option<String>,
    pub(crate) format: OutputFormat,
}

pub(crate) struct AnalyzeOptions {
    pub(crate) target: ProtoTarget,
    pub(crate) auto_detect_target: bool,
    pub(crate) trace: Option<String>,
    pub(crate) input_path: Option<String>,
    pub(crate) output_path: Option<String>,
    pub(crate) secret: Option<String>,
    pub(crate) secret_file: Option<String>,
    pub(crate) format: OutputFormat,
}

pub(crate) struct ValidateOptions {
    pub(crate) target: ProtoTarget,
    pub(crate) auto_detect_target: bool,
    pub(crate) trace: Option<String>,
    pub(crate) input_path: Option<String>,
    pub(crate) output_path: Option<String>,
    pub(crate) secret: Option<String>,
    pub(crate) secret_file: Option<String>,
    pub(crate) format: OutputFormat,
}

pub(crate) struct TemplateQueryOptions {
    pub(crate) format: OutputFormat,
    pub(crate) target: Option<ProtoTarget>,
}

pub(crate) struct CatalogQueryOptions {
    pub(crate) format: OutputFormat,
    pub(crate) target: Option<ProtoTarget>,
}

pub(crate) struct ProfileQueryOptions {
    pub(crate) format: OutputFormat,
    pub(crate) target: Option<ProtoTarget>,
}

pub(crate) struct SchemaQueryOptions {
    pub(crate) format: OutputFormat,
    pub(crate) target: Option<ProtoTarget>,
}

pub(crate) struct TraceAnalysisSummary {
    pub(crate) language: &'static str,
    pub(crate) language_display: &'static str,
    pub(crate) frame_count: usize,
    pub(crate) consumed_bits: usize,
    pub(crate) symbolic_bits: usize,
    pub(crate) padding_bits: usize,
    pub(crate) encoded_bytes: usize,
    pub(crate) payload_bytes: Option<usize>,
    pub(crate) payload_hex: Option<String>,
    pub(crate) payload_utf8: Option<String>,
    pub(crate) contiguous_ranges: bool,
    pub(crate) integrity_ok: bool,
    pub(crate) integrity_error: Option<String>,
    pub(crate) envelope_present: bool,
    pub(crate) envelope_version: Option<u8>,
    pub(crate) envelope_kdf: Option<String>,
    pub(crate) envelope_aead: Option<String>,
    pub(crate) envelope_error: Option<String>,
}

#[derive(Clone, Copy)]
pub(crate) enum OutputFormat {
    Text,
    Json,
}

pub(crate) enum DemoTarget {
    Farsi,
    English,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ProtoTarget {
    Farsi,
    English,
}

impl ProtoTarget {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Farsi => "fa",
            Self::English => "en",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CliErrorKind {
    Usage,
    Input,
    Config,
    Security,
    Trace,
    Io,
    Domain,
    Internal,
}

#[derive(Debug, Clone)]
pub(crate) struct CliError {
    kind: CliErrorKind,
    code: &'static str,
    message: String,
}

impl CliError {
    pub(crate) fn usage(message: impl Into<String>) -> Self {
        Self::new(CliErrorKind::Usage, "LSTEG-CLI-ARG-001", message)
    }

    pub(crate) fn input(message: impl Into<String>) -> Self {
        Self::new(CliErrorKind::Input, "LSTEG-CLI-INP-001", message)
    }

    pub(crate) fn config(message: impl Into<String>) -> Self {
        Self::new(CliErrorKind::Config, "LSTEG-CLI-CFG-001", message)
    }

    pub(crate) fn trace(message: impl Into<String>) -> Self {
        Self::new(CliErrorKind::Trace, "LSTEG-CLI-TRC-001", message)
    }

    pub(crate) fn security(message: impl Into<String>) -> Self {
        Self::new(CliErrorKind::Security, "LSTEG-CLI-SEC-001", message)
    }

    pub(crate) fn io(operation: &str, path: Option<&str>, error: impl Display) -> Self {
        let location = path.map_or_else(String::new, |item| format!(" '{item}'"));
        Self::new(
            CliErrorKind::Io,
            "LSTEG-CLI-IO-001",
            format!("{operation}{location}: {error}"),
        )
    }

    pub(crate) fn domain(message: impl Into<String>) -> Self {
        Self::new(CliErrorKind::Domain, "LSTEG-CLI-DOM-001", message)
    }

    pub(crate) fn internal(message: impl Into<String>) -> Self {
        Self::new(CliErrorKind::Internal, "LSTEG-CLI-INT-001", message)
    }

    pub(crate) fn kind(&self) -> CliErrorKind {
        self.kind
    }

    pub(crate) fn code(&self) -> &'static str {
        self.code
    }

    pub(crate) fn message(&self) -> &str {
        &self.message
    }

    pub(crate) fn exit_code(&self) -> u8 {
        if self.kind == CliErrorKind::Usage {
            2
        } else {
            1
        }
    }

    fn new(kind: CliErrorKind, code: &'static str, message: impl Into<String>) -> Self {
        Self {
            kind,
            code,
            message: message.into(),
        }
    }
}

impl Display for CliError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}] {}", self.code, self.message)
    }
}
