use std::collections::HashMap;

use serde_json::Value;

#[derive(Debug, Clone)]
pub(crate) struct ProtoEncodeTrace {
    pub(crate) language: Option<String>,
    pub(crate) frames: Vec<ProtoEncodeTraceFrame>,
}

#[derive(Debug, Clone)]
pub(crate) struct ProtoEncodeTraceFrame {
    pub(crate) template_id: String,
    pub(crate) start_bit: usize,
    pub(crate) end_bit: usize,
    pub(crate) values: HashMap<String, u32>,
}

pub(crate) fn parse_proto_encode_trace_json(
    json_text: &str,
) -> Result<Option<ProtoEncodeTrace>, String> {
    let value: Value =
        serde_json::from_str(json_text).map_err(|error| format!("invalid json trace: {error}"))?;

    let Some(mode) = value.get("mode").and_then(Value::as_str) else {
        return Ok(None);
    };

    if mode != "proto-encode" {
        return Ok(None);
    }

    let language = value
        .get("language")
        .and_then(Value::as_str)
        .map(ToOwned::to_owned);

    let frames_value = value
        .get("frames")
        .cloned()
        .unwrap_or_else(|| Value::Array(Vec::new()));
    let frames_array = frames_value
        .as_array()
        .ok_or_else(|| "proto-encode json field 'frames' must be an array".to_string())?;

    let mut frames = Vec::with_capacity(frames_array.len());
    for (index, frame) in frames_array.iter().enumerate() {
        let frame_object = frame.as_object().ok_or_else(|| {
            format!(
                "proto-encode frame at index {} must be an object",
                index + 1
            )
        })?;

        let template_id = frame_object
            .get("template_id")
            .and_then(Value::as_str)
            .ok_or_else(|| {
                format!(
                    "proto-encode frame at index {} is missing string field 'template_id'",
                    index + 1
                )
            })?
            .to_string();

        let start_bit = parse_usize_field(frame_object, "start_bit", index + 1)?;
        let end_bit = parse_usize_field(frame_object, "end_bit", index + 1)?;

        let values_object = frame_object
            .get("values")
            .and_then(Value::as_object)
            .ok_or_else(|| {
                format!(
                    "proto-encode frame at index {} is missing object field 'values'",
                    index + 1
                )
            })?;

        let mut values = HashMap::with_capacity(values_object.len());
        for (slot, raw_value) in values_object {
            let Some(raw) = raw_value.as_u64() else {
                return Err(format!(
                    "proto-encode frame at index {} has non-integer value for slot '{}'",
                    index + 1,
                    slot
                ));
            };
            let value = u32::try_from(raw).map_err(|_| {
                format!(
                    "proto-encode frame at index {} has out-of-range value for slot '{}'",
                    index + 1,
                    slot
                )
            })?;
            values.insert(slot.clone(), value);
        }

        frames.push(ProtoEncodeTraceFrame {
            template_id,
            start_bit,
            end_bit,
            values,
        });
    }

    Ok(Some(ProtoEncodeTrace { language, frames }))
}

fn parse_usize_field(
    frame_object: &serde_json::Map<String, Value>,
    field: &str,
    index: usize,
) -> Result<usize, String> {
    let raw = frame_object
        .get(field)
        .and_then(Value::as_u64)
        .ok_or_else(|| {
            format!(
                "proto-encode frame at index {} is missing integer field '{}'",
                index, field
            )
        })?;

    usize::try_from(raw).map_err(|_| {
        format!(
            "proto-encode frame at index {} has out-of-range integer for field '{}'",
            index, field
        )
    })
}
