use std::fmt::Display;
use std::io::Read;

use linguasteg_core::{
    DecodeRequest, EncodeRequest, FixedWidthPlanningOptions, GrammarConstraintChecker,
    LanguageRealizer, LanguageTag, RealizationPlan, SlotAssignment, SlotId, StyleProfileRegistry,
    TemplateId, TemplateRegistry, open_payload, seal_payload,
};

use super::analysis::render_trace_analysis_output;
use super::formatters::{build_proto_decode_json, build_proto_encode_json};
use super::language::resolve_trace_target;
use super::runtime::PrototypeRuntime;
use super::trace::{frame_sequence_error, parse_frames_from_trace, schema_for_template};
use super::types::{
    AnalyzeOptions, CliError, Command, DecodeOptions, DemoTarget, EncodeOptions, OutputFormat,
    ProtoTarget,
};

pub(crate) fn execute(command: Command) -> Result<(), CliError> {
    match command {
        Command::Encode(options) => run_encode(options)?,
        Command::Decode(options) => run_decode(options)?,
        Command::Analyze(options) => run_analyze(options)?,
        Command::Demo(DemoTarget::Farsi) => run_demo(ProtoTarget::Farsi)?,
        Command::Demo(DemoTarget::English) => run_demo(ProtoTarget::English)?,
        Command::ProtoEncode(target, payload_text, json) => {
            run_proto_encode(target, &payload_text, json)?
        }
        Command::ProtoDecode(target, trace_input, json) => {
            run_proto_decode(target, trace_input, json)?
        }
    }

    Ok(())
}

fn run_encode(options: EncodeOptions) -> Result<(), CliError> {
    let payload_text = resolve_encode_payload(&options)?;
    let secret = resolve_required_secret_bytes(
        options.secret.as_deref(),
        options.secret_file.as_deref(),
        "encode",
    )?;
    let output =
        render_proto_encode_output(options.target, &payload_text, options.format, Some(&secret))?;
    write_output(&output, options.output_path.as_deref())
}

fn run_decode(options: DecodeOptions) -> Result<(), CliError> {
    let trace_text = resolve_trace_input(options.trace.as_deref(), options.input_path.as_deref())?;
    let secret = resolve_required_secret_bytes(
        options.secret.as_deref(),
        options.secret_file.as_deref(),
        "decode",
    )?;
    let output = render_proto_decode_output(
        options.target,
        options.auto_detect_target,
        &trace_text,
        options.format,
        Some(&secret),
    )?;
    write_output(&output, options.output_path.as_deref())
}

fn run_analyze(options: AnalyzeOptions) -> Result<(), CliError> {
    let trace_text = resolve_trace_input(options.trace.as_deref(), options.input_path.as_deref())?;
    let secret =
        resolve_optional_secret_bytes(options.secret.as_deref(), options.secret_file.as_deref())?;
    let output = render_trace_analysis_output(
        options.target,
        options.auto_detect_target,
        &trace_text,
        options.format,
        secret.as_deref(),
    )?;
    write_output(&output, options.output_path.as_deref())
}

fn run_demo(target: ProtoTarget) -> Result<(), CliError> {
    let runtime = runtime_for_target(target)?;
    let language = map_domain(
        LanguageTag::new(runtime.language_code),
        "invalid language tag",
    )?;
    let templates = runtime.pack.templates_for_language(&language);
    let style_profiles = runtime.pack.style_profiles_for_language(&language);

    println!("{} prototype demo", runtime.language_display);
    println!("language: {language}");
    println!("templates: {}", templates.len());
    for template in &templates {
        println!("  - {} ({})", template.id, template.display_name);
    }
    println!("style profiles: {}", style_profiles.len());
    for profile in &style_profiles {
        println!("  - {} ({})", profile.id, profile.display_name);
    }

    let template_id = map_domain(
        TemplateId::new(time_location_template_id(target)),
        "invalid template identifier",
    )?;
    let template = runtime
        .pack
        .template(&template_id)
        .ok_or_else(|| CliError::domain(format!("missing template: {template_id}")))?;

    let valid_plan = RealizationPlan {
        template_id: template_id.clone(),
        assignments: demo_assignments(target, true)?
            .into_iter()
            .map(|(slot, surface)| assignment(slot, surface))
            .collect::<Result<Vec<_>, _>>()?,
    };

    map_domain(
        runtime.checker.validate_plan(template, &valid_plan),
        "demo plan validation failed",
    )?;
    let rendered = map_domain(
        runtime.realizer.render(template, &valid_plan),
        "demo realization failed",
    )?;
    println!();
    println!("valid render:");
    println!("  {rendered}");

    let invalid_plan = RealizationPlan {
        template_id,
        assignments: demo_assignments(target, false)?
            .into_iter()
            .map(|(slot, surface)| assignment(slot, surface))
            .collect::<Result<Vec<_>, _>>()?,
    };

    println!();
    println!("invalid semantic combo demo:");
    match runtime.checker.validate_plan(template, &invalid_plan) {
        Ok(()) => println!("  unexpected: invalid plan was accepted"),
        Err(error) => println!("  rejected as expected: {error}"),
    }

    Ok(())
}

fn demo_assignments(
    target: ProtoTarget,
    valid: bool,
) -> Result<Vec<(&'static str, &'static str)>, CliError> {
    let data = match (target, valid) {
        (ProtoTarget::Farsi, true) => vec![
            ("subject", "دانشجو"),
            ("time", "امروز"),
            ("location", "کتابخانه"),
            ("object", "نامه"),
            ("verb", "نوشت"),
        ],
        (ProtoTarget::Farsi, false) => vec![
            ("subject", "دانشجو"),
            ("time", "امروز"),
            ("location", "کتابخانه"),
            ("object", "کتاب"),
            ("verb", "نوشید"),
        ],
        (ProtoTarget::English, true) => vec![
            ("subject", "the writer"),
            ("time", "today"),
            ("location", "in the library"),
            ("verb", "writes"),
            ("object", "letter"),
        ],
        (ProtoTarget::English, false) => vec![
            ("subject", " "),
            ("time", "today"),
            ("location", "in the library"),
            ("verb", "writes"),
            ("object", "letter"),
        ],
    };
    Ok(data)
}

fn time_location_template_id(target: ProtoTarget) -> &'static str {
    match target {
        ProtoTarget::Farsi => "fa-time-location-sov",
        ProtoTarget::English => "en-time-location-svo",
    }
}

fn assignment(slot: &str, surface: &str) -> Result<SlotAssignment, CliError> {
    Ok(SlotAssignment {
        slot: map_domain(SlotId::new(slot), "invalid slot identifier")?,
        surface: surface.to_string(),
        lemma: None,
    })
}

fn run_proto_encode(target: ProtoTarget, payload_text: &str, json: bool) -> Result<(), CliError> {
    let format = if json {
        OutputFormat::Json
    } else {
        OutputFormat::Text
    };
    let output = render_proto_encode_output(target, payload_text, format, None)?;
    println!("{output}");
    Ok(())
}

fn run_proto_decode(
    target: ProtoTarget,
    trace_input: Option<String>,
    json: bool,
) -> Result<(), CliError> {
    let trace_text = match trace_input {
        Some(value) => value,
        None => {
            let mut buffer = String::new();
            std::io::stdin()
                .read_to_string(&mut buffer)
                .map_err(|error| CliError::io("failed to read trace from stdin", None, error))?;
            buffer
        }
    };

    let format = if json {
        OutputFormat::Json
    } else {
        OutputFormat::Text
    };
    let output = render_proto_decode_output(target, false, &trace_text, format, None)?;
    println!("{output}");
    Ok(())
}

fn render_proto_encode_output(
    target: ProtoTarget,
    payload_text: &str,
    format: OutputFormat,
    secret: Option<&[u8]>,
) -> Result<String, CliError> {
    let payload = payload_text.as_bytes();
    let symbolic_payload = match secret {
        Some(secret) => seal_payload(payload, secret)
            .map_err(|_| CliError::security("failed to encrypt payload with provided secret"))?,
        None => payload.to_vec(),
    };
    let runtime = runtime_for_target(target)?;
    let schemas = runtime.mapper.frame_schemas();
    let orchestration = map_domain(
        runtime.orchestrator().orchestrate_encode(
            EncodeRequest {
                carrier_text: payload_text.to_string(),
                payload: symbolic_payload,
                options: runtime.pipeline_options().map_err(|error| {
                    CliError::config(format!("invalid pipeline options: {error}"))
                })?,
            },
            &schemas,
        ),
        "encode orchestration failed",
    )?;
    let payload_plan = orchestration.symbolic_plan;
    let realization_plans = map_domain(
        runtime.mapper.map_payload_to_plans(&payload_plan),
        "failed to map payload to realization plans",
    )?;

    let mut sentences = Vec::with_capacity(realization_plans.len());
    let mut frame_lines = Vec::with_capacity(realization_plans.len());
    for (index, plan) in realization_plans.iter().enumerate() {
        let template = runtime
            .pack
            .template(&plan.template_id)
            .ok_or_else(|| CliError::domain(format!("missing template: {}", plan.template_id)))?;
        map_domain(
            runtime.checker.validate_plan(template, plan),
            "render plan validation failed",
        )?;
        let sentence = map_domain(
            runtime.realizer.render(template, plan),
            "render plan failed",
        )?;
        let symbol_values = payload_plan.frames[index]
            .values
            .iter()
            .map(|value| format!("{}:{}", value.slot, value.value))
            .collect::<Vec<_>>()
            .join(",");
        frame_lines.push(format!(
            "frame {:02} [{}] bits={}..{} values={} => {}",
            index + 1,
            plan.template_id,
            payload_plan.frames[index].source.start_bit,
            payload_plan.frames[index].source.start_bit
                + payload_plan.frames[index].source.consumed_bits,
            symbol_values,
            sentence
        ));
        sentences.push(sentence);
    }

    let final_text = format!("{}.", sentences.join(". "));
    let gateway_response = orchestration.gateway_response.map(|item| item.content);

    if matches!(format, OutputFormat::Json) {
        return Ok(build_proto_encode_json(
            runtime.language_code,
            payload_text,
            payload.len(),
            payload_plan.encoded_len_bytes,
            payload_plan.padding_bits,
            &payload_plan.frames,
            &sentences,
            &final_text,
            gateway_response.as_deref(),
        ));
    }

    let mut report_lines = Vec::new();
    report_lines.push(format!("{} prototype encode", runtime.language_display));
    report_lines.push(format!("input text: {payload_text}"));
    report_lines.push(format!("payload bytes: {}", payload.len()));
    report_lines.push(format!(
        "encoded bytes (with length prefix): {}",
        payload_plan.encoded_len_bytes
    ));
    report_lines.push(format!("frames: {}", payload_plan.frames.len()));
    report_lines.push(format!("padding bits: {}", payload_plan.padding_bits));
    report_lines.push(String::new());
    for line in frame_lines {
        report_lines.push(line);
    }
    report_lines.push(String::new());
    report_lines.push("final prototype text:".to_string());
    report_lines.push(final_text);

    if let Some(gateway_response) = gateway_response {
        report_lines.push(format!("gateway response: {gateway_response}"));
    }

    Ok(report_lines.join("\n"))
}

fn render_proto_decode_output(
    target: ProtoTarget,
    auto_detect_target: bool,
    trace_text: &str,
    format: OutputFormat,
    secret: Option<&[u8]>,
) -> Result<String, CliError> {
    if trace_text.trim().is_empty() {
        return Err(CliError::input(
            "proto-decode requires trace input from proto-encode output",
        ));
    }

    let target = resolve_trace_target(target, auto_detect_target, trace_text)?;
    let runtime = runtime_for_target(target)?;
    let schemas = runtime.mapper.frame_schemas();
    let frames = parse_frames_from_trace(trace_text, &schemas)
        .map_err(|error| CliError::trace(format!("failed to parse trace frames: {error}")))?;

    if frames.is_empty() {
        return Err(CliError::trace("no frame lines were found in trace input"));
    }
    if let Some(sequence_error) = frame_sequence_error(&frames) {
        return Err(CliError::trace(format!(
            "invalid trace frame sequence: {sequence_error}"
        )));
    }

    let ordered_schemas = frames
        .iter()
        .map(|frame| schema_for_template(&schemas, &frame.template_id))
        .collect::<Result<Vec<_>, String>>()
        .map_err(|error| CliError::trace(format!("failed to resolve trace schemas: {error}")))?;

    let orchestration = map_domain(
        runtime
            .orchestrator()
            .with_symbolic_options(FixedWidthPlanningOptions::default())
            .orchestrate_decode(
                DecodeRequest {
                    stego_text: trace_text.to_string(),
                    options: runtime.pipeline_options().map_err(|error| {
                        CliError::config(format!("invalid pipeline options: {error}"))
                    })?,
                },
                &frames,
                &ordered_schemas,
            ),
        "decode orchestration failed",
    )?;
    let raw_payload = orchestration.payload;
    let payload = match secret {
        Some(secret) => open_payload(&raw_payload, secret)
            .map_err(|_| CliError::security("failed to decrypt payload; verify provided secret"))?,
        None => raw_payload,
    };
    let hex_payload = payload
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<Vec<_>>()
        .join("");

    let utf8_text = String::from_utf8(payload.clone()).ok();
    let gateway_response = orchestration.gateway_response.map(|item| item.content);

    if matches!(format, OutputFormat::Json) {
        return Ok(build_proto_decode_json(
            runtime.language_code,
            payload.len(),
            &hex_payload,
            utf8_text.as_deref(),
            gateway_response.as_deref(),
        ));
    }

    let mut report_lines = Vec::new();
    report_lines.push(format!("{} prototype decode", runtime.language_display));
    report_lines.push(format!("decoded bytes: {}", payload.len()));
    report_lines.push(format!("payload hex: {hex_payload}"));
    match utf8_text {
        Some(text) => report_lines.push(format!("payload utf8: {text}")),
        None => report_lines.push("payload utf8: <invalid utf8>".to_string()),
    }
    if let Some(gateway_response) = gateway_response {
        report_lines.push(format!("gateway response: {gateway_response}"));
    }

    Ok(report_lines.join("\n"))
}

fn resolve_encode_payload(options: &EncodeOptions) -> Result<String, CliError> {
    if let Some(input_path) = &options.input_path {
        let payload = std::fs::read_to_string(input_path)
            .map_err(|error| CliError::io("failed to read input file", Some(input_path), error))?;
        return Ok(payload);
    }
    if let Some(message) = &options.message {
        return Ok(message.clone());
    }
    Err(CliError::input(
        "encode payload source is missing (--message or --input is required)",
    ))
}

fn resolve_trace_input(trace: Option<&str>, input_path: Option<&str>) -> Result<String, CliError> {
    if let Some(input_path) = input_path {
        let trace_text = std::fs::read_to_string(input_path)
            .map_err(|error| CliError::io("failed to read trace file", Some(input_path), error))?;
        return Ok(trace_text);
    }
    if let Some(trace) = trace {
        return Ok(trace.to_string());
    }

    let mut buffer = String::new();
    std::io::stdin()
        .read_to_string(&mut buffer)
        .map_err(|error| CliError::io("failed to read trace from stdin", None, error))?;
    Ok(buffer)
}

fn resolve_required_secret_bytes(
    secret: Option<&str>,
    secret_file: Option<&str>,
    command: &str,
) -> Result<Vec<u8>, CliError> {
    let secret = resolve_optional_secret_bytes(secret, secret_file)?;
    match secret {
        Some(value) => Ok(value),
        None => Err(CliError::config(format!(
            "{command} requires --secret <value> or --secret-file <file> (or LSTEG_SECRET)"
        ))),
    }
}

fn resolve_optional_secret_bytes(
    secret: Option<&str>,
    secret_file: Option<&str>,
) -> Result<Option<Vec<u8>>, CliError> {
    if secret.is_some() && secret_file.is_some() {
        return Err(CliError::usage(
            "secret source is ambiguous; use either --secret or --secret-file".to_string(),
        ));
    }

    if let Some(secret) = secret {
        let normalized = secret.trim();
        if normalized.is_empty() {
            return Err(CliError::config("secret cannot be empty"));
        }
        return Ok(Some(normalized.as_bytes().to_vec()));
    }

    if let Some(secret_file) = secret_file {
        let file_value = std::fs::read_to_string(secret_file).map_err(|error| {
            CliError::io("failed to read secret file", Some(secret_file), error)
        })?;
        let normalized = file_value.trim_end_matches(['\r', '\n']);
        if normalized.trim().is_empty() {
            return Err(CliError::config(format!(
                "secret file '{}' is empty",
                secret_file
            )));
        }
        return Ok(Some(normalized.as_bytes().to_vec()));
    }

    Ok(None)
}

fn write_output(output: &str, output_path: Option<&str>) -> Result<(), CliError> {
    if let Some(path) = output_path {
        std::fs::write(path, output)
            .map_err(|error| CliError::io("failed to write output file", Some(path), error))?;
    } else {
        println!("{output}");
    }
    Ok(())
}

fn runtime_for_target(target: ProtoTarget) -> Result<PrototypeRuntime, CliError> {
    PrototypeRuntime::new(target).map_err(|error| {
        CliError::internal(format!(
            "failed to initialize {} runtime: {error}",
            target.as_str()
        ))
    })
}

fn map_domain<T, E>(result: Result<T, E>, context: &str) -> Result<T, CliError>
where
    E: Display,
{
    result.map_err(|error| CliError::domain(format!("{context}: {error}")))
}
