use std::io::Write;

use super::types::{
    AnalyzeOptions, CliError, Command, DecodeOptions, DemoTarget, EncodeOptions, OutputFormat,
    ProtoTarget,
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
        "demo" => parse_demo_command(args),
        "proto-encode" => parse_proto_encode_command(args),
        "proto-decode" => parse_proto_decode_command(args),
        _ => Err(CliError::usage(format!("unknown command: {command}"))),
    }
}

fn parse_encode_command(
    mut args: impl Iterator<Item = String>,
) -> Result<Option<Command>, CliError> {
    let mut lang = ProtoTarget::Farsi;
    let mut format = OutputFormat::Text;
    let mut message = None;
    let mut input_path = None;
    let mut output_path = None;

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
                if message.is_some() {
                    return Err(CliError::usage(
                        "--message cannot be provided multiple times".to_string(),
                    ));
                }
                message = Some(next_arg_value(&mut args, "--message")?);
            }
            "--input" => {
                if input_path.is_some() {
                    return Err(CliError::usage(
                        "--input cannot be provided multiple times".to_string(),
                    ));
                }
                input_path = Some(next_arg_value(&mut args, "--input")?);
            }
            "--output" => {
                if output_path.is_some() {
                    return Err(CliError::usage(
                        "--output cannot be provided multiple times".to_string(),
                    ));
                }
                output_path = Some(next_arg_value(&mut args, "--output")?);
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

    Ok(Some(Command::Encode(EncodeOptions {
        target: lang,
        message,
        input_path,
        output_path,
        format,
    })))
}

fn parse_decode_command(
    mut args: impl Iterator<Item = String>,
) -> Result<Option<Command>, CliError> {
    let mut lang = ProtoTarget::Farsi;
    let mut format = OutputFormat::Text;
    let mut trace = None;
    let mut input_path = None;
    let mut output_path = None;

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
            "--trace" => {
                if trace.is_some() {
                    return Err(CliError::usage(
                        "--trace cannot be provided multiple times".to_string(),
                    ));
                }
                trace = Some(next_arg_value(&mut args, "--trace")?);
            }
            "--input" => {
                if input_path.is_some() {
                    return Err(CliError::usage(
                        "--input cannot be provided multiple times".to_string(),
                    ));
                }
                input_path = Some(next_arg_value(&mut args, "--input")?);
            }
            "--output" => {
                if output_path.is_some() {
                    return Err(CliError::usage(
                        "--output cannot be provided multiple times".to_string(),
                    ));
                }
                output_path = Some(next_arg_value(&mut args, "--output")?);
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

    Ok(Some(Command::Decode(DecodeOptions {
        target: lang,
        trace,
        input_path,
        output_path,
        format,
    })))
}

fn parse_analyze_command(
    mut args: impl Iterator<Item = String>,
) -> Result<Option<Command>, CliError> {
    let mut lang = ProtoTarget::Farsi;
    let mut format = OutputFormat::Text;
    let mut trace = None;
    let mut input_path = None;
    let mut output_path = None;

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
            "--trace" => {
                if trace.is_some() {
                    return Err(CliError::usage(
                        "--trace cannot be provided multiple times".to_string(),
                    ));
                }
                trace = Some(next_arg_value(&mut args, "--trace")?);
            }
            "--input" => {
                if input_path.is_some() {
                    return Err(CliError::usage(
                        "--input cannot be provided multiple times".to_string(),
                    ));
                }
                input_path = Some(next_arg_value(&mut args, "--input")?);
            }
            "--output" => {
                if output_path.is_some() {
                    return Err(CliError::usage(
                        "--output cannot be provided multiple times".to_string(),
                    ));
                }
                output_path = Some(next_arg_value(&mut args, "--output")?);
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

    Ok(Some(Command::Analyze(AnalyzeOptions {
        target: lang,
        trace,
        input_path,
        output_path,
        format,
    })))
}

fn parse_demo_command(mut args: impl Iterator<Item = String>) -> Result<Option<Command>, CliError> {
    match args.next().as_deref() {
        Some("fa") => Ok(Some(Command::Demo(DemoTarget::Farsi))),
        _ => Err(CliError::usage(
            "demo target is required (supported: fa)".to_string(),
        )),
    }
}

fn parse_proto_encode_command(
    args: impl Iterator<Item = String>,
) -> Result<Option<Command>, CliError> {
    let mut args = args.collect::<Vec<_>>();
    if args.first().map(String::as_str) != Some("fa") {
        return Err(CliError::usage(
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
        return Err(CliError::usage(
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

fn next_arg_value(args: &mut impl Iterator<Item = String>, flag: &str) -> Result<String, CliError> {
    args.next()
        .ok_or_else(|| CliError::usage(format!("{flag} requires a value")))
}

fn parse_proto_target(value: &str) -> Result<ProtoTarget, CliError> {
    match value {
        "fa" => Ok(ProtoTarget::Farsi),
        _ => Err(CliError::config(format!(
            "unsupported language '{value}' (supported: fa)"
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
        "Usage: lsteg <encode|decode|analyze|demo|proto-encode|proto-decode>"
    )?;
    writeln!(
        writer,
        "       lsteg encode [--lang fa] (--message <text> | --input <file>) [--format text|json] [--output <file>]"
    )?;
    writeln!(
        writer,
        "       lsteg decode [--lang fa] [--trace <text> | --input <file>] [--format text|json] [--output <file>]"
    )?;
    writeln!(
        writer,
        "       lsteg analyze [--lang fa] [--trace <text> | --input <file>] [--format text|json] [--output <file>]"
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
