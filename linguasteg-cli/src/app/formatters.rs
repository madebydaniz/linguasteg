use linguasteg_core::SymbolicFramePlan;

use super::types::TraceAnalysisSummary;

pub(crate) fn build_trace_analysis_text(summary: &TraceAnalysisSummary) -> String {
    let mut lines = Vec::new();
    lines.push(format!("{} prototype analyze", summary.language_display));
    lines.push(format!("language: {}", summary.language));
    lines.push(format!("frames: {}", summary.frame_count));
    lines.push(format!("consumed bits: {}", summary.consumed_bits));
    lines.push(format!("symbolic bits: {}", summary.symbolic_bits));
    lines.push(format!("padding bits: {}", summary.padding_bits));
    lines.push(format!("encoded bytes: {}", summary.encoded_bytes));
    match summary.payload_bytes {
        Some(count) => lines.push(format!("payload bytes: {count}")),
        None => lines.push("payload bytes: <unavailable>".to_string()),
    }
    match &summary.payload_hex {
        Some(payload_hex) => lines.push(format!("payload hex: {payload_hex}")),
        None => lines.push("payload hex: <unavailable>".to_string()),
    }
    match &summary.payload_utf8 {
        Some(payload_utf8) => lines.push(format!("payload utf8: {payload_utf8}")),
        None => lines.push("payload utf8: <unavailable>".to_string()),
    }
    lines.push(format!(
        "secure envelope: {}",
        if summary.envelope_present {
            "detected"
        } else {
            "not-detected"
        }
    ));
    match summary.envelope_version {
        Some(version) => lines.push(format!("envelope version: {version}")),
        None => lines.push("envelope version: <unavailable>".to_string()),
    }
    match &summary.envelope_kdf {
        Some(kdf) => lines.push(format!("envelope kdf: {kdf}")),
        None => lines.push("envelope kdf: <unavailable>".to_string()),
    }
    match &summary.envelope_aead {
        Some(aead) => lines.push(format!("envelope aead: {aead}")),
        None => lines.push("envelope aead: <unavailable>".to_string()),
    }
    if let Some(error) = &summary.envelope_error {
        lines.push(format!("envelope error: {error}"));
    }
    lines.push(format!(
        "contiguous ranges: {}",
        if summary.contiguous_ranges {
            "yes"
        } else {
            "no"
        }
    ));
    lines.push(format!(
        "trace integrity: {}",
        if summary.integrity_ok { "ok" } else { "failed" }
    ));
    if let Some(error) = &summary.integrity_error {
        lines.push(format!("integrity error: {error}"));
    }
    lines.join("\n")
}

pub(crate) fn build_trace_analysis_json(summary: &TraceAnalysisSummary) -> String {
    let payload_bytes = match summary.payload_bytes {
        Some(value) => value.to_string(),
        None => "null".to_string(),
    };
    let payload_hex = match &summary.payload_hex {
        Some(value) => format!("\"{}\"", json_escape(value)),
        None => "null".to_string(),
    };
    let payload_utf8 = match &summary.payload_utf8 {
        Some(value) => format!("\"{}\"", json_escape(value)),
        None => "null".to_string(),
    };
    let integrity_error = match &summary.integrity_error {
        Some(value) => format!("\"{}\"", json_escape(value)),
        None => "null".to_string(),
    };
    let envelope_version = match summary.envelope_version {
        Some(value) => value.to_string(),
        None => "null".to_string(),
    };
    let envelope_kdf = match &summary.envelope_kdf {
        Some(value) => format!("\"{}\"", json_escape(value)),
        None => "null".to_string(),
    };
    let envelope_aead = match &summary.envelope_aead {
        Some(value) => format!("\"{}\"", json_escape(value)),
        None => "null".to_string(),
    };
    let envelope_error = match &summary.envelope_error {
        Some(value) => format!("\"{}\"", json_escape(value)),
        None => "null".to_string(),
    };

    format!(
        "{{\"mode\":\"analyze\",\"language\":\"{}\",\"frame_count\":{},\"consumed_bits\":{},\"symbolic_bits\":{},\"padding_bits\":{},\"encoded_bytes\":{},\"payload_bytes\":{},\"payload_hex\":{},\"payload_utf8\":{},\"envelope_present\":{},\"envelope_version\":{},\"envelope_kdf\":{},\"envelope_aead\":{},\"envelope_error\":{},\"contiguous_ranges\":{},\"integrity_ok\":{},\"integrity_error\":{}}}",
        summary.language,
        summary.frame_count,
        summary.consumed_bits,
        summary.symbolic_bits,
        summary.padding_bits,
        summary.encoded_bytes,
        payload_bytes,
        payload_hex,
        payload_utf8,
        summary.envelope_present,
        envelope_version,
        envelope_kdf,
        envelope_aead,
        envelope_error,
        summary.contiguous_ranges,
        summary.integrity_ok,
        integrity_error
    )
}

pub(crate) fn build_proto_encode_json(
    language: &str,
    input_text: &str,
    payload_bytes: usize,
    encoded_bytes: usize,
    padding_bits: u8,
    frames: &[SymbolicFramePlan],
    sentences: &[String],
    final_text: &str,
    gateway_response: Option<&str>,
) -> String {
    let mut frame_items = Vec::with_capacity(frames.len());
    for (index, frame) in frames.iter().enumerate() {
        let mut values = String::new();
        values.push('{');
        for (value_index, value) in frame.values.iter().enumerate() {
            if value_index > 0 {
                values.push(',');
            }
            values.push_str(&format!(
                "\"{}\":{}",
                json_escape(value.slot.as_str()),
                value.value
            ));
        }
        values.push('}');

        let sentence = sentences.get(index).map_or("", |item| item.as_str());
        frame_items.push(format!(
            "{{\"index\":{},\"template_id\":\"{}\",\"start_bit\":{},\"end_bit\":{},\"values\":{},\"sentence\":\"{}\"}}",
            index + 1,
            json_escape(frame.template_id.as_str()),
            frame.source.start_bit,
            frame.source.start_bit + frame.source.consumed_bits,
            values,
            json_escape(sentence)
        ));
    }
    let gateway_json = match gateway_response {
        Some(content) => format!("\"{}\"", json_escape(content)),
        None => "null".to_string(),
    };

    format!(
        "{{\"mode\":\"proto-encode\",\"language\":\"{}\",\"input_text\":\"{}\",\"payload_bytes\":{},\"encoded_bytes\":{},\"frame_count\":{},\"padding_bits\":{},\"frames\":[{}],\"final_text\":\"{}\",\"gateway_response\":{}}}",
        json_escape(language),
        json_escape(input_text),
        payload_bytes,
        encoded_bytes,
        frames.len(),
        padding_bits,
        frame_items.join(","),
        json_escape(final_text),
        gateway_json
    )
}

pub(crate) fn build_proto_decode_json(
    language: &str,
    decoded_bytes: usize,
    payload_hex: &str,
    payload_utf8: Option<&str>,
    gateway_response: Option<&str>,
) -> String {
    let utf8_json = match payload_utf8 {
        Some(text) => format!("\"{}\"", json_escape(text)),
        None => "null".to_string(),
    };
    let gateway_json = match gateway_response {
        Some(content) => format!("\"{}\"", json_escape(content)),
        None => "null".to_string(),
    };

    format!(
        "{{\"mode\":\"proto-decode\",\"language\":\"{}\",\"decoded_bytes\":{},\"payload_hex\":\"{}\",\"payload_utf8\":{},\"gateway_response\":{}}}",
        json_escape(language),
        decoded_bytes,
        json_escape(payload_hex),
        utf8_json,
        gateway_json
    )
}

pub(crate) fn json_escape(input: &str) -> String {
    let mut escaped = String::with_capacity(input.len());
    for ch in input.chars() {
        match ch {
            '"' => escaped.push_str("\\\""),
            '\\' => escaped.push_str("\\\\"),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            '\u{08}' => escaped.push_str("\\b"),
            '\u{0C}' => escaped.push_str("\\f"),
            _ => escaped.push(ch),
        }
    }
    escaped
}
