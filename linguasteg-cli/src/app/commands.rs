use std::io::Read;

use linguasteg_core::{
    DecodeRequest, EncodeRequest, FixedWidthPlanningOptions, GrammarConstraintChecker,
    LanguageRealizer, LanguageTag, RealizationPlan, SlotAssignment, SlotId, StyleProfileRegistry,
    TemplateId, TemplateRegistry,
};

use super::analysis::render_farsi_trace_analysis_output;
use super::formatters::{build_proto_decode_json, build_proto_encode_json};
use super::runtime::FarsiProtoRuntime;
use super::trace::{parse_frames_from_trace, schema_for_template};
use super::types::{
    AnalyzeOptions, Command, DecodeOptions, DemoTarget, DynError, EncodeOptions, OutputFormat,
    ProtoTarget,
};

pub(crate) fn execute(command: Command) -> Result<(), DynError> {
    match command {
        Command::Encode(options) => run_encode(options)?,
        Command::Decode(options) => run_decode(options)?,
        Command::Analyze(options) => run_analyze(options)?,
        Command::Demo(DemoTarget::Farsi) => run_farsi_demo()?,
        Command::ProtoEncode(ProtoTarget::Farsi, payload_text, json) => {
            run_farsi_proto_encode(&payload_text, json)?
        }
        Command::ProtoDecode(ProtoTarget::Farsi, trace_input, json) => {
            run_farsi_proto_decode(trace_input, json)?
        }
    }

    Ok(())
}

fn run_encode(options: EncodeOptions) -> Result<(), DynError> {
    let payload_text = resolve_encode_payload(&options)?;
    let output = match options.target {
        ProtoTarget::Farsi => render_farsi_proto_encode_output(&payload_text, options.format)?,
    };
    write_output(&output, options.output_path.as_deref())
}

fn run_decode(options: DecodeOptions) -> Result<(), DynError> {
    let trace_text = resolve_trace_input(options.trace.as_deref(), options.input_path.as_deref())?;
    let output = match options.target {
        ProtoTarget::Farsi => render_farsi_proto_decode_output(&trace_text, options.format)?,
    };
    write_output(&output, options.output_path.as_deref())
}

fn run_analyze(options: AnalyzeOptions) -> Result<(), DynError> {
    let trace_text = resolve_trace_input(options.trace.as_deref(), options.input_path.as_deref())?;
    let output = match options.target {
        ProtoTarget::Farsi => render_farsi_trace_analysis_output(&trace_text, options.format)?,
    };
    write_output(&output, options.output_path.as_deref())
}

fn resolve_encode_payload(options: &EncodeOptions) -> Result<String, DynError> {
    if let Some(input_path) = &options.input_path {
        let payload = std::fs::read_to_string(input_path)?;
        return Ok(payload);
    }
    if let Some(message) = &options.message {
        return Ok(message.clone());
    }
    Err("encode payload source is missing".into())
}

fn resolve_trace_input(trace: Option<&str>, input_path: Option<&str>) -> Result<String, DynError> {
    if let Some(input_path) = input_path {
        let trace_text = std::fs::read_to_string(input_path)?;
        return Ok(trace_text);
    }
    if let Some(trace) = trace {
        return Ok(trace.to_string());
    }

    let mut buffer = String::new();
    std::io::stdin().read_to_string(&mut buffer)?;
    Ok(buffer)
}

fn write_output(output: &str, output_path: Option<&str>) -> Result<(), DynError> {
    if let Some(path) = output_path {
        std::fs::write(path, output)?;
    } else {
        println!("{output}");
    }
    Ok(())
}

fn run_farsi_demo() -> Result<(), DynError> {
    let pack = linguasteg_models::FarsiPrototypeLanguagePack::default();
    let checker = linguasteg_models::FarsiPrototypeConstraintChecker;
    let realizer = linguasteg_models::FarsiPrototypeRealizer;

    let fa = LanguageTag::new("fa")?;
    let templates = pack.templates_for_language(&fa);
    let style_profiles = pack.style_profiles_for_language(&fa);

    println!("Farsi prototype demo");
    println!("language: {fa}");
    println!("templates: {}", templates.len());
    for template in &templates {
        println!("  - {} ({})", template.id, template.display_name);
    }
    println!("style profiles: {}", style_profiles.len());
    for profile in &style_profiles {
        println!("  - {} ({})", profile.id, profile.display_name);
    }

    let template_id = TemplateId::new("fa-time-location-sov")?;
    let template = pack
        .template(&template_id)
        .ok_or_else(|| format!("missing template: {template_id}"))?;

    let valid_plan = RealizationPlan {
        template_id: template_id.clone(),
        assignments: vec![
            assignment("subject", "دانشجو")?,
            assignment("time", "امروز")?,
            assignment("location", "کتابخانه")?,
            assignment("object", "نامه")?,
            assignment("verb", "نوشت")?,
        ],
    };

    checker.validate_plan(template, &valid_plan)?;
    let rendered = realizer.render(template, &valid_plan)?;
    println!();
    println!("valid render:");
    println!("  {rendered}");

    let invalid_plan = RealizationPlan {
        template_id,
        assignments: vec![
            assignment("subject", "دانشجو")?,
            assignment("time", "امروز")?,
            assignment("location", "کتابخانه")?,
            assignment("object", "کتاب")?,
            assignment("verb", "نوشید")?,
        ],
    };

    println!();
    println!("invalid semantic combo demo:");
    match checker.validate_plan(template, &invalid_plan) {
        Ok(()) => println!("  unexpected: invalid plan was accepted"),
        Err(error) => println!("  rejected as expected: {error}"),
    }

    Ok(())
}

fn assignment(slot: &str, surface: &str) -> Result<SlotAssignment, DynError> {
    Ok(SlotAssignment {
        slot: SlotId::new(slot)?,
        surface: surface.to_string(),
        lemma: None,
    })
}

fn run_farsi_proto_encode(payload_text: &str, json: bool) -> Result<(), DynError> {
    let format = if json {
        OutputFormat::Json
    } else {
        OutputFormat::Text
    };
    let output = render_farsi_proto_encode_output(payload_text, format)?;
    println!("{output}");
    Ok(())
}

fn render_farsi_proto_encode_output(
    payload_text: &str,
    format: OutputFormat,
) -> Result<String, DynError> {
    let payload = payload_text.as_bytes();
    let runtime = FarsiProtoRuntime::new()?;
    let schemas = runtime.mapper.frame_schemas();
    let orchestration = runtime.orchestrator().orchestrate_encode(
        EncodeRequest {
            carrier_text: payload_text.to_string(),
            payload: payload.to_vec(),
            options: runtime.pipeline_options()?,
        },
        &schemas,
    )?;
    let payload_plan = orchestration.symbolic_plan;
    let realization_plans = runtime.mapper.map_payload_to_plans(&payload_plan)?;

    let mut sentences = Vec::with_capacity(realization_plans.len());
    let mut frame_lines = Vec::with_capacity(realization_plans.len());
    for (index, plan) in realization_plans.iter().enumerate() {
        let template = runtime
            .pack
            .template(&plan.template_id)
            .ok_or_else(|| format!("missing template: {}", plan.template_id))?;
        runtime.checker.validate_plan(template, plan)?;
        let sentence = runtime.realizer.render(template, plan)?;
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
    report_lines.push("Farsi prototype encode".to_string());
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

fn run_farsi_proto_decode(trace_input: Option<String>, json: bool) -> Result<(), DynError> {
    let trace_text = match trace_input {
        Some(value) => value,
        None => {
            let mut buffer = String::new();
            std::io::stdin().read_to_string(&mut buffer)?;
            buffer
        }
    };

    let format = if json {
        OutputFormat::Json
    } else {
        OutputFormat::Text
    };
    let output = render_farsi_proto_decode_output(&trace_text, format)?;
    println!("{output}");
    Ok(())
}

fn render_farsi_proto_decode_output(
    trace_text: &str,
    format: OutputFormat,
) -> Result<String, DynError> {
    if trace_text.trim().is_empty() {
        return Err("proto-decode requires trace input from proto-encode output".into());
    }

    let runtime = FarsiProtoRuntime::new()?;
    let schemas = runtime.mapper.frame_schemas();
    let frames = parse_frames_from_trace(trace_text, &schemas)?;

    if frames.is_empty() {
        return Err("no frame lines were found in trace input".into());
    }

    let ordered_schemas = frames
        .iter()
        .map(|frame| schema_for_template(&schemas, &frame.template_id))
        .collect::<Result<Vec<_>, String>>()?;

    let orchestration = runtime
        .orchestrator()
        .with_symbolic_options(FixedWidthPlanningOptions::default())
        .orchestrate_decode(
            DecodeRequest {
                stego_text: trace_text.to_string(),
                options: runtime.pipeline_options()?,
            },
            &frames,
            &ordered_schemas,
        )?;
    let payload = orchestration.payload;
    let hex_payload = payload
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<Vec<_>>()
        .join("");

    let utf8_text = String::from_utf8(payload.clone()).ok();
    let gateway_response = orchestration.gateway_response.map(|item| item.content);

    if matches!(format, OutputFormat::Json) {
        return Ok(build_proto_decode_json(
            payload.len(),
            &hex_payload,
            utf8_text.as_deref(),
            gateway_response.as_deref(),
        ));
    }

    let mut report_lines = Vec::new();
    report_lines.push("Farsi prototype decode".to_string());
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
