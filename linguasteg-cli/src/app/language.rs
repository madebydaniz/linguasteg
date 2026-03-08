use super::trace_contract::parse_proto_encode_trace_json;
use super::types::{CliError, ProtoTarget};

#[derive(Clone, Debug, PartialEq, Eq)]
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
            "trace contains mixed language templates".to_string(),
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
    let mut seen_de = false;
    let mut seen_it = false;

    let trimmed = trace_text.trim_start();
    if trimmed.starts_with('{') {
        inspect_json_trace_language(
            trimmed,
            &mut seen_fa,
            &mut seen_en,
            &mut seen_de,
            &mut seen_it,
        );
    }

    for line in trace_text.lines() {
        let trimmed_line = line.trim();
        if !trimmed_line.starts_with("frame ") {
            continue;
        }

        if let Some(template_id) = extract_template_id_from_frame_line(trimmed_line) {
            record_template(
                template_id,
                &mut seen_fa,
                &mut seen_en,
                &mut seen_de,
                &mut seen_it,
            );
        }
    }

    let seen_count =
        usize::from(seen_fa) + usize::from(seen_en) + usize::from(seen_de) + usize::from(seen_it);
    if seen_count > 1 {
        return TraceLanguageState::Mixed;
    }

    if seen_fa {
        TraceLanguageState::Single(ProtoTarget::Farsi)
    } else if seen_en {
        TraceLanguageState::Single(ProtoTarget::English)
    } else if seen_de {
        TraceLanguageState::Single(ProtoTarget::Other("de".to_string()))
    } else if seen_it {
        TraceLanguageState::Single(ProtoTarget::Other("it".to_string()))
    } else {
        TraceLanguageState::Undetected
    }
}

fn extract_template_id_from_frame_line(line: &str) -> Option<&str> {
    let open = line.find('[')?;
    let close_relative = line[open + 1..].find(']')?;
    let close = open + 1 + close_relative;
    Some(&line[open + 1..close])
}

fn inspect_json_trace_language(
    json_text: &str,
    seen_fa: &mut bool,
    seen_en: &mut bool,
    seen_de: &mut bool,
    seen_it: &mut bool,
) {
    let Ok(Some(trace)) = parse_proto_encode_trace_json(json_text) else {
        return;
    };

    if let Some(language) = trace.language.as_deref() {
        record_language(language, seen_fa, seen_en, seen_de, seen_it);
    }

    for frame in &trace.frames {
        record_template(&frame.template_id, seen_fa, seen_en, seen_de, seen_it);
    }
}

fn record_language(
    value: &str,
    seen_fa: &mut bool,
    seen_en: &mut bool,
    seen_de: &mut bool,
    seen_it: &mut bool,
) {
    match value {
        "fa" => *seen_fa = true,
        "en" => *seen_en = true,
        "de" => *seen_de = true,
        "it" => *seen_it = true,
        _ => {}
    }
}

fn record_template(
    value: &str,
    seen_fa: &mut bool,
    seen_en: &mut bool,
    seen_de: &mut bool,
    seen_it: &mut bool,
) {
    if value.starts_with("fa-") {
        *seen_fa = true;
    } else if value.starts_with("en-") {
        *seen_en = true;
    } else if value.starts_with("de-") {
        *seen_de = true;
    } else if value.starts_with("it-") {
        *seen_it = true;
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
    fn resolve_trace_target_auto_detects_german_from_json_language() {
        let trace = "{\"mode\":\"proto-encode\",\"language\":\"de\",\"frames\":[]}";
        let target =
            resolve_trace_target(ProtoTarget::Farsi, true, trace).expect("resolve should pass");
        assert_eq!(target.as_str(), "de");
    }

    #[test]
    fn resolve_trace_target_auto_detects_italian_from_json_language() {
        let trace = "{\"mode\":\"proto-encode\",\"language\":\"it\",\"frames\":[]}";
        let target =
            resolve_trace_target(ProtoTarget::Farsi, true, trace).expect("resolve should pass");
        assert_eq!(target.as_str(), "it");
    }

    #[test]
    fn resolve_trace_target_rejects_mixed_templates() {
        let trace = "frame 01 [fa-time-location-sov] bits=0..1 values=subject:1 => ...\nframe 02 [en-time-location-svo] bits=1..2 values=subject:1 => ...";
        let error = resolve_trace_target(ProtoTarget::Farsi, true, trace).expect_err("should fail");
        assert_eq!(error.code(), "LSTEG-CLI-CFG-001");
        assert!(error.message().contains("mixed language templates"));
    }
}
