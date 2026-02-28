use linguasteg_core::{
    FixedWidthPlanningOptions, SymbolicFramePlan, SymbolicFrameSchema,
    decode_payload_from_symbolic_frames,
};

use super::formatters::{build_trace_analysis_json, build_trace_analysis_text};
use super::runtime::FarsiProtoRuntime;
use super::trace::{parse_frames_from_trace, schema_for_template};
use super::types::{DynError, OutputFormat, TraceAnalysisSummary};

pub(crate) fn render_farsi_trace_analysis_output(
    trace_text: &str,
    format: OutputFormat,
) -> Result<String, DynError> {
    if trace_text.trim().is_empty() {
        return Err("analyze requires trace input from proto-encode output".into());
    }

    let runtime = FarsiProtoRuntime::new()?;
    let schemas = runtime.mapper.frame_schemas();
    let frames = parse_frames_from_trace(trace_text, &schemas)?;
    if frames.is_empty() {
        return Err("no frame lines were found in trace input".into());
    }

    let summary = analyze_farsi_trace(&frames, &schemas)?;
    if matches!(format, OutputFormat::Json) {
        return Ok(build_trace_analysis_json(&summary));
    }

    Ok(build_trace_analysis_text(&summary))
}

fn analyze_farsi_trace(
    frames: &[SymbolicFramePlan],
    schemas: &[SymbolicFrameSchema],
) -> Result<TraceAnalysisSummary, DynError> {
    let mut ordered_schemas = Vec::with_capacity(frames.len());
    let mut symbolic_bits = 0usize;
    let mut consumed_bits = 0usize;
    let mut contiguous_ranges = true;
    let mut expected_start = 0usize;

    for frame in frames {
        if frame.source.start_bit != expected_start {
            contiguous_ranges = false;
        }
        expected_start = frame.source.start_bit + frame.source.consumed_bits;
        consumed_bits += frame.source.consumed_bits;

        let schema = schema_for_template(schemas, &frame.template_id)?;
        symbolic_bits += schema.total_bits();
        ordered_schemas.push(schema);
    }

    let encoded_bytes = consumed_bits.div_ceil(8);
    let mut integrity_ok = contiguous_ranges;
    let mut integrity_error = if contiguous_ranges {
        None
    } else {
        Some("frame bit ranges are not contiguous".to_string())
    };

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
