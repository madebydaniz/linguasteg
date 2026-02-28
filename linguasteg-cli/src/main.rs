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
use std::io::Read;

enum Command {
    Encode,
    Decode,
    Analyze,
    Demo(DemoTarget),
    ProtoEncode(ProtoTarget, String),
    ProtoDecode(ProtoTarget, Option<String>),
}

enum DemoTarget {
    Farsi,
}

enum ProtoTarget {
    Farsi,
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

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args().skip(1);
    let command = match args.next().as_deref() {
        Some("encode") => Command::Encode,
        Some("decode") => Command::Decode,
        Some("analyze") => Command::Analyze,
        Some("demo") => match args.next().as_deref() {
            Some("fa") => Command::Demo(DemoTarget::Farsi),
            _ => {
                eprintln!("demo target is required (supported: fa)");
                print_usage();
                return Ok(());
            }
        },
        Some("proto-encode") => match args.next().as_deref() {
            Some("fa") => {
                let message = args.collect::<Vec<_>>().join(" ");
                let payload_text = if message.trim().is_empty() {
                    "salam donya".to_string()
                } else {
                    message
                };
                Command::ProtoEncode(ProtoTarget::Farsi, payload_text)
            }
            _ => {
                eprintln!("proto-encode target is required (supported: fa)");
                print_usage();
                return Ok(());
            }
        },
        Some("proto-decode") => match args.next().as_deref() {
            Some("fa") => {
                let trace_input = args.collect::<Vec<_>>().join(" ");
                let trace = if trace_input.trim().is_empty() {
                    None
                } else {
                    Some(trace_input)
                };
                Command::ProtoDecode(ProtoTarget::Farsi, trace)
            }
            _ => {
                eprintln!("proto-decode target is required (supported: fa)");
                print_usage();
                return Ok(());
            }
        },
        _ => {
            print_usage();
            return Ok(());
        }
    };

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
        Command::ProtoEncode(ProtoTarget::Farsi, payload_text) => {
            run_farsi_proto_encode(&payload_text)?
        }
        Command::ProtoDecode(ProtoTarget::Farsi, trace_input) => {
            run_farsi_proto_decode(trace_input)?
        }
    }

    Ok(())
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

fn run_farsi_proto_encode(payload_text: &str) -> Result<(), Box<dyn std::error::Error>> {
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

    let mut sentences = Vec::with_capacity(realization_plans.len());
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
        println!(
            "frame {:02} [{}] bits={}..{} values={} => {}",
            index + 1,
            plan.template_id,
            payload_plan.frames[index].source.start_bit,
            payload_plan.frames[index].source.start_bit
                + payload_plan.frames[index].source.consumed_bits,
            symbol_values,
            sentence
        );
        sentences.push(sentence);
    }

    println!();
    println!("final prototype text:");
    println!("{}.", sentences.join(". "));

    if let Some(gateway_response) = orchestration.gateway_response {
        println!("gateway response: {}", gateway_response.content);
    }

    Ok(())
}

fn run_farsi_proto_decode(trace_input: Option<String>) -> Result<(), Box<dyn std::error::Error>> {
    let trace_text = match trace_input {
        Some(value) => value,
        None => {
            let mut buffer = String::new();
            std::io::stdin().read_to_string(&mut buffer)?;
            buffer
        }
    };

    if trace_text.trim().is_empty() {
        eprintln!("proto-decode requires trace input from proto-encode output");
        return Ok(());
    }

    let runtime = FarsiProtoRuntime::new()?;
    let schemas = runtime.mapper.frame_schemas();
    let frames = parse_frames_from_trace(&trace_text, &schemas)?;

    if frames.is_empty() {
        eprintln!("no frame lines were found in trace input");
        return Ok(());
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

    println!("Farsi prototype decode");
    println!("decoded bytes: {}", payload.len());
    println!("payload hex: {hex_payload}");
    match String::from_utf8(payload.clone()) {
        Ok(text) => println!("payload utf8: {text}"),
        Err(_) => println!("payload utf8: <invalid utf8>"),
    }
    if let Some(gateway_response) = orchestration.gateway_response {
        println!("gateway response: {}", gateway_response.content);
    }

    Ok(())
}

fn parse_frames_from_trace(
    trace_text: &str,
    schemas: &[SymbolicFrameSchema],
) -> Result<Vec<SymbolicFramePlan>, Box<dyn std::error::Error>> {
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
    if values_section.trim().is_empty() {
        return Err("trace values section is empty".into());
    }

    let mut parsed = HashMap::new();
    for part in values_section.split(',') {
        let trimmed = part.trim();
        if trimmed.is_empty() {
            continue;
        }

        let (slot, value_raw) = trimmed
            .split_once(':')
            .ok_or_else(|| format!("malformed symbolic value pair: '{trimmed}'"))?;
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

fn print_usage() {
    println!("LinguaSteg CLI (scaffold)");
    println!("Usage: lsteg <encode|decode|analyze|demo|proto-encode|proto-decode>");
    println!("       lsteg demo fa");
    println!("       lsteg proto-encode fa [message]");
    println!("       lsteg proto-encode fa [message] | lsteg proto-decode fa");
    println!("       lsteg proto-decode fa \"<frame trace lines>\"");
}
