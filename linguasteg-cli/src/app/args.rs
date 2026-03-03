use std::io::Write;

use super::types::{
    AnalyzeOptions, CliError, Command, DecodeOptions, DemoTarget, EncodeOptions, OutputFormat,
    ProfileQueryOptions, ProtoTarget, TemplateQueryOptions,
};

pub(crate) fn parse_command(args: Vec<String>) -> Result<Option<Command>, CliError> {
    let mut args = args.into_iter();
    let Some(command) = args.next() else {
        return Ok(None);
    };

    if command == "--help" || command == "-h" {
        return Ok(None);
    }

    match command.as_str() {
        "encode" => parse_encode_command(args),
        "decode" => parse_decode_command(args),
        "analyze" => parse_analyze_command(args),
        "languages" => parse_languages_command(args),
        "strategies" => parse_strategies_command(args),
        "models" => parse_models_command(args),
        "catalog" => parse_catalog_command(args),
        "templates" => parse_templates_command(args),
        "profiles" => parse_profiles_command(args),
        "demo" => parse_demo_command(args),
        "proto-encode" => parse_proto_encode_command(args),
        "proto-decode" => parse_proto_decode_command(args),
        _ => Err(CliError::usage(format!("unknown command: {command}"))),
    }
}

fn parse_encode_command(
    mut args: impl Iterator<Item = String>,
) -> Result<Option<Command>, CliError> {
    let mut lang = env_proto_target("LSTEG_LANG")?.unwrap_or(ProtoTarget::Farsi);
    let mut format = env_output_format("LSTEG_FORMAT")?.unwrap_or(OutputFormat::Text);
    let mut message = env_optional("LSTEG_ENCODE_MESSAGE");
    let mut input_path = env_optional("LSTEG_INPUT");
    let mut output_path = env_optional("LSTEG_OUTPUT");
    let mut secret = env_optional("LSTEG_SECRET");
    let mut secret_file = None;

    let mut seen_message = false;
    let mut seen_input = false;
    let mut seen_output = false;
    let mut seen_secret = false;
    let mut seen_secret_file = false;

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--help" | "-h" => return Ok(None),
            "--lang" => {
                let value = next_arg_value(&mut args, "--lang")?;
                lang = parse_proto_target(&value)?;
            }
            "--format" => {
                let value = next_arg_value(&mut args, "--format")?;
                format = parse_output_format(&value)?;
            }
            "--message" => {
                if seen_message {
                    return Err(CliError::usage(
                        "--message cannot be provided multiple times".to_string(),
                    ));
                }
                seen_message = true;
                message = Some(next_arg_value(&mut args, "--message")?);
            }
            "--input" => {
                if seen_input {
                    return Err(CliError::usage(
                        "--input cannot be provided multiple times".to_string(),
                    ));
                }
                seen_input = true;
                input_path = Some(next_arg_value(&mut args, "--input")?);
            }
            "--output" => {
                if seen_output {
                    return Err(CliError::usage(
                        "--output cannot be provided multiple times".to_string(),
                    ));
                }
                seen_output = true;
                output_path = Some(next_arg_value(&mut args, "--output")?);
            }
            "--secret" => {
                if seen_secret {
                    return Err(CliError::usage(
                        "--secret cannot be provided multiple times".to_string(),
                    ));
                }
                if seen_secret_file {
                    return Err(CliError::usage(
                        "encode accepts either --secret or --secret-file, not both".to_string(),
                    ));
                }
                seen_secret = true;
                secret = Some(next_arg_value(&mut args, "--secret")?);
                secret_file = None;
            }
            "--secret-file" => {
                if seen_secret_file {
                    return Err(CliError::usage(
                        "--secret-file cannot be provided multiple times".to_string(),
                    ));
                }
                if seen_secret {
                    return Err(CliError::usage(
                        "encode accepts either --secret or --secret-file, not both".to_string(),
                    ));
                }
                seen_secret_file = true;
                secret_file = Some(next_arg_value(&mut args, "--secret-file")?);
                secret = None;
            }
            _ => {
                return Err(CliError::usage(format!("unknown encode argument: {arg}")));
            }
        }
    }

    if message.is_some() && input_path.is_some() {
        return Err(CliError::usage(
            "encode accepts either --message or --input, not both".to_string(),
        ));
    }
    if message.is_none() && input_path.is_none() {
        return Err(CliError::usage(
            "encode requires --message <text> or --input <file>".to_string(),
        ));
    }
    if secret.is_some() && secret_file.is_some() {
        return Err(CliError::usage(
            "encode accepts either --secret or --secret-file, not both".to_string(),
        ));
    }

    Ok(Some(Command::Encode(EncodeOptions {
        target: lang,
        message,
        input_path,
        output_path,
        secret,
        secret_file,
        format,
    })))
}

fn parse_decode_command(
    mut args: impl Iterator<Item = String>,
) -> Result<Option<Command>, CliError> {
    let (mut lang, mut auto_detect_target) =
        env_trace_proto_target("LSTEG_LANG")?.unwrap_or((ProtoTarget::Farsi, true));
    let mut format = env_output_format("LSTEG_FORMAT")?.unwrap_or(OutputFormat::Text);
    let mut trace = env_optional("LSTEG_TRACE");
    let mut input_path = env_optional("LSTEG_INPUT");
    let mut output_path = env_optional("LSTEG_OUTPUT");
    let mut secret = env_optional("LSTEG_SECRET");
    let mut secret_file = None;

    let mut seen_trace = false;
    let mut seen_input = false;
    let mut seen_output = false;
    let mut seen_secret = false;
    let mut seen_secret_file = false;

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--help" | "-h" => return Ok(None),
            "--lang" => {
                let value = next_arg_value(&mut args, "--lang")?;
                let (resolved_lang, resolved_auto_detect) = parse_trace_proto_target(&value)?;
                lang = resolved_lang;
                auto_detect_target = resolved_auto_detect;
            }
            "--format" => {
                let value = next_arg_value(&mut args, "--format")?;
                format = parse_output_format(&value)?;
            }
            "--trace" => {
                if seen_trace {
                    return Err(CliError::usage(
                        "--trace cannot be provided multiple times".to_string(),
                    ));
                }
                seen_trace = true;
                trace = Some(next_arg_value(&mut args, "--trace")?);
            }
            "--input" => {
                if seen_input {
                    return Err(CliError::usage(
                        "--input cannot be provided multiple times".to_string(),
                    ));
                }
                seen_input = true;
                input_path = Some(next_arg_value(&mut args, "--input")?);
            }
            "--output" => {
                if seen_output {
                    return Err(CliError::usage(
                        "--output cannot be provided multiple times".to_string(),
                    ));
                }
                seen_output = true;
                output_path = Some(next_arg_value(&mut args, "--output")?);
            }
            "--secret" => {
                if seen_secret {
                    return Err(CliError::usage(
                        "--secret cannot be provided multiple times".to_string(),
                    ));
                }
                if seen_secret_file {
                    return Err(CliError::usage(
                        "decode accepts either --secret or --secret-file, not both".to_string(),
                    ));
                }
                seen_secret = true;
                secret = Some(next_arg_value(&mut args, "--secret")?);
                secret_file = None;
            }
            "--secret-file" => {
                if seen_secret_file {
                    return Err(CliError::usage(
                        "--secret-file cannot be provided multiple times".to_string(),
                    ));
                }
                if seen_secret {
                    return Err(CliError::usage(
                        "decode accepts either --secret or --secret-file, not both".to_string(),
                    ));
                }
                seen_secret_file = true;
                secret_file = Some(next_arg_value(&mut args, "--secret-file")?);
                secret = None;
            }
            _ => {
                return Err(CliError::usage(format!("unknown decode argument: {arg}")));
            }
        }
    }

    if trace.is_some() && input_path.is_some() {
        return Err(CliError::usage(
            "decode accepts either --trace or --input, not both".to_string(),
        ));
    }
    if secret.is_some() && secret_file.is_some() {
        return Err(CliError::usage(
            "decode accepts either --secret or --secret-file, not both".to_string(),
        ));
    }

    Ok(Some(Command::Decode(DecodeOptions {
        target: lang,
        auto_detect_target,
        trace,
        input_path,
        output_path,
        secret,
        secret_file,
        format,
    })))
}

fn parse_analyze_command(
    mut args: impl Iterator<Item = String>,
) -> Result<Option<Command>, CliError> {
    let (mut lang, mut auto_detect_target) =
        env_trace_proto_target("LSTEG_LANG")?.unwrap_or((ProtoTarget::Farsi, true));
    let mut format = env_output_format("LSTEG_FORMAT")?.unwrap_or(OutputFormat::Text);
    let mut trace = env_optional("LSTEG_TRACE");
    let mut input_path = env_optional("LSTEG_INPUT");
    let mut output_path = env_optional("LSTEG_OUTPUT");
    let mut secret = env_optional("LSTEG_SECRET");
    let mut secret_file = None;

    let mut seen_trace = false;
    let mut seen_input = false;
    let mut seen_output = false;
    let mut seen_secret = false;
    let mut seen_secret_file = false;

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--help" | "-h" => return Ok(None),
            "--lang" => {
                let value = next_arg_value(&mut args, "--lang")?;
                let (resolved_lang, resolved_auto_detect) = parse_trace_proto_target(&value)?;
                lang = resolved_lang;
                auto_detect_target = resolved_auto_detect;
            }
            "--format" => {
                let value = next_arg_value(&mut args, "--format")?;
                format = parse_output_format(&value)?;
            }
            "--trace" => {
                if seen_trace {
                    return Err(CliError::usage(
                        "--trace cannot be provided multiple times".to_string(),
                    ));
                }
                seen_trace = true;
                trace = Some(next_arg_value(&mut args, "--trace")?);
            }
            "--input" => {
                if seen_input {
                    return Err(CliError::usage(
                        "--input cannot be provided multiple times".to_string(),
                    ));
                }
                seen_input = true;
                input_path = Some(next_arg_value(&mut args, "--input")?);
            }
            "--output" => {
                if seen_output {
                    return Err(CliError::usage(
                        "--output cannot be provided multiple times".to_string(),
                    ));
                }
                seen_output = true;
                output_path = Some(next_arg_value(&mut args, "--output")?);
            }
            "--secret" => {
                if seen_secret {
                    return Err(CliError::usage(
                        "--secret cannot be provided multiple times".to_string(),
                    ));
                }
                if seen_secret_file {
                    return Err(CliError::usage(
                        "analyze accepts either --secret or --secret-file, not both".to_string(),
                    ));
                }
                seen_secret = true;
                secret = Some(next_arg_value(&mut args, "--secret")?);
                secret_file = None;
            }
            "--secret-file" => {
                if seen_secret_file {
                    return Err(CliError::usage(
                        "--secret-file cannot be provided multiple times".to_string(),
                    ));
                }
                if seen_secret {
                    return Err(CliError::usage(
                        "analyze accepts either --secret or --secret-file, not both".to_string(),
                    ));
                }
                seen_secret_file = true;
                secret_file = Some(next_arg_value(&mut args, "--secret-file")?);
                secret = None;
            }
            _ => {
                return Err(CliError::usage(format!("unknown analyze argument: {arg}")));
            }
        }
    }

    if trace.is_some() && input_path.is_some() {
        return Err(CliError::usage(
            "analyze accepts either --trace or --input, not both".to_string(),
        ));
    }
    if secret.is_some() && secret_file.is_some() {
        return Err(CliError::usage(
            "analyze accepts either --secret or --secret-file, not both".to_string(),
        ));
    }

    Ok(Some(Command::Analyze(AnalyzeOptions {
        target: lang,
        auto_detect_target,
        trace,
        input_path,
        output_path,
        secret,
        secret_file,
        format,
    })))
}

fn parse_demo_command(mut args: impl Iterator<Item = String>) -> Result<Option<Command>, CliError> {
    match args.next().as_deref() {
        Some("fa") => Ok(Some(Command::Demo(DemoTarget::Farsi))),
        Some("en") => Ok(Some(Command::Demo(DemoTarget::English))),
        _ => Err(CliError::usage(
            "demo target is required (supported: fa, en)".to_string(),
        )),
    }
}

fn parse_languages_command(
    mut args: impl Iterator<Item = String>,
) -> Result<Option<Command>, CliError> {
    let mut format = env_output_format("LSTEG_FORMAT")?.unwrap_or(OutputFormat::Text);

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--help" | "-h" => return Ok(None),
            "--format" => {
                let value = next_arg_value(&mut args, "--format")?;
                format = parse_output_format(&value)?;
            }
            _ => {
                return Err(CliError::usage(format!(
                    "unknown languages argument: {arg}"
                )));
            }
        }
    }

    Ok(Some(Command::Languages(format)))
}

fn parse_strategies_command(
    mut args: impl Iterator<Item = String>,
) -> Result<Option<Command>, CliError> {
    let mut format = env_output_format("LSTEG_FORMAT")?.unwrap_or(OutputFormat::Text);

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--help" | "-h" => return Ok(None),
            "--format" => {
                let value = next_arg_value(&mut args, "--format")?;
                format = parse_output_format(&value)?;
            }
            _ => {
                return Err(CliError::usage(format!(
                    "unknown strategies argument: {arg}"
                )));
            }
        }
    }

    Ok(Some(Command::Strategies(format)))
}

fn parse_models_command(
    mut args: impl Iterator<Item = String>,
) -> Result<Option<Command>, CliError> {
    let mut format = env_output_format("LSTEG_FORMAT")?.unwrap_or(OutputFormat::Text);

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--help" | "-h" => return Ok(None),
            "--format" => {
                let value = next_arg_value(&mut args, "--format")?;
                format = parse_output_format(&value)?;
            }
            _ => return Err(CliError::usage(format!("unknown models argument: {arg}"))),
        }
    }

    Ok(Some(Command::Models(format)))
}

fn parse_catalog_command(
    mut args: impl Iterator<Item = String>,
) -> Result<Option<Command>, CliError> {
    let mut format = env_output_format("LSTEG_FORMAT")?.unwrap_or(OutputFormat::Text);

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--help" | "-h" => return Ok(None),
            "--format" => {
                let value = next_arg_value(&mut args, "--format")?;
                format = parse_output_format(&value)?;
            }
            _ => return Err(CliError::usage(format!("unknown catalog argument: {arg}"))),
        }
    }

    Ok(Some(Command::Catalog(format)))
}

fn parse_templates_command(
    mut args: impl Iterator<Item = String>,
) -> Result<Option<Command>, CliError> {
    let mut format = env_output_format("LSTEG_FORMAT")?.unwrap_or(OutputFormat::Text);
    let mut target = None;
    let mut seen_lang = false;

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--help" | "-h" => return Ok(None),
            "--format" => {
                let value = next_arg_value(&mut args, "--format")?;
                format = parse_output_format(&value)?;
            }
            "--lang" => {
                if seen_lang {
                    return Err(CliError::usage(
                        "--lang cannot be provided multiple times".to_string(),
                    ));
                }
                seen_lang = true;
                let value = next_arg_value(&mut args, "--lang")?;
                target = Some(parse_proto_target(&value)?);
            }
            _ => {
                return Err(CliError::usage(format!(
                    "unknown templates argument: {arg}"
                )));
            }
        }
    }

    Ok(Some(Command::Templates(TemplateQueryOptions {
        format,
        target,
    })))
}

fn parse_profiles_command(
    mut args: impl Iterator<Item = String>,
) -> Result<Option<Command>, CliError> {
    let mut format = env_output_format("LSTEG_FORMAT")?.unwrap_or(OutputFormat::Text);
    let mut target = None;
    let mut seen_lang = false;

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--help" | "-h" => return Ok(None),
            "--format" => {
                let value = next_arg_value(&mut args, "--format")?;
                format = parse_output_format(&value)?;
            }
            "--lang" => {
                if seen_lang {
                    return Err(CliError::usage(
                        "--lang cannot be provided multiple times".to_string(),
                    ));
                }
                seen_lang = true;
                let value = next_arg_value(&mut args, "--lang")?;
                target = Some(parse_proto_target(&value)?);
            }
            _ => return Err(CliError::usage(format!("unknown profiles argument: {arg}"))),
        }
    }

    Ok(Some(Command::Profiles(ProfileQueryOptions {
        format,
        target,
    })))
}

fn parse_proto_encode_command(
    args: impl Iterator<Item = String>,
) -> Result<Option<Command>, CliError> {
    let mut args = args.collect::<Vec<_>>();
    let target = match args.first().map(String::as_str) {
        Some("fa") => ProtoTarget::Farsi,
        Some("en") => ProtoTarget::English,
        _ => {
            return Err(CliError::usage(
                "proto-encode target is required (supported: fa, en)".to_string(),
            ));
        }
    };
    if args.is_empty() {
        return Err(CliError::usage(
            "proto-encode target is required (supported: fa, en)".to_string(),
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

    Ok(Some(Command::ProtoEncode(target, payload_text, json)))
}

fn parse_proto_decode_command(
    args: impl Iterator<Item = String>,
) -> Result<Option<Command>, CliError> {
    let mut args = args.collect::<Vec<_>>();
    let target = match args.first().map(String::as_str) {
        Some("fa") => ProtoTarget::Farsi,
        Some("en") => ProtoTarget::English,
        _ => {
            return Err(CliError::usage(
                "proto-decode target is required (supported: fa, en)".to_string(),
            ));
        }
    };
    if args.is_empty() {
        return Err(CliError::usage(
            "proto-decode target is required (supported: fa, en)".to_string(),
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

    Ok(Some(Command::ProtoDecode(target, trace, json)))
}

fn next_arg_value(args: &mut impl Iterator<Item = String>, flag: &str) -> Result<String, CliError> {
    args.next()
        .ok_or_else(|| CliError::usage(format!("{flag} requires a value")))
}

fn parse_proto_target(value: &str) -> Result<ProtoTarget, CliError> {
    match value {
        "fa" => Ok(ProtoTarget::Farsi),
        "en" => Ok(ProtoTarget::English),
        _ => Err(CliError::config(format!(
            "unsupported language '{value}' (supported: fa, en)"
        ))),
    }
}

fn parse_output_format(value: &str) -> Result<OutputFormat, CliError> {
    match value {
        "text" => Ok(OutputFormat::Text),
        "json" => Ok(OutputFormat::Json),
        _ => Err(CliError::config(format!(
            "unsupported output format '{value}' (supported: text, json)"
        ))),
    }
}

fn env_optional(key: &str) -> Option<String> {
    std::env::var(key)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn env_proto_target(key: &str) -> Result<Option<ProtoTarget>, CliError> {
    match env_optional(key) {
        Some(value) => parse_proto_target(&value).map(Some),
        None => Ok(None),
    }
}

fn env_trace_proto_target(key: &str) -> Result<Option<(ProtoTarget, bool)>, CliError> {
    match env_optional(key) {
        Some(value) => parse_trace_proto_target(&value).map(Some),
        None => Ok(None),
    }
}

fn parse_trace_proto_target(value: &str) -> Result<(ProtoTarget, bool), CliError> {
    match value {
        "fa" => Ok((ProtoTarget::Farsi, false)),
        "en" => Ok((ProtoTarget::English, false)),
        "auto" => Ok((ProtoTarget::Farsi, true)),
        _ => Err(CliError::config(format!(
            "unsupported language '{value}' (supported: auto, fa, en)"
        ))),
    }
}

fn env_output_format(key: &str) -> Result<Option<OutputFormat>, CliError> {
    match env_optional(key) {
        Some(value) => parse_output_format(&value).map(Some),
        None => Ok(None),
    }
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

pub(crate) fn write_usage(mut writer: impl Write) -> std::io::Result<()> {
    writeln!(writer, "LinguaSteg CLI (scaffold)")?;
    writeln!(
        writer,
        "Usage: lsteg <encode|decode|analyze|languages|strategies|models|catalog|templates|profiles|demo|proto-encode|proto-decode>"
    )?;
    writeln!(
        writer,
        "       lsteg encode [--lang fa|en] (--message <text> | --input <file>) [--secret <value> | --secret-file <file>] [--format text|json] [--output <file>]"
    )?;
    writeln!(
        writer,
        "       lsteg decode [--lang auto|fa|en] [--trace <text> | --input <file>] [--secret <value> | --secret-file <file>] [--format text|json] [--output <file>]"
    )?;
    writeln!(
        writer,
        "       lsteg analyze [--lang auto|fa|en] [--trace <text> | --input <file>] [--secret <value> | --secret-file <file>] [--format text|json] [--output <file>]"
    )?;
    writeln!(writer, "       lsteg languages [--format text|json]")?;
    writeln!(writer, "       lsteg strategies [--format text|json]")?;
    writeln!(writer, "       lsteg models [--format text|json]")?;
    writeln!(writer, "       lsteg catalog [--format text|json]")?;
    writeln!(
        writer,
        "       lsteg templates [--lang fa|en] [--format text|json]"
    )?;
    writeln!(
        writer,
        "       lsteg profiles [--lang fa|en] [--format text|json]"
    )?;
    writeln!(writer, "       lsteg demo <fa|en>")?;
    writeln!(
        writer,
        "       lsteg proto-encode <fa|en> [message] [--json]"
    )?;
    writeln!(
        writer,
        "       lsteg proto-encode <fa|en> [message] | lsteg proto-decode <fa|en> [--json]"
    )?;
    writeln!(
        writer,
        "       lsteg proto-decode <fa|en> \"<frame trace lines>\" [--json]"
    )?;
    writeln!(
        writer,
        "Env defaults: LSTEG_LANG (decode/analyze accepts auto), LSTEG_FORMAT, LSTEG_INPUT, LSTEG_OUTPUT, LSTEG_ENCODE_MESSAGE, LSTEG_TRACE, LSTEG_SECRET"
    )?;
    Ok(())
}
