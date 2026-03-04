use linguasteg_core::{
    CryptoEnvelopeInspection, FixedWidthPlanningOptions, SymbolicFramePlan, SymbolicFrameSchema,
    decode_payload_from_symbolic_frames, inspect_envelope, open_payload,
};

use super::formatters::{build_trace_analysis_json, build_trace_analysis_text};
use super::language::resolve_trace_target;
use super::runtime::PrototypeRuntime;
use super::trace::{frame_sequence_error, parse_frames_from_trace, schema_for_template};
use super::types::{CliError, DecodeInputMode, OutputFormat, ProtoTarget, TraceAnalysisSummary};

pub(crate) fn render_trace_analysis_output(
    target: ProtoTarget,
    auto_detect_target: bool,
    input_mode: DecodeInputMode,
    trace_text: &str,
    format: OutputFormat,
    secret: Option<&[u8]>,
) -> Result<String, CliError> {
    let summary = analyze_trace_summary(
        "analyze",
        target,
        auto_detect_target,
        input_mode,
        trace_text,
        secret,
    )?;
    if matches!(format, OutputFormat::Json) {
        return Ok(build_trace_analysis_json(&summary));
    }

    Ok(build_trace_analysis_text(&summary))
}

pub(crate) fn analyze_trace_summary(
    operation: &str,
    target: ProtoTarget,
    auto_detect_target: bool,
    input_mode: DecodeInputMode,
    trace_text: &str,
    secret: Option<&[u8]>,
) -> Result<TraceAnalysisSummary, CliError> {
    if trace_text.trim().is_empty() {
        return Err(CliError::input(format!(
            "{operation} requires input from proto-encode trace output or canonical stego text"
        )));
    }

    let target = resolve_trace_target(target, auto_detect_target, trace_text)?;
    let runtime = PrototypeRuntime::new(target).map_err(|error| {
        CliError::internal(format!(
            "failed to initialize {} runtime: {error}",
            target.as_str()
        ))
    })?;
    let schemas = runtime.mapper.frame_schemas();
    let parsed_trace_frames = parse_frames_from_trace(trace_text, &schemas)
        .map_err(|error| CliError::trace(format!("failed to parse trace frames: {error}")))?;
    let frames = match input_mode {
        DecodeInputMode::Trace => {
            if parsed_trace_frames.is_empty() {
                return Err(CliError::input(format!(
                    "{operation} trace mode requires proto-encode trace input (rerun encode with --emit-trace)"
                )));
            }
            parsed_trace_frames
        }
        DecodeInputMode::Text => {
            if !runtime.text_decode_lossless {
                return Err(CliError::input(format!(
                    "{} text decode is not lossless yet; rerun encode with --emit-trace and use --trace-input",
                    runtime.language_display
                )));
            }
            runtime
                .extract_plans(trace_text)
                .ok()
                .filter(|plans| !plans.is_empty())
                .and_then(|plans| runtime.mapper.map_plans_to_frames(&plans).ok())
                .ok_or_else(|| {
                    CliError::input(format!(
                        "{operation} text mode requires canonical stego text compatible with active language extractor"
                    ))
                })?
        }
        DecodeInputMode::Auto => {
            if parsed_trace_frames.is_empty() {
                if !runtime.text_decode_lossless {
                    return Err(CliError::input(format!(
                        "{} text decode is not lossless yet; rerun encode with --emit-trace and use --trace-input",
                        runtime.language_display
                    )));
                }
                runtime
                    .extract_plans(trace_text)
                    .ok()
                    .filter(|plans| !plans.is_empty())
                    .and_then(|plans| runtime.mapper.map_plans_to_frames(&plans).ok())
                    .ok_or_else(|| {
                        CliError::input(format!(
                            "{operation} requires parseable trace frames or canonical stego text"
                        ))
                    })?
            } else {
                parsed_trace_frames
            }
        }
    };

    if frames.is_empty() {
        return Err(CliError::trace("no frame lines were found in trace input"));
    }

    analyze_trace(
        runtime.language_code,
        runtime.language_display,
        &frames,
        &schemas,
        secret,
    )
}

fn analyze_trace(
    language: &'static str,
    language_display: &'static str,
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
    let mut envelope_present = false;
    let mut envelope_version = None;
    let mut envelope_kdf = None;
    let mut envelope_aead = None;
    let mut envelope_error = None;

    match decode_payload_from_symbolic_frames(
        frames,
        &ordered_schemas,
        &FixedWidthPlanningOptions::default(),
    ) {
        Ok(raw_payload) => {
            match inspect_envelope(&raw_payload) {
                CryptoEnvelopeInspection::NotEnvelope => {}
                CryptoEnvelopeInspection::Metadata(metadata) => {
                    envelope_present = true;
                    envelope_version = Some(metadata.version);
                    envelope_kdf = Some(metadata.kdf_name().to_string());
                    envelope_aead = Some(metadata.aead_name().to_string());
                }
                CryptoEnvelopeInspection::Invalid(message) => {
                    envelope_present = true;
                    envelope_error = Some(message);
                }
            }

            if let Some(secret) = secret {
                if envelope_present && envelope_error.is_none() {
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
                } else {
                    envelope_error.get_or_insert_with(|| {
                        "payload is not a valid secure envelope".to_string()
                    });
                    integrity_ok = false;
                    if integrity_error.is_none() {
                        integrity_error = Some(
                            "failed to decrypt payload; payload is not a valid secure envelope"
                                .to_string(),
                        );
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
        language,
        language_display,
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
        envelope_present,
        envelope_version,
        envelope_kdf,
        envelope_aead,
        envelope_error,
    })
}
