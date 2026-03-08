use linguasteg_core::{
    CryptoEnvelopeInspection, FixedWidthPlanningOptions, SymbolicFramePlan, SymbolicFrameSchema,
    decode_payload_from_symbolic_frames, inspect_envelope, open_payload,
};

use super::data::{
    resolve_active_data_source_selection, resolve_active_data_source_variant_catalog,
};
use super::dataset::LexiconVariantCatalog;
use super::formatters::{build_trace_analysis_json, build_trace_analysis_text};
use super::language::resolve_trace_target;
use super::runtime::{PrototypeRuntime, initialize_runtime};
use super::symbol_mix::apply_secret_symbolic_mix;
use super::trace::{frame_sequence_error, parse_frames_from_trace, schema_for_template};
use super::types::{CliError, DecodeInputMode, OutputFormat, ProtoTarget, TraceAnalysisSummary};

pub(crate) fn render_trace_analysis_output(
    target: ProtoTarget,
    auto_detect_target: bool,
    input_mode: DecodeInputMode,
    trace_text: &str,
    format: OutputFormat,
    secret: Option<&[u8]>,
    data_dir: Option<&str>,
) -> Result<String, CliError> {
    let summary = analyze_trace_summary(
        "analyze",
        target,
        auto_detect_target,
        input_mode,
        trace_text,
        secret,
        data_dir,
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
    data_dir: Option<&str>,
) -> Result<TraceAnalysisSummary, CliError> {
    if trace_text.trim().is_empty() {
        return Err(operation_requires_trace_or_text_input_error(operation));
    }

    let target = resolve_trace_target(target, auto_detect_target, trace_text)?;
    let mut runtime = initialize_runtime(target.clone())?;
    let schemas = runtime.mapper.frame_schemas();
    let parsed_trace_frames = parse_frames_from_trace(trace_text, &schemas)
        .map_err(|error| CliError::trace(format!("failed to parse trace frames: {error}")))?;
    let frames = match input_mode {
        DecodeInputMode::Trace => {
            if parsed_trace_frames.is_empty() {
                return Err(operation_trace_mode_requires_trace_input_error(operation));
            }
            parsed_trace_frames
        }
        DecodeInputMode::Text => resolve_text_frames_with_auto_fallback(
            &mut runtime,
            target,
            auto_detect_target,
            trace_text,
            operation,
            operation_text_mode_requires_canonical_text_error,
            data_dir,
        )?,
        DecodeInputMode::Auto => {
            if parsed_trace_frames.is_empty() {
                resolve_text_frames_with_auto_fallback(
                    &mut runtime,
                    target,
                    auto_detect_target,
                    trace_text,
                    operation,
                    operation_auto_requires_trace_or_text_error,
                    data_dir,
                )?
            } else {
                parsed_trace_frames
            }
        }
    };

    if frames.is_empty() {
        return Err(CliError::trace("no frame lines were found in trace input"));
    }

    let active_schemas = runtime.mapper.frame_schemas();

    analyze_trace(
        runtime.language_code,
        runtime.language_display,
        &frames,
        &active_schemas,
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
            let mut candidate_payload = raw_payload;
            if let Some(secret_bytes) = secret {
                let decryptable_candidate = matches!(
                    inspect_envelope(&candidate_payload),
                    CryptoEnvelopeInspection::Metadata(_)
                ) && open_payload(&candidate_payload, secret_bytes)
                    .is_ok();

                if !decryptable_candidate {
                    let mut unmixed_frames = frames.to_vec();
                    apply_secret_symbolic_mix(&mut unmixed_frames, secret_bytes);
                    if let Ok(unmixed_payload) = decode_payload_from_symbolic_frames(
                        &unmixed_frames,
                        &ordered_schemas,
                        &FixedWidthPlanningOptions::default(),
                    ) {
                        let decryptable_unmixed = matches!(
                            inspect_envelope(&unmixed_payload),
                            CryptoEnvelopeInspection::Metadata(_)
                        ) && open_payload(&unmixed_payload, secret_bytes)
                            .is_ok();
                        if decryptable_unmixed {
                            candidate_payload = unmixed_payload;
                        }
                    }
                }
            }

            match inspect_envelope(&candidate_payload) {
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
                    match open_payload(&candidate_payload, secret) {
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

fn operation_requires_trace_or_text_input_error(operation: &str) -> CliError {
    CliError::input(format!(
        "{operation} requires input from proto-encode trace output or canonical stego text"
    ))
}

fn operation_trace_mode_requires_trace_input_error(operation: &str) -> CliError {
    CliError::input(format!(
        "{operation} trace mode requires proto-encode trace input (rerun encode with --emit-trace)"
    ))
}

fn text_decode_not_lossless_error(language_display: &str, operation: &str) -> CliError {
    CliError::input(format!(
        "{language_display} text decode is not lossless yet; rerun encode with --emit-trace and use {operation} --trace-input"
    ))
}

fn operation_text_mode_requires_canonical_text_error(operation: &str) -> CliError {
    CliError::input(format!(
        "{operation} text mode requires canonical stego text compatible with active language extractor"
    ))
}

fn operation_auto_requires_trace_or_text_error(operation: &str) -> CliError {
    CliError::input(format!(
        "{operation} requires parseable trace frames or canonical stego text"
    ))
}

fn resolve_text_frames_with_auto_fallback(
    runtime: &mut PrototypeRuntime,
    target: ProtoTarget,
    auto_detect_target: bool,
    trace_text: &str,
    operation: &str,
    missing_input_error: fn(&str) -> CliError,
    data_dir: Option<&str>,
) -> Result<Vec<SymbolicFramePlan>, CliError> {
    let variant_catalog = resolve_variant_catalog_for_target(&target, data_dir)?;
    if runtime.text_decode_lossless {
        if let Some(frames) = extract_text_frames(runtime, trace_text, variant_catalog.as_ref()) {
            return Ok(frames);
        }
    } else if !auto_detect_target {
        return Err(text_decode_not_lossless_error(
            runtime.language_display,
            operation,
        ));
    }

    if auto_detect_target {
        let fallback_target = alternate_target(target);
        let fallback_catalog = resolve_variant_catalog_for_target(&fallback_target, data_dir)?;
        let fallback_runtime = initialize_runtime(fallback_target.clone())?;
        if fallback_runtime.text_decode_lossless {
            if let Some(frames) =
                extract_text_frames(&fallback_runtime, trace_text, fallback_catalog.as_ref())
            {
                *runtime = fallback_runtime;
                return Ok(frames);
            }
        }
    }

    if !runtime.text_decode_lossless {
        return Err(text_decode_not_lossless_error(
            runtime.language_display,
            operation,
        ));
    }

    Err(missing_input_error(operation))
}

fn extract_text_frames(
    runtime: &PrototypeRuntime,
    trace_text: &str,
    variant_catalog: Option<&LexiconVariantCatalog>,
) -> Option<Vec<SymbolicFramePlan>> {
    let normalized_trace = variant_catalog.map(|catalog| catalog.normalize_text(trace_text));
    let extraction_input = normalized_trace.as_deref().unwrap_or(trace_text);
    runtime
        .extract_plans(extraction_input)
        .ok()
        .filter(|plans| !plans.is_empty())
        .and_then(|plans| runtime.mapper.map_plans_to_frames(&plans).ok())
}

fn resolve_variant_catalog_for_target(
    target: &ProtoTarget,
    data_dir: Option<&str>,
) -> Result<Option<LexiconVariantCatalog>, CliError> {
    let active_source = resolve_active_data_source_selection(target.clone(), None, data_dir)?;
    resolve_active_data_source_variant_catalog(
        target.clone(),
        active_source.as_ref().map(|item| item.source_id.as_str()),
        data_dir,
    )
}

fn alternate_target(target: ProtoTarget) -> ProtoTarget {
    match target {
        ProtoTarget::Farsi => ProtoTarget::English,
        ProtoTarget::English => ProtoTarget::Farsi,
        ProtoTarget::Other(code) => ProtoTarget::Other(code),
    }
}
