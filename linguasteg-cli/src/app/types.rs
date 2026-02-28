pub(crate) type DynError = Box<dyn std::error::Error>;

pub(crate) enum Command {
    Encode(EncodeOptions),
    Decode(DecodeOptions),
    Analyze(AnalyzeOptions),
    Demo(DemoTarget),
    ProtoEncode(ProtoTarget, String, bool),
    ProtoDecode(ProtoTarget, Option<String>, bool),
}

pub(crate) struct EncodeOptions {
    pub(crate) target: ProtoTarget,
    pub(crate) message: Option<String>,
    pub(crate) input_path: Option<String>,
    pub(crate) output_path: Option<String>,
    pub(crate) format: OutputFormat,
}

pub(crate) struct DecodeOptions {
    pub(crate) target: ProtoTarget,
    pub(crate) trace: Option<String>,
    pub(crate) input_path: Option<String>,
    pub(crate) output_path: Option<String>,
    pub(crate) format: OutputFormat,
}

pub(crate) struct AnalyzeOptions {
    pub(crate) target: ProtoTarget,
    pub(crate) trace: Option<String>,
    pub(crate) input_path: Option<String>,
    pub(crate) output_path: Option<String>,
    pub(crate) format: OutputFormat,
}

pub(crate) struct TraceAnalysisSummary {
    pub(crate) language: &'static str,
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
}

#[derive(Clone, Copy)]
pub(crate) enum OutputFormat {
    Text,
    Json,
}

pub(crate) enum DemoTarget {
    Farsi,
}

#[derive(Clone, Copy)]
pub(crate) enum ProtoTarget {
    Farsi,
}

pub(crate) enum CliError {
    Usage(String),
}
