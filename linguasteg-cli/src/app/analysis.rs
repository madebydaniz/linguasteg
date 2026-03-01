use linguasteg_core::{
    FixedWidthPlanningOptions, SymbolicFramePlan, SymbolicFrameSchema,
    decode_payload_from_symbolic_frames, open_payload,
};

use super::formatters::{build_trace_analysis_json, build_trace_analysis_text};
use super::runtime::FarsiProtoRuntime;
use super::trace::{frame_sequence_error, parse_frames_from_trace, schema_for_template};
use super::types::{CliError, OutputFormat, TraceAnalysisSummary};

pub(crate) fn render_farsi_trace_analysis_output(
    trace_text: &str,
    format: OutputFormat,
    secret: Option<&[u8]>,
) -> Result<String, CliError> {
    if trace_text.trim().is_empty() {
        return Err(CliError::input(
            "analyze requires trace input from proto-encode output",
        ));
    }

    let runtime = FarsiProtoRuntime::new().map_err(|error| {
        CliError::internal(format!("failed to initialize Farsi runtime: {error}"))
    })?;
    let schemas = runtime.mapper.frame_schemas();
    let frames = parse_frames_from_trace(trace_text, &schemas)
        .map_err(|error| CliError::trace(format!("failed to parse trace frames: {error}")))?;
    if frames.is_empty() {
        return Err(CliError::trace("no frame lines were found in trace input"));
    }

    let summary = analyze_farsi_trace(&frames, &schemas, secret)?;
    if matches!(format, OutputFormat::Json) {
        return Ok(build_trace_analysis_json(&summary));
    }

    Ok(build_trace_analysis_text(&summary))
}

fn analyze_farsi_trace(
    frames: &[SymbolicFramePlan],
    schemas: &[SymbolicFrameSchema],
    secret: Option<&[u8]>,
) -> Result<TraceAnalysisSummary, CliError> {
    let mut ordered_schemas = Vec::with_capacity(frames.len());
    let mut symbolic_bits = 0usize;
    let mut consumed_bits = 0usize;
    let sequence_error = frame_sequence_error(frames);
    let contiguous_ranges = sequence_error.is_none();

    for frame in frames {
        consumed_bits += frame.source.consumed_bits;

        let schema = schema_for_template(schemas, &frame.template_id).map_err(|error| {
            CliError::trace(format!("failed to resolve trace schemas: {error}"))
        })?;
        symbolic_bits += schema.total_bits();
        ordered_schemas.push(schema);
    }

    let encoded_bytes = consumed_bits.div_ceil(8);
    let mut integrity_ok = contiguous_ranges;
    let mut integrity_error = sequence_error;

    let padding_bits = if symbolic_bits >= consumed_bits {
        symbolic_bits - consumed_bits
    } else {
        integrity_ok = false;
        integrity_error = Some(format!(
            "consumed bits ({consumed_bits}) exceed symbolic capacity ({symbolic_bits})"
        ));
        0
    };

    let mut payload_bytes = None;
    let mut payload_hex = None;
    let mut payload_utf8 = None;

    match decode_payload_from_symbolic_frames(
        frames,
        &ordered_schemas,
        &FixedWidthPlanningOptions::default(),
    ) {
        Ok(raw_payload) => {
            if let Some(secret) = secret {
                match open_payload(&raw_payload, secret) {
                    Ok(payload) => {
                        payload_bytes = Some(payload.len());
                        payload_hex = Some(
                            payload
                                .iter()
                                .map(|byte| format!("{byte:02x}"))
                                .collect::<Vec<_>>()
                                .join(""),
                        );
                        payload_utf8 = String::from_utf8(payload).ok();
                    }
                    Err(_) => {
                        integrity_ok = false;
                        if integrity_error.is_none() {
                            integrity_error = Some(
                                "failed to decrypt payload; verify provided secret".to_string(),
                            );
                        }
                    }
                }
            }
        }
        Err(error) => {
            integrity_ok = false;
            if integrity_error.is_none() {
                integrity_error = Some(error.to_string());
            }
        }
    }

    Ok(TraceAnalysisSummary {
        language: "fa",
        frame_count: frames.len(),
        consumed_bits,
        symbolic_bits,
        padding_bits,
        encoded_bytes,
        payload_bytes,
        payload_hex,
        payload_utf8,
        contiguous_ranges,
        integrity_ok,
        integrity_error,
    })
}
