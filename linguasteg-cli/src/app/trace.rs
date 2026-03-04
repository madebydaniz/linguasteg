use std::collections::{HashMap, HashSet};

use linguasteg_core::{
    BitRange, SymbolicFramePlan, SymbolicFrameSchema, SymbolicSlotValue, TemplateId,
};

use super::trace_contract::parse_proto_encode_trace_json;

type DynError = Box<dyn std::error::Error>;

pub(crate) fn parse_frames_from_trace(
    trace_text: &str,
    schemas: &[SymbolicFrameSchema],
) -> Result<Vec<SymbolicFramePlan>, DynError> {
    let trimmed = trace_text.trim_start();
    if trimmed.starts_with('{') {
        if let Some(frames) = parse_frames_from_proto_encode_json(trimmed, schemas)? {
            return Ok(frames);
        }
    }

    let mut frames = Vec::new();

    for line in trace_text.lines() {
        let trimmed = line.trim();
        if !trimmed.starts_with("frame ") {
            continue;
        }

        let frame = parse_trace_line(trimmed, schemas)?;
        frames.push(frame);
    }

    Ok(frames)
}

pub(crate) fn frame_sequence_error(frames: &[SymbolicFramePlan]) -> Option<String> {
    let mut expected_start = 0usize;
    for (index, frame) in frames.iter().enumerate() {
        if frame.source.consumed_bits == 0 {
            return Some(format!("frame {:02} has zero consumed bits", index + 1));
        }
        if frame.source.start_bit != expected_start {
            return Some(format!(
                "frame {:02} starts at bit {} but expected {}",
                index + 1,
                frame.source.start_bit,
                expected_start
            ));
        }
        expected_start = frame
            .source
            .start_bit
            .saturating_add(frame.source.consumed_bits);
    }
    None
}

pub(crate) fn schema_for_template(
    schemas: &[SymbolicFrameSchema],
    template_id: &TemplateId,
) -> Result<SymbolicFrameSchema, String> {
    schemas
        .iter()
        .find(|schema| schema.template_id == *template_id)
        .cloned()
        .ok_or_else(|| format!("no schema found for template '{template_id}'"))
}

fn parse_frames_from_proto_encode_json(
    json_text: &str,
    schemas: &[SymbolicFrameSchema],
) -> Result<Option<Vec<SymbolicFramePlan>>, DynError> {
    let Some(trace) =
        parse_proto_encode_trace_json(json_text).map_err(|error| -> DynError { error.into() })?
    else {
        return Ok(None);
    };

    let mut frames = Vec::with_capacity(trace.frames.len());
    for frame in trace.frames {
        let template_id = TemplateId::new(&frame.template_id)?;

        let schema = schema_for_template(schemas, &template_id)?;
        let known_slots = schema
            .fields
            .iter()
            .map(|field| field.slot.as_str().to_string())
            .collect::<HashSet<_>>();
        for slot in frame.values.keys() {
            if !known_slots.contains(slot) {
                return Err(format!(
                    "unknown symbolic slot '{}' in template '{}'",
                    slot, template_id
                )
                .into());
            }
        }

        let consumed_bits = frame.end_bit - frame.start_bit;
        if consumed_bits != schema.total_bits() {
            return Err(format!(
                "invalid bit range for template '{}': expected {} bits, got {} ({}..{})",
                template_id,
                schema.total_bits(),
                consumed_bits,
                frame.start_bit,
                frame.end_bit
            )
            .into());
        }

        let values = schema
            .fields
            .iter()
            .map(|field| {
                let value = frame.values.get(field.slot.as_str()).ok_or_else(|| {
                    format!(
                        "missing symbolic value for slot '{}' in template '{}'",
                        field.slot, template_id
                    )
                })?;
                if !value_fits_bit_width(*value, field.bit_width) {
                    return Err(format!(
                        "symbolic value {} exceeds bit width {} for slot '{}' in template '{}'",
                        value, field.bit_width, field.slot, template_id
                    ));
                }

                Ok(SymbolicSlotValue {
                    slot: field.slot.clone(),
                    bit_width: field.bit_width,
                    value: *value,
                })
            })
            .collect::<Result<Vec<_>, String>>()?;

        frames.push(SymbolicFramePlan {
            template_id,
            source: BitRange {
                start_bit: frame.start_bit,
                consumed_bits,
            },
            values,
        });
    }

    Ok(Some(frames))
}

fn parse_trace_line(
    line: &str,
    schemas: &[SymbolicFrameSchema],
) -> Result<SymbolicFramePlan, DynError> {
    let template_id = extract_template_id(line)?;
    let (start_bit, end_bit) = extract_bit_range(line)?;
    let values_section = extract_values_section(line)?;

    let schema = schema_for_template(schemas, &template_id)?;
    let value_map = parse_value_map(values_section)?;
    let values = schema
        .fields
        .iter()
        .map(|field| {
            let value = value_map.get(field.slot.as_str()).ok_or_else(|| {
                format!(
                    "missing symbolic value for slot '{}' in template '{}'",
                    field.slot, template_id
                )
            })?;

            Ok(SymbolicSlotValue {
                slot: field.slot.clone(),
                bit_width: field.bit_width,
                value: *value,
            })
        })
        .collect::<Result<Vec<_>, String>>()?;

    Ok(SymbolicFramePlan {
        template_id,
        source: BitRange {
            start_bit,
            consumed_bits: end_bit.saturating_sub(start_bit),
        },
        values,
    })
}

fn extract_template_id(line: &str) -> Result<TemplateId, DynError> {
    let open = line
        .find('[')
        .ok_or_else(|| "trace line missing '[' for template id".to_string())?;
    let close_relative = line[open + 1..]
        .find(']')
        .ok_or_else(|| "trace line missing ']' for template id".to_string())?;
    let close = open + 1 + close_relative;
    let raw_template = &line[open + 1..close];
    Ok(TemplateId::new(raw_template)?)
}

fn extract_bit_range(line: &str) -> Result<(usize, usize), DynError> {
    let bits_label_index = line
        .find("bits=")
        .ok_or_else(|| "trace line missing bits section".to_string())?;
    let bits_start = bits_label_index + "bits=".len();
    let bits_tail = &line[bits_start..];
    let bits_end_relative = bits_tail
        .find(' ')
        .ok_or_else(|| "trace line has malformed bits section".to_string())?;
    let bits = &bits_tail[..bits_end_relative];
    let (start_raw, end_raw) = bits
        .split_once("..")
        .ok_or_else(|| "trace line bits section must use '..' range".to_string())?;
    let start_bit = start_raw.parse::<usize>()?;
    let end_bit = end_raw.parse::<usize>()?;
    Ok((start_bit, end_bit))
}

fn extract_values_section(line: &str) -> Result<&str, DynError> {
    let values_label_index = line
        .find("values=")
        .ok_or_else(|| "trace line missing values section".to_string())?;
    let values_start = values_label_index + "values=".len();
    let values_tail = &line[values_start..];
    let values_end = values_tail.find(" =>").unwrap_or(values_tail.len());
    Ok(&values_tail[..values_end])
}

fn parse_value_map(values_section: &str) -> Result<HashMap<String, u32>, DynError> {
    let mut normalized = values_section.trim();
    if normalized.starts_with('{') && normalized.ends_with('}') && normalized.len() >= 2 {
        normalized = &normalized[1..normalized.len() - 1];
    }

    if normalized.trim().is_empty() {
        return Err("trace values section is empty".into());
    }

    let mut parsed = HashMap::new();
    for part in normalized.split(',') {
        let trimmed = part.trim();
        if trimmed.is_empty() {
            continue;
        }

        let (slot, value_raw) = trimmed
            .split_once(':')
            .ok_or_else(|| format!("malformed symbolic value pair: '{trimmed}'"))?;
        let slot = slot.trim().trim_matches('"');
        let value = value_raw.parse::<u32>()?;
        parsed.insert(slot.to_string(), value);
    }

    Ok(parsed)
}

fn value_fits_bit_width(value: u32, bit_width: u8) -> bool {
    if bit_width >= 32 {
        return true;
    }
    value < (1_u32 << bit_width)
}

#[cfg(test)]
mod tests {
    use linguasteg_models::FarsiPrototypeSymbolicMapper;

    use super::{frame_sequence_error, parse_frames_from_trace};

    fn farsi_schemas() -> Vec<linguasteg_core::SymbolicFrameSchema> {
        FarsiPrototypeSymbolicMapper.frame_schemas()
    }

    #[test]
    fn parse_line_trace_returns_expected_frames() {
        let trace = "frame 01 [fa-basic-sov] bits=0..18 values=subject:0,object:0,adjective:0,verb:21 => x\nframe 02 [fa-time-location-sov] bits=18..39 values=subject:25,time:5,location:4,object:5,verb:22 => y";
        let frames = parse_frames_from_trace(trace, &farsi_schemas()).expect("trace should parse");

        assert_eq!(frames.len(), 2);
        assert_eq!(frames[0].template_id.as_str(), "fa-basic-sov");
        assert_eq!(frames[0].source.start_bit, 0);
        assert_eq!(frames[0].source.consumed_bits, 18);
        assert_eq!(frames[1].template_id.as_str(), "fa-time-location-sov");
        assert_eq!(frames[1].source.start_bit, 18);
        assert_eq!(frames[1].source.consumed_bits, 21);
    }

    #[test]
    fn parse_proto_encode_json_trace_returns_frames() {
        let trace = r#"{"mode":"proto-encode","language":"fa","frames":[{"index":1,"template_id":"fa-basic-sov","start_bit":0,"end_bit":18,"values":{"subject":0,"object":0,"adjective":0,"verb":21},"sentence":"x"},{"index":2,"template_id":"fa-time-location-sov","start_bit":18,"end_bit":39,"values":{"subject":25,"time":5,"location":4,"object":5,"verb":22},"sentence":"y"}]}"#;
        let frames =
            parse_frames_from_trace(trace, &farsi_schemas()).expect("json trace should parse");

        assert_eq!(frames.len(), 2);
        assert_eq!(frames[0].template_id.as_str(), "fa-basic-sov");
        assert_eq!(frames[1].template_id.as_str(), "fa-time-location-sov");
    }

    #[test]
    fn parse_proto_encode_json_fails_on_non_sequential_frame_index() {
        let trace = r#"{"mode":"proto-encode","language":"fa","frames":[{"index":2,"template_id":"fa-basic-sov","start_bit":0,"end_bit":18,"values":{"subject":0,"object":0,"adjective":0,"verb":21},"sentence":"x"}]}"#;
        let error = parse_frames_from_trace(trace, &farsi_schemas()).expect_err("json should fail");

        assert!(error.to_string().contains("inconsistent 'index'"));
    }

    #[test]
    fn parse_proto_encode_json_fails_on_frame_count_mismatch() {
        let trace = r#"{"mode":"proto-encode","language":"fa","frame_count":2,"frames":[{"index":1,"template_id":"fa-basic-sov","start_bit":0,"end_bit":18,"values":{"subject":0,"object":0,"adjective":0,"verb":21},"sentence":"x"}]}"#;
        let error = parse_frames_from_trace(trace, &farsi_schemas()).expect_err("json should fail");

        assert!(error.to_string().contains("field 'frame_count' mismatch"));
    }

    #[test]
    fn parse_proto_encode_json_fails_on_invalid_frame_range() {
        let trace = r#"{"mode":"proto-encode","language":"fa","frames":[{"index":1,"template_id":"fa-basic-sov","start_bit":18,"end_bit":18,"values":{"subject":0,"object":0,"adjective":0,"verb":21},"sentence":"x"}]}"#;
        let error = parse_frames_from_trace(trace, &farsi_schemas()).expect_err("json should fail");

        assert!(error.to_string().contains("invalid bit range"));
    }

    #[test]
    fn parse_proto_encode_json_fails_on_schema_width_mismatch() {
        let trace = r#"{"mode":"proto-encode","language":"fa","frames":[{"index":1,"template_id":"fa-basic-sov","start_bit":0,"end_bit":17,"values":{"subject":0,"object":0,"adjective":0,"verb":21},"sentence":"x"}]}"#;
        let error = parse_frames_from_trace(trace, &farsi_schemas()).expect_err("json should fail");

        assert!(error.to_string().contains("expected 18 bits, got 17"));
    }

    #[test]
    fn parse_proto_encode_json_fails_on_unknown_symbolic_slot() {
        let trace = r#"{"mode":"proto-encode","language":"fa","frames":[{"index":1,"template_id":"fa-basic-sov","start_bit":0,"end_bit":18,"values":{"subject":0,"object":0,"adjective":0,"verb":21,"unexpected":1},"sentence":"x"}]}"#;
        let error = parse_frames_from_trace(trace, &farsi_schemas()).expect_err("json should fail");

        assert!(
            error
                .to_string()
                .contains("unknown symbolic slot 'unexpected'")
        );
    }

    #[test]
    fn parse_proto_encode_json_fails_on_out_of_range_slot_value() {
        let trace = r#"{"mode":"proto-encode","language":"fa","frames":[{"index":1,"template_id":"fa-basic-sov","start_bit":0,"end_bit":18,"values":{"subject":0,"object":0,"adjective":8,"verb":21},"sentence":"x"}]}"#;
        let error = parse_frames_from_trace(trace, &farsi_schemas()).expect_err("json should fail");

        assert!(
            error
                .to_string()
                .contains("symbolic value 8 exceeds bit width 3")
        );
    }

    #[test]
    fn parse_non_proto_json_returns_empty() {
        let trace = r#"{"mode":"proto-decode","language":"fa","decoded_bytes":5}"#;
        let frames =
            parse_frames_from_trace(trace, &farsi_schemas()).expect("parser should not fail");

        assert!(frames.is_empty());
    }

    #[test]
    fn parse_trace_fails_when_required_slot_is_missing() {
        let trace = "frame 01 [fa-basic-sov] bits=0..18 values=subject:0,object:0,verb:21 => x";
        let error =
            parse_frames_from_trace(trace, &farsi_schemas()).expect_err("trace should fail");

        assert!(
            error
                .to_string()
                .contains("missing symbolic value for slot 'adjective'")
        );
    }

    #[test]
    fn parse_proto_encode_json_fails_on_malformed_frame_array() {
        let trace = r#"{"mode":"proto-encode","language":"fa","frames":[{"index":1,"template_id":"fa-basic-sov","start_bit":0,"end_bit":18,"values":{"subject":0,"object":0,"adjective":0,"verb":21}}}]}"#;
        let error = parse_frames_from_trace(trace, &farsi_schemas()).expect_err("json should fail");

        assert!(error.to_string().contains("invalid json trace:"));
    }

    #[test]
    fn frame_sequence_error_detects_non_contiguous_ranges() {
        let trace = "frame 01 [fa-basic-sov] bits=0..18 values=subject:0,object:0,adjective:0,verb:21 => x\nframe 02 [fa-time-location-sov] bits=19..40 values=subject:25,time:5,location:4,object:5,verb:22 => y";
        let frames = parse_frames_from_trace(trace, &farsi_schemas()).expect("trace should parse");
        let error = frame_sequence_error(&frames).expect("should detect range gap");
        assert!(error.contains("frame 02 starts at bit 19 but expected 18"));
    }
}
