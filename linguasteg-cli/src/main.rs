use linguasteg_core::{
    GrammarConstraintChecker, LanguageRealizer, LanguageTag, RealizationPlan, SlotAssignment,
    SlotId, StrategyId, StyleProfileRegistry, TemplateId, TemplateRegistry,
};
use linguasteg_models::{
    FarsiPrototypeConstraintChecker, FarsiPrototypeLanguagePack, FarsiPrototypeRealizer,
};

enum Command {
    Encode,
    Decode,
    Analyze,
    Demo(DemoTarget),
}

enum DemoTarget {
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

fn print_usage() {
    println!("LinguaSteg CLI (scaffold)");
    println!("Usage: lsteg <encode|decode|analyze|demo>");
    println!("       lsteg demo fa");
}
