use linguasteg_core::{LanguageTag, StrategyId};

enum Command {
    Encode,
    Decode,
    Analyze,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args().skip(1);
    let command = match args.next().as_deref() {
        Some("encode") => Command::Encode,
        Some("decode") => Command::Decode,
        Some("analyze") => Command::Analyze,
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
    }

    Ok(())
}

fn print_usage() {
    println!("LinguaSteg CLI (scaffold)");
    println!("Usage: lsteg <encode|decode|analyze>");
}
