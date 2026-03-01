use super::types::{CliError, ProtoTarget};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum TraceLanguageState {
    Undetected,
    Single(ProtoTarget),
    Mixed,
}

pub(crate) fn resolve_trace_target(
    requested: ProtoTarget,
    auto_detect_target: bool,
    trace_text: &str,
) -> Result<ProtoTarget, CliError> {
    let state = inspect_trace_language(trace_text);
    if state == TraceLanguageState::Mixed {
        return Err(CliError::config(
            "trace contains mixed language templates (fa and en)".to_string(),
        ));
    }

    let detected = match state {
        TraceLanguageState::Single(target) => Some(target),
        _ => None,
    };

    if auto_detect_target {
        return Ok(detected.unwrap_or(requested));
    }

    if let Some(detected) = detected {
        if detected.as_str() != requested.as_str() {
            return Err(CliError::config(format!(
                "trace language '{}' does not match requested --lang '{}'",
                detected.as_str(),
                requested.as_str()
            )));
        }
    }

    Ok(requested)
}

fn inspect_trace_language(trace_text: &str) -> TraceLanguageState {
    let mut seen_fa = false;
    let mut seen_en = false;

    let trimmed = trace_text.trim_start();
    if trimmed.starts_with('{') {
        if let Some(language) = extract_json_string_field(trimmed, "language") {
            record_language(language, &mut seen_fa, &mut seen_en);
        }

        for template in extract_json_string_values(trimmed, "template_id") {
            record_template(template, &mut seen_fa, &mut seen_en);
        }
    }

    for line in trace_text.lines() {
        let trimmed_line = line.trim();
        if !trimmed_line.starts_with("frame ") {
            continue;
        }

        if let Some(template_id) = extract_template_id_from_frame_line(trimmed_line) {
            record_template(template_id, &mut seen_fa, &mut seen_en);
        }
    }

    match (seen_fa, seen_en) {
        (true, true) => TraceLanguageState::Mixed,
        (true, false) => TraceLanguageState::Single(ProtoTarget::Farsi),
        (false, true) => TraceLanguageState::Single(ProtoTarget::English),
        (false, false) => TraceLanguageState::Undetected,
    }
}

fn extract_template_id_from_frame_line(line: &str) -> Option<&str> {
    let open = line.find('[')?;
    let close_relative = line[open + 1..].find(']')?;
    let close = open + 1 + close_relative;
    Some(&line[open + 1..close])
}

fn extract_json_string_field<'a>(json_text: &'a str, key: &str) -> Option<&'a str> {
    let pattern = format!("\"{key}\":\"");
    let start = json_text.find(&pattern)? + pattern.len();
    let tail = &json_text[start..];
    let end = tail.find('"')?;
    Some(&tail[..end])
}

fn extract_json_string_values<'a>(json_text: &'a str, key: &str) -> Vec<&'a str> {
    let mut values = Vec::new();
    let pattern = format!("\"{key}\":\"");
    let mut cursor = 0usize;

    while cursor < json_text.len() {
        let Some(found) = json_text[cursor..].find(&pattern) else {
            break;
        };
        let start = cursor + found + pattern.len();
        let tail = &json_text[start..];
        let Some(end) = tail.find('"') else {
            break;
        };
        values.push(&tail[..end]);
        cursor = start + end + 1;
    }

    values
}

fn record_language(value: &str, seen_fa: &mut bool, seen_en: &mut bool) {
    match value {
        "fa" => *seen_fa = true,
        "en" => *seen_en = true,
        _ => {}
    }
}

fn record_template(value: &str, seen_fa: &mut bool, seen_en: &mut bool) {
    if value.starts_with("fa-") {
        *seen_fa = true;
    } else if value.starts_with("en-") {
        *seen_en = true;
    }
}

#[cfg(test)]
mod tests {
    use super::resolve_trace_target;
    use crate::app::types::ProtoTarget;

    #[test]
    fn resolve_trace_target_auto_detects_from_json_language() {
        let trace = "{\"mode\":\"proto-encode\",\"language\":\"en\",\"frames\":[]}";
        let target =
            resolve_trace_target(ProtoTarget::Farsi, true, trace).expect("resolve should pass");
        assert_eq!(target.as_str(), "en");
    }

    #[test]
    fn resolve_trace_target_rejects_mixed_templates() {
        let trace = "frame 01 [fa-time-location-sov] bits=0..1 values=subject:1 => ...\nframe 02 [en-time-location-svo] bits=1..2 values=subject:1 => ...";
        let error = resolve_trace_target(ProtoTarget::Farsi, true, trace).expect_err("should fail");
        assert_eq!(error.code(), "LSTEG-CLI-CFG-001");
        assert!(error.message().contains("mixed language templates"));
    }
}
