use linguasteg_core::{
    BitRange, DecodeRequest, EncodeRequest, FixedWidthBitPlanner, FixedWidthPlanningOptions,
    GrammarConstraintChecker, LanguageRealizer, LanguageTag, ModelCapability, ModelDescriptor,
    ModelId, ModelRegistry, ModelSelection, PipelineOptions, PipelineOrchestrator, ProviderId,
    RealizationPlan, SlotAssignment, SlotId, StrategyDescriptor, StrategyId, StrategyRegistry,
    StyleProfileRegistry, SymbolicFramePlan, SymbolicFrameSchema, SymbolicSlotValue, TemplateId,
    TemplateRegistry,
};
use linguasteg_models::{
    FarsiPrototypeConstraintChecker, FarsiPrototypeLanguagePack, FarsiPrototypeRealizer,
    FarsiPrototypeSymbolicMapper, InMemoryGatewayRegistry,
};
use std::collections::HashMap;
use std::io::{Read, Write};
use std::process::ExitCode;

type DynError = Box<dyn std::error::Error>;

enum Command {
    Encode,
    Decode,
    Analyze,
    Demo(DemoTarget),
    ProtoEncode(ProtoTarget, String, bool),
    ProtoDecode(ProtoTarget, Option<String>, bool),
}

enum DemoTarget {
    Farsi,
}

enum ProtoTarget {
    Farsi,
}

enum CliError {
    Usage(String),
}

struct InMemoryStrategyRegistry {
    strategies: Vec<StrategyDescriptor>,
}

impl StrategyRegistry for InMemoryStrategyRegistry {
    fn all_strategies(&self) -> &[StrategyDescriptor] {
        &self.strategies
    }
}

struct InMemoryModelRegistry {
    models: Vec<ModelDescriptor>,
}

impl ModelRegistry for InMemoryModelRegistry {
    fn all_models(&self) -> &[ModelDescriptor] {
        &self.models
    }
}

struct FarsiProtoRuntime {
    pack: FarsiPrototypeLanguagePack,
    checker: FarsiPrototypeConstraintChecker,
    realizer: FarsiPrototypeRealizer,
    mapper: FarsiPrototypeSymbolicMapper,
    planner: FixedWidthBitPlanner,
    strategy_registry: InMemoryStrategyRegistry,
    model_registry: InMemoryModelRegistry,
    gateway_registry: InMemoryGatewayRegistry,
}

impl FarsiProtoRuntime {
    fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let strategy_id = StrategyId::new("symbolic-stub")?;
        let provider = ProviderId::new("stub")?;
        let model = ModelId::new("stub-local")?;
        let fa = LanguageTag::new("fa")?;

        Ok(Self {
            pack: FarsiPrototypeLanguagePack::default(),
            checker: FarsiPrototypeConstraintChecker,
            realizer: FarsiPrototypeRealizer,
            mapper: FarsiPrototypeSymbolicMapper,
            planner: FixedWidthBitPlanner::default(),
            strategy_registry: InMemoryStrategyRegistry {
                strategies: vec![StrategyDescriptor {
                    id: strategy_id,
                    display_name: "Symbolic Stub".to_string(),
                    required_capabilities: vec![ModelCapability::DeterministicSeed],
                }],
            },
            model_registry: InMemoryModelRegistry {
                models: vec![ModelDescriptor {
                    provider,
                    model,
                    display_name: "Stub Local".to_string(),
                    supported_languages: vec![fa],
                    capabilities: vec![ModelCapability::DeterministicSeed],
                }],
            },
            gateway_registry: InMemoryGatewayRegistry::with_stub(),
        })
    }

    fn orchestrator(&self) -> PipelineOrchestrator<'_> {
        PipelineOrchestrator::new(
            &self.pack,
            &self.strategy_registry,
            &self.model_registry,
            &self.gateway_registry,
            &self.planner,
        )
    }

    fn pipeline_options(&self) -> Result<PipelineOptions, Box<dyn std::error::Error>> {
        Ok(PipelineOptions {
            language: LanguageTag::new("fa")?,
            strategy: StrategyId::new("symbolic-stub")?,
            model_selection: Some(ModelSelection {
                provider: ProviderId::new("stub")?,
                model: ModelId::new("stub-local")?,
            }),
        })
    }
}

fn main() -> ExitCode {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    let command = match parse_command(args) {
        Ok(Some(command)) => command,
        Ok(None) => {
            let _ = write_usage(std::io::stdout());
            return ExitCode::SUCCESS;
        }
        Err(CliError::Usage(message)) => {
            eprintln!("{message}");
            let _ = write_usage(std::io::stderr());
            return ExitCode::from(2);
        }
    };

    match execute(command) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("error: {error}");
            ExitCode::from(1)
        }
    }
}

fn execute(command: Command) -> Result<(), DynError> {
    match command {
        Command::Encode => {
            let language = LanguageTag::new("en")?;
            let strategy = StrategyId::new("synonym")?;
            println!("encode scaffold: language={language} strategy={strategy}");
        }
        Command::Decode => {
            let language = LanguageTag::new("en")?;
            let strategy = StrategyId::new("synonym")?;
            println!("decode scaffold: language={language} strategy={strategy}");
        }
        Command::Analyze => {
            println!("analyze scaffold");
        }
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

fn parse_command(args: Vec<String>) -> Result<Option<Command>, CliError> {
    let mut args = args.into_iter();
    let Some(command) = args.next() else {
        return Ok(None);
    };

    if command == "--help" || command == "-h" {
        return Ok(None);
    }

    match command.as_str() {
        "encode" => Ok(Some(Command::Encode)),
        "decode" => Ok(Some(Command::Decode)),
        "analyze" => Ok(Some(Command::Analyze)),
        "demo" => parse_demo_command(args),
        "proto-encode" => parse_proto_encode_command(args),
        "proto-decode" => parse_proto_decode_command(args),
        _ => Err(CliError::Usage(format!("unknown command: {command}"))),
    }
}

fn parse_demo_command(mut args: impl Iterator<Item = String>) -> Result<Option<Command>, CliError> {
    match args.next().as_deref() {
        Some("fa") => Ok(Some(Command::Demo(DemoTarget::Farsi))),
        _ => Err(CliError::Usage(
            "demo target is required (supported: fa)".to_string(),
        )),
    }
}

fn parse_proto_encode_command(
    args: impl Iterator<Item = String>,
) -> Result<Option<Command>, CliError> {
    let mut args = args.collect::<Vec<_>>();
    if args.first().map(String::as_str) != Some("fa") {
        return Err(CliError::Usage(
            "proto-encode target is required (supported: fa)".to_string(),
        ));
    }

    args.remove(0);
    let json = take_flag(&mut args, "--json");
    let message = args.join(" ");
    let payload_text = if message.trim().is_empty() {
        "salam donya".to_string()
    } else {
        message
    };

    Ok(Some(Command::ProtoEncode(
        ProtoTarget::Farsi,
        payload_text,
        json,
    )))
}

fn parse_proto_decode_command(
    args: impl Iterator<Item = String>,
) -> Result<Option<Command>, CliError> {
    let mut args = args.collect::<Vec<_>>();
    if args.first().map(String::as_str) != Some("fa") {
        return Err(CliError::Usage(
            "proto-decode target is required (supported: fa)".to_string(),
        ));
    }

    args.remove(0);
    let json = take_flag(&mut args, "--json");
    let trace_input = args.join(" ");
    let trace = if trace_input.trim().is_empty() {
        None
    } else {
        Some(trace_input)
    };

    Ok(Some(Command::ProtoDecode(ProtoTarget::Farsi, trace, json)))
}

fn run_farsi_demo() -> Result<(), Box<dyn std::error::Error>> {
    let pack = FarsiPrototypeLanguagePack::default();
    let checker = FarsiPrototypeConstraintChecker;
    let realizer = FarsiPrototypeRealizer;

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

fn assignment(slot: &str, surface: &str) -> Result<SlotAssignment, Box<dyn std::error::Error>> {
    Ok(SlotAssignment {
        slot: SlotId::new(slot)?,
        surface: surface.to_string(),
        lemma: None,
    })
}

fn run_farsi_proto_encode(
    payload_text: &str,
    json: bool,
) -> Result<(), Box<dyn std::error::Error>> {
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

    if json {
        println!(
            "{}",
            build_proto_encode_json(
                payload_text,
                payload.len(),
                payload_plan.encoded_len_bytes,
                payload_plan.padding_bits,
                &payload_plan.frames,
                &sentences,
                &final_text,
                gateway_response.as_deref(),
            )
        );
        return Ok(());
    }

    println!("Farsi prototype encode");
    println!("input text: {}", payload_text);
    println!("payload bytes: {}", payload.len());
    println!(
        "encoded bytes (with length prefix): {}",
        payload_plan.encoded_len_bytes
    );
    println!("frames: {}", payload_plan.frames.len());
    println!("padding bits: {}", payload_plan.padding_bits);
    println!();
    for line in frame_lines {
        println!("{line}");
    }
    println!();
    println!("final prototype text:");
    println!("{final_text}");

    if let Some(gateway_response) = gateway_response {
        println!("gateway response: {gateway_response}");
    }

    Ok(())
}

fn run_farsi_proto_decode(
    trace_input: Option<String>,
    json: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let trace_text = match trace_input {
        Some(value) => value,
        None => {
            let mut buffer = String::new();
            std::io::stdin().read_to_string(&mut buffer)?;
            buffer
        }
    };

    if trace_text.trim().is_empty() {
        return Err("proto-decode requires trace input from proto-encode output".into());
    }

    let runtime = FarsiProtoRuntime::new()?;
    let schemas = runtime.mapper.frame_schemas();
    let frames = parse_frames_from_trace(&trace_text, &schemas)?;

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
                stego_text: trace_text.clone(),
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

    if json {
        println!(
            "{}",
            build_proto_decode_json(
                payload.len(),
                &hex_payload,
                utf8_text.as_deref(),
                gateway_response.as_deref(),
            )
        );
        return Ok(());
    }

    println!("Farsi prototype decode");
    println!("decoded bytes: {}", payload.len());
    println!("payload hex: {hex_payload}");
    match utf8_text {
        Some(text) => println!("payload utf8: {text}"),
        None => println!("payload utf8: <invalid utf8>"),
    }
    if let Some(gateway_response) = gateway_response {
        println!("gateway response: {gateway_response}");
    }

    Ok(())
}

fn parse_frames_from_trace(
    trace_text: &str,
    schemas: &[SymbolicFrameSchema],
) -> Result<Vec<SymbolicFramePlan>, Box<dyn std::error::Error>> {
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

fn parse_frames_from_proto_encode_json(
    json_text: &str,
    schemas: &[SymbolicFrameSchema],
) -> Result<Option<Vec<SymbolicFramePlan>>, Box<dyn std::error::Error>> {
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
) -> Result<SymbolicFramePlan, Box<dyn std::error::Error>> {
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

fn extract_template_id(line: &str) -> Result<TemplateId, Box<dyn std::error::Error>> {
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

fn extract_bit_range(line: &str) -> Result<(usize, usize), Box<dyn std::error::Error>> {
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

fn extract_values_section(line: &str) -> Result<&str, Box<dyn std::error::Error>> {
    let values_label_index = line
        .find("values=")
        .ok_or_else(|| "trace line missing values section".to_string())?;
    let values_start = values_label_index + "values=".len();
    let values_tail = &line[values_start..];
    let values_end = values_tail.find(" =>").unwrap_or(values_tail.len());
    Ok(&values_tail[..values_end])
}

fn parse_value_map(
    values_section: &str,
) -> Result<HashMap<String, u32>, Box<dyn std::error::Error>> {
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

fn schema_for_template(
    schemas: &[SymbolicFrameSchema],
    template_id: &TemplateId,
) -> Result<SymbolicFrameSchema, String> {
    schemas
        .iter()
        .find(|schema| schema.template_id == *template_id)
        .cloned()
        .ok_or_else(|| format!("no schema found for template '{template_id}'"))
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

fn extract_balanced_section(
    input: &str,
    open_char: char,
    close_char: char,
) -> Option<&str> {
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

fn split_json_objects(array_content: &str) -> Result<Vec<&str>, Box<dyn std::error::Error>> {
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

fn take_flag(parts: &mut Vec<String>, flag: &str) -> bool {
    let mut found = false;
    parts.retain(|item| {
        if item == flag {
            found = true;
            false
        } else {
            true
        }
    });
    found
}

fn build_proto_encode_json(
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
        "{{\"mode\":\"proto-encode\",\"language\":\"fa\",\"input_text\":\"{}\",\"payload_bytes\":{},\"encoded_bytes\":{},\"frame_count\":{},\"padding_bits\":{},\"frames\":[{}],\"final_text\":\"{}\",\"gateway_response\":{}}}",
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

fn build_proto_decode_json(
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
        "{{\"mode\":\"proto-decode\",\"language\":\"fa\",\"decoded_bytes\":{},\"payload_hex\":\"{}\",\"payload_utf8\":{},\"gateway_response\":{}}}",
        decoded_bytes,
        json_escape(payload_hex),
        utf8_json,
        gateway_json
    )
}

fn json_escape(input: &str) -> String {
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

fn write_usage(mut writer: impl Write) -> std::io::Result<()> {
    writeln!(writer, "LinguaSteg CLI (scaffold)")?;
    writeln!(
        writer,
        "Usage: lsteg <encode|decode|analyze|demo|proto-encode|proto-decode>"
    )?;
    writeln!(writer, "       lsteg demo fa")?;
    writeln!(writer, "       lsteg proto-encode fa [message] [--json]")?;
    writeln!(
        writer,
        "       lsteg proto-encode fa [message] | lsteg proto-decode fa [--json]"
    )?;
    writeln!(
        writer,
        "       lsteg proto-decode fa \"<frame trace lines>\" [--json]"
    )?;
    Ok(())
}
