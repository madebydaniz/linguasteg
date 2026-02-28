use linguasteg_core::{
    FixedWidthBitPlanner, GrammarConstraintChecker, LanguageRealizer, LanguageTag, RealizationPlan,
    SlotAssignment, SlotId, StrategyId, StyleProfileRegistry, SymbolicPayloadPlanner, TemplateId,
    TemplateRegistry,
};
use linguasteg_models::{
    FarsiPrototypeConstraintChecker, FarsiPrototypeLanguagePack, FarsiPrototypeRealizer,
    FarsiPrototypeSymbolicMapper,
};

enum Command {
    Encode,
    Decode,
    Analyze,
    Demo(DemoTarget),
    ProtoEncode(ProtoTarget, String),
}

enum DemoTarget {
    Farsi,
}

enum ProtoTarget {
    Farsi,
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
    let pack = FarsiPrototypeLanguagePack::default();
    let checker = FarsiPrototypeConstraintChecker;
    let realizer = FarsiPrototypeRealizer;
    let mapper = FarsiPrototypeSymbolicMapper;
    let planner = FixedWidthBitPlanner::default();
    let schemas = mapper.frame_schemas();

    let payload_plan = planner.plan_payload(payload, &schemas)?;
    let realization_plans = mapper.map_payload_to_plans(&payload_plan)?;

    println!("Farsi prototype encode");
    println!("input text: {}", payload_text);
    println!("payload bytes: {}", payload.len());
    println!("encoded bytes (with length prefix): {}", payload_plan.encoded_len_bytes);
    println!("frames: {}", payload_plan.frames.len());
    println!("padding bits: {}", payload_plan.padding_bits);
    println!();

    let mut sentences = Vec::with_capacity(realization_plans.len());
    for (index, plan) in realization_plans.iter().enumerate() {
        let template = pack
            .template(&plan.template_id)
            .ok_or_else(|| format!("missing template: {}", plan.template_id))?;
        checker.validate_plan(template, plan)?;
        let sentence = realizer.render(template, plan)?;
        println!(
            "frame {:02} [{}] bits={}..{} => {}",
            index + 1,
            plan.template_id,
            payload_plan.frames[index].source.start_bit,
            payload_plan.frames[index].source.start_bit + payload_plan.frames[index].source.consumed_bits,
            sentence
        );
        sentences.push(sentence);
    }

    println!();
    println!("final prototype text:");
    println!("{}.", sentences.join(". "));
    Ok(())
}

fn print_usage() {
    println!("LinguaSteg CLI (scaffold)");
    println!("Usage: lsteg <encode|decode|analyze|demo|proto-encode>");
    println!("       lsteg demo fa");
    println!("       lsteg proto-encode fa [message]");
}
