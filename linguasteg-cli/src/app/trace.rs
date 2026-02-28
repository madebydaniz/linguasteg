use std::collections::HashMap;

use linguasteg_core::{
    BitRange, SymbolicFramePlan, SymbolicFrameSchema, SymbolicSlotValue, TemplateId,
};

use super::types::DynError;

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
    let mode = extract_json_string_field(json_text, "mode");
    if mode.as_deref() != Some("proto-encode") {
        return Ok(None);
    }

    let frames_section = match extract_json_array_section(json_text, "frames") {
        Some(section) => section,
        None => return Ok(Some(Vec::new())),
    };

    let mut frames = Vec::new();
    for frame_object in split_json_objects(frames_section)? {
        let template_raw = extract_json_string_field(frame_object, "template_id")
            .ok_or_else(|| "frame object missing template_id".to_string())?;
        let template_id = TemplateId::new(&template_raw)?;
        let start_bit = extract_json_usize_field(frame_object, "start_bit")
            .ok_or_else(|| "frame object missing start_bit".to_string())?;
        let end_bit = extract_json_usize_field(frame_object, "end_bit")
            .ok_or_else(|| "frame object missing end_bit".to_string())?;
        let values_section = extract_json_object_section(frame_object, "values")
            .ok_or_else(|| "frame object missing values".to_string())?;
        let value_map = parse_value_map(values_section)?;

        let schema = schema_for_template(schemas, &template_id)?;
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

        frames.push(SymbolicFramePlan {
            template_id,
            source: BitRange {
                start_bit,
                consumed_bits: end_bit.saturating_sub(start_bit),
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

fn extract_json_string_field(json_text: &str, key: &str) -> Option<String> {
    let pattern = format!("\"{key}\":\"");
    let start = json_text.find(&pattern)? + pattern.len();
    let tail = &json_text[start..];
    let end = tail.find('"')?;
    Some(tail[..end].to_string())
}

fn extract_json_usize_field(json_text: &str, key: &str) -> Option<usize> {
    let pattern = format!("\"{key}\":");
    let start = json_text.find(&pattern)? + pattern.len();
    let tail = &json_text[start..];
    let digits_len = tail
        .chars()
        .take_while(|ch| ch.is_ascii_digit())
        .collect::<String>()
        .len();
    if digits_len == 0 {
        return None;
    }
    tail[..digits_len].parse::<usize>().ok()
}

fn extract_json_array_section<'a>(json_text: &'a str, key: &str) -> Option<&'a str> {
    let pattern = format!("\"{key}\":[");
    let start = json_text.find(&pattern)? + pattern.len();
    extract_balanced_section(&json_text[start - 1..], '[', ']').map(|section| {
        if section.len() >= 2 {
            &section[1..section.len() - 1]
        } else {
            section
        }
    })
}

fn extract_json_object_section<'a>(json_text: &'a str, key: &str) -> Option<&'a str> {
    let pattern = format!("\"{key}\":{{");
    let start = json_text.find(&pattern)? + pattern.len();
    extract_balanced_section(&json_text[start - 1..], '{', '}')
}

fn extract_balanced_section(input: &str, open_char: char, close_char: char) -> Option<&str> {
    if !input.starts_with(open_char) {
        return None;
    }
    let mut depth = 0usize;
    for (index, ch) in input.char_indices() {
        if ch == open_char {
            depth += 1;
        } else if ch == close_char {
            depth = depth.saturating_sub(1);
            if depth == 0 {
                return Some(&input[..=index]);
            }
        }
    }
    None
}

fn split_json_objects(array_content: &str) -> Result<Vec<&str>, DynError> {
    let mut objects = Vec::new();
    let mut depth = 0usize;
    let mut current_start: Option<usize> = None;

    for (index, ch) in array_content.char_indices() {
        if ch == '{' {
            if depth == 0 {
                current_start = Some(index);
            }
            depth += 1;
        } else if ch == '}' {
            if depth == 0 {
                return Err("malformed json frame array: unexpected '}'".into());
            }
            depth -= 1;
            if depth == 0 {
                if let Some(start) = current_start {
                    objects.push(&array_content[start..=index]);
                }
                current_start = None;
            }
        }
    }

    if depth != 0 {
        return Err("malformed json frame array: unbalanced braces".into());
    }

    Ok(objects)
}
