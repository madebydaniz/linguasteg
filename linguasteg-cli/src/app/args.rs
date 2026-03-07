use std::io::Write;

use super::types::{
    AnalyzeOptions, CatalogQueryOptions, CliError, Command, DataArtifactValidateOptions,
    DataCleanOptions, DataCommand, DataDoctorOptions, DataExportManifestOptions,
    DataImportManifestOptions, DataInstallOptions, DataListOptions, DataPinOptions,
    DataStatusOptions, DataVerifyOptions, DecodeInputMode, DecodeOptions, DemoTarget,
    EncodeOptions, OutputFormat, ProfileQueryOptions, ProtoTarget, SchemaQueryOptions,
    TemplateQueryOptions, ValidateOptions,
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
        "validate" => parse_validate_command(args),
        "languages" => parse_languages_command(args),
        "strategies" => parse_strategies_command(args),
        "models" => parse_models_command(args),
        "catalog" => parse_catalog_command(args),
        "templates" => parse_templates_command(args),
        "profiles" => parse_profiles_command(args),
        "schemas" => parse_schemas_command(args),
        "data" => parse_data_command(args),
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
    let mut data_dir = env_optional("LSTEG_DATA_DIR");
    let mut source_id = None;
    let mut emit_trace = false;
    let mut profile = env_optional("LSTEG_PROFILE");
    let mut seen_source = false;
    let mut seen_data_dir = false;
    let mut seen_profile = false;
    let mut payload = EncodePayloadArgs::from_env();
    let mut secrets = SecretArgs::from_env();

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--help" | "-h" => return Ok(None),
            "--lang" => {
                parse_proto_lang_arg(&mut args, &mut lang)?;
            }
            "--format" => {
                parse_output_format_arg(&mut args, &mut format)?;
            }
            "--emit-trace" => {
                emit_trace = true;
            }
            "--source" => {
                if seen_source {
                    return Err(CliError::usage(
                        "--source cannot be provided multiple times".to_string(),
                    ));
                }
                seen_source = true;
                source_id = Some(next_arg_value(&mut args, "--source")?);
            }
            "--data-dir" => {
                if seen_data_dir {
                    return Err(CliError::usage(
                        "--data-dir cannot be provided multiple times".to_string(),
                    ));
                }
                seen_data_dir = true;
                data_dir = Some(next_arg_value(&mut args, "--data-dir")?);
            }
            "--profile" => {
                if seen_profile {
                    return Err(CliError::usage(
                        "--profile cannot be provided multiple times".to_string(),
                    ));
                }
                seen_profile = true;
                profile = Some(next_arg_value(&mut args, "--profile")?);
            }
            _ => {
                if payload.handle_flag(arg.as_str(), &mut args)? {
                    continue;
                }
                if secrets.handle_flag(arg.as_str(), &mut args, "encode")? {
                    continue;
                }
                return Err(CliError::usage(format!("unknown encode argument: {arg}")));
            }
        }
    }

    payload.ensure_valid()?;
    let (message, input_path, output_path) = payload.into_parts();
    secrets.ensure_not_ambiguous("encode")?;
    let (secret, secret_file) = secrets.into_parts();

    Ok(Some(Command::Encode(EncodeOptions {
        target: lang,
        message,
        input_path,
        output_path,
        source_id,
        data_dir,
        emit_trace,
        profile,
        secret,
        secret_file,
        format,
    })))
}

fn parse_decode_command(args: impl Iterator<Item = String>) -> Result<Option<Command>, CliError> {
    let parsed = match parse_trace_like_command_args(args, "decode")? {
        Some(value) => value,
        None => return Ok(None),
    };

    Ok(Some(Command::Decode(DecodeOptions {
        target: parsed.lang,
        auto_detect_target: parsed.auto_detect_target,
        input_mode: parsed.decode_input_mode,
        trace: parsed.trace,
        input_path: parsed.input_path,
        output_path: parsed.output_path,
        data_dir: parsed.data_dir,
        secret: parsed.secret,
        secret_file: parsed.secret_file,
        format: parsed.format,
    })))
}

fn parse_analyze_command(args: impl Iterator<Item = String>) -> Result<Option<Command>, CliError> {
    let parsed = match parse_trace_like_command_args(args, "analyze")? {
        Some(value) => value,
        None => return Ok(None),
    };

    Ok(Some(Command::Analyze(AnalyzeOptions {
        target: parsed.lang,
        auto_detect_target: parsed.auto_detect_target,
        input_mode: parsed.decode_input_mode,
        trace: parsed.trace,
        input_path: parsed.input_path,
        output_path: parsed.output_path,
        data_dir: parsed.data_dir,
        secret: parsed.secret,
        secret_file: parsed.secret_file,
        format: parsed.format,
    })))
}

fn parse_validate_command(args: impl Iterator<Item = String>) -> Result<Option<Command>, CliError> {
    let parsed = match parse_trace_like_command_args(args, "validate")? {
        Some(value) => value,
        None => return Ok(None),
    };

    Ok(Some(Command::Validate(ValidateOptions {
        target: parsed.lang,
        auto_detect_target: parsed.auto_detect_target,
        input_mode: parsed.decode_input_mode,
        trace: parsed.trace,
        input_path: parsed.input_path,
        output_path: parsed.output_path,
        data_dir: parsed.data_dir,
        secret: parsed.secret,
        secret_file: parsed.secret_file,
        format: parsed.format,
    })))
}

struct ParsedTraceLikeCommand {
    lang: ProtoTarget,
    auto_detect_target: bool,
    decode_input_mode: DecodeInputMode,
    trace: Option<String>,
    input_path: Option<String>,
    output_path: Option<String>,
    data_dir: Option<String>,
    secret: Option<String>,
    secret_file: Option<String>,
    format: OutputFormat,
}

fn parse_trace_like_command_args(
    mut args: impl Iterator<Item = String>,
    command: &str,
) -> Result<Option<ParsedTraceLikeCommand>, CliError> {
    let supports_input_mode = matches!(command, "decode" | "analyze" | "validate");
    let (mut lang, mut auto_detect_target) =
        env_trace_proto_target("LSTEG_LANG")?.unwrap_or((ProtoTarget::Farsi, true));
    let mut format = env_output_format("LSTEG_FORMAT")?.unwrap_or(OutputFormat::Text);
    let mut trace = env_optional("LSTEG_TRACE");
    let mut input_path = env_optional("LSTEG_INPUT");
    let mut output_path = env_optional("LSTEG_OUTPUT");
    let mut data_dir = env_optional("LSTEG_DATA_DIR");
    let mut decode_input_mode = DecodeInputMode::Auto;
    let mut secrets = SecretArgs::from_env();

    let mut seen_trace = false;
    let mut seen_input = false;
    let mut seen_output = false;
    let mut seen_data_dir = false;
    let mut seen_trace_input = false;
    let mut seen_text_input = false;

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--help" | "-h" => return Ok(None),
            "--lang" => {
                parse_trace_lang_arg(&mut args, &mut lang, &mut auto_detect_target)?;
            }
            "--format" => {
                parse_output_format_arg(&mut args, &mut format)?;
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
            "--data-dir" => {
                if seen_data_dir {
                    return Err(CliError::usage(
                        "--data-dir cannot be provided multiple times".to_string(),
                    ));
                }
                seen_data_dir = true;
                data_dir = Some(next_arg_value(&mut args, "--data-dir")?);
            }
            "--trace-input" if supports_input_mode => {
                if seen_trace_input {
                    return Err(CliError::usage(
                        "--trace-input cannot be provided multiple times".to_string(),
                    ));
                }
                if seen_text_input {
                    return Err(CliError::usage(format!(
                        "{command} accepts either --trace-input or --text-input, not both"
                    )));
                }
                seen_trace_input = true;
                decode_input_mode = DecodeInputMode::Trace;
            }
            "--text-input" if supports_input_mode => {
                if seen_text_input {
                    return Err(CliError::usage(
                        "--text-input cannot be provided multiple times".to_string(),
                    ));
                }
                if seen_trace_input {
                    return Err(CliError::usage(format!(
                        "{command} accepts either --trace-input or --text-input, not both"
                    )));
                }
                seen_text_input = true;
                decode_input_mode = DecodeInputMode::Text;
            }
            _ => {
                if secrets.handle_flag(arg.as_str(), &mut args, command)? {
                    continue;
                }
                return Err(CliError::usage(format!(
                    "unknown {command} argument: {arg}"
                )));
            }
        }
    }

    if trace.is_some() && input_path.is_some() {
        return Err(CliError::usage(format!(
            "{command} accepts either --trace or --input, not both"
        )));
    }
    secrets.ensure_not_ambiguous(command)?;
    let (secret, secret_file) = secrets.into_parts();

    Ok(Some(ParsedTraceLikeCommand {
        lang,
        auto_detect_target,
        decode_input_mode,
        trace,
        input_path,
        output_path,
        data_dir,
        secret,
        secret_file,
        format,
    }))
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
                parse_output_format_arg(&mut args, &mut format)?;
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
                parse_output_format_arg(&mut args, &mut format)?;
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
                parse_output_format_arg(&mut args, &mut format)?;
            }
            _ => return Err(CliError::usage(format!("unknown models argument: {arg}"))),
        }
    }

    Ok(Some(Command::Models(format)))
}

fn parse_catalog_command(args: impl Iterator<Item = String>) -> Result<Option<Command>, CliError> {
    let parsed = match parse_discovery_command_args(args, "catalog")? {
        Some(value) => value,
        None => return Ok(None),
    };

    Ok(Some(Command::Catalog(CatalogQueryOptions {
        format: parsed.format,
        target: parsed.target,
    })))
}

fn parse_templates_command(
    args: impl Iterator<Item = String>,
) -> Result<Option<Command>, CliError> {
    let parsed = match parse_discovery_command_args(args, "templates")? {
        Some(value) => value,
        None => return Ok(None),
    };

    Ok(Some(Command::Templates(TemplateQueryOptions {
        format: parsed.format,
        target: parsed.target,
    })))
}

fn parse_profiles_command(args: impl Iterator<Item = String>) -> Result<Option<Command>, CliError> {
    let parsed = match parse_discovery_command_args(args, "profiles")? {
        Some(value) => value,
        None => return Ok(None),
    };

    Ok(Some(Command::Profiles(ProfileQueryOptions {
        format: parsed.format,
        target: parsed.target,
    })))
}

fn parse_schemas_command(args: impl Iterator<Item = String>) -> Result<Option<Command>, CliError> {
    let parsed = match parse_discovery_command_args(args, "schemas")? {
        Some(value) => value,
        None => return Ok(None),
    };

    Ok(Some(Command::Schemas(SchemaQueryOptions {
        format: parsed.format,
        target: parsed.target,
    })))
}

fn parse_data_command(mut args: impl Iterator<Item = String>) -> Result<Option<Command>, CliError> {
    let Some(subcommand) = args.next() else {
        return Err(CliError::usage(
            "data subcommand is required (supported: list, status, verify, doctor, clean, pin, artifact validate, export-manifest, import-manifest, install, update)"
                .to_string(),
        ));
    };
    if subcommand == "--help" || subcommand == "-h" {
        return Ok(None);
    }

    match subcommand.as_str() {
        "list" => parse_data_list_command(args),
        "status" => parse_data_status_command(args),
        "verify" => parse_data_verify_command(args),
        "doctor" => parse_data_doctor_command(args),
        "clean" => parse_data_clean_command(args),
        "pin" => parse_data_pin_command(args),
        "artifact" => parse_data_artifact_command(args),
        "export-manifest" => parse_data_export_manifest_command(args),
        "import-manifest" => parse_data_import_manifest_command(args),
        "install" => parse_data_install_command(args, false),
        "update" => parse_data_install_command(args, true),
        _ => Err(CliError::usage(format!(
            "unknown data subcommand: {subcommand} (supported: list, status, verify, doctor, clean, pin, artifact validate, export-manifest, import-manifest, install, update)"
        ))),
    }
}

fn parse_data_artifact_command(
    mut args: impl Iterator<Item = String>,
) -> Result<Option<Command>, CliError> {
    let Some(subcommand) = args.next() else {
        return Err(CliError::usage(
            "data artifact subcommand is required (supported: validate)".to_string(),
        ));
    };
    if subcommand == "--help" || subcommand == "-h" {
        return Ok(None);
    }

    match subcommand.as_str() {
        "validate" => parse_data_artifact_validate_command(args),
        _ => Err(CliError::usage(format!(
            "unknown data artifact subcommand: {subcommand} (supported: validate)"
        ))),
    }
}

fn parse_data_artifact_validate_command(
    mut args: impl Iterator<Item = String>,
) -> Result<Option<Command>, CliError> {
    let mut format = env_output_format("LSTEG_FORMAT")?.unwrap_or(OutputFormat::Text);
    let mut target = None;
    let mut input_path = None;
    let mut seen_lang = false;
    let mut seen_input = false;

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--help" | "-h" => return Ok(None),
            "--format" => {
                parse_output_format_arg(&mut args, &mut format)?;
            }
            "--lang" => {
                if seen_lang {
                    return Err(CliError::usage(
                        "--lang cannot be provided multiple times".to_string(),
                    ));
                }
                seen_lang = true;
                target = Some(parse_proto_target(&next_arg_value(&mut args, "--lang")?)?);
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
            _ => {
                return Err(CliError::usage(format!(
                    "unknown data artifact validate argument: {arg}"
                )));
            }
        }
    }

    let target =
        target.ok_or_else(|| CliError::usage("data artifact validate requires --lang <code>"))?;
    let input_path = input_path
        .ok_or_else(|| CliError::usage("data artifact validate requires --input <file>"))?;

    Ok(Some(Command::Data(DataCommand::ArtifactValidate(
        DataArtifactValidateOptions {
            format,
            target,
            input_path,
        },
    ))))
}

fn parse_data_list_command(
    mut args: impl Iterator<Item = String>,
) -> Result<Option<Command>, CliError> {
    let mut format = env_output_format("LSTEG_FORMAT")?.unwrap_or(OutputFormat::Text);
    let mut target = None;
    let mut seen_lang = false;
    let mut data_dir = env_optional("LSTEG_DATA_DIR");
    let mut seen_data_dir = false;

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--help" | "-h" => return Ok(None),
            "--format" => {
                parse_output_format_arg(&mut args, &mut format)?;
            }
            "--lang" => {
                parse_discovery_lang_arg(&mut args, &mut target, &mut seen_lang)?;
            }
            "--data-dir" => {
                if seen_data_dir {
                    return Err(CliError::usage(
                        "--data-dir cannot be provided multiple times".to_string(),
                    ));
                }
                seen_data_dir = true;
                data_dir = Some(next_arg_value(&mut args, "--data-dir")?);
            }
            _ => {
                return Err(CliError::usage(format!(
                    "unknown data list argument: {arg}"
                )));
            }
        }
    }

    Ok(Some(Command::Data(DataCommand::List(DataListOptions {
        format,
        target,
        data_dir,
    }))))
}

fn parse_data_status_command(
    mut args: impl Iterator<Item = String>,
) -> Result<Option<Command>, CliError> {
    let mut format = env_output_format("LSTEG_FORMAT")?.unwrap_or(OutputFormat::Text);
    let mut target = None;
    let mut seen_lang = false;
    let mut data_dir = env_optional("LSTEG_DATA_DIR");
    let mut seen_data_dir = false;

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--help" | "-h" => return Ok(None),
            "--format" => {
                parse_output_format_arg(&mut args, &mut format)?;
            }
            "--lang" => {
                parse_discovery_lang_arg(&mut args, &mut target, &mut seen_lang)?;
            }
            "--data-dir" => {
                if seen_data_dir {
                    return Err(CliError::usage(
                        "--data-dir cannot be provided multiple times".to_string(),
                    ));
                }
                seen_data_dir = true;
                data_dir = Some(next_arg_value(&mut args, "--data-dir")?);
            }
            _ => {
                return Err(CliError::usage(format!(
                    "unknown data status argument: {arg}"
                )));
            }
        }
    }

    Ok(Some(Command::Data(DataCommand::Status(
        DataStatusOptions {
            format,
            target,
            data_dir,
        },
    ))))
}

fn parse_data_verify_command(
    mut args: impl Iterator<Item = String>,
) -> Result<Option<Command>, CliError> {
    let mut format = env_output_format("LSTEG_FORMAT")?.unwrap_or(OutputFormat::Text);
    let mut target = None;
    let mut source_id = None;
    let mut seen_lang = false;
    let mut seen_source = false;
    let mut data_dir = env_optional("LSTEG_DATA_DIR");
    let mut seen_data_dir = false;

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--help" | "-h" => return Ok(None),
            "--format" => {
                parse_output_format_arg(&mut args, &mut format)?;
            }
            "--lang" => {
                parse_discovery_lang_arg(&mut args, &mut target, &mut seen_lang)?;
            }
            "--source" => {
                if seen_source {
                    return Err(CliError::usage(
                        "--source cannot be provided multiple times".to_string(),
                    ));
                }
                seen_source = true;
                source_id = Some(next_arg_value(&mut args, "--source")?);
            }
            "--data-dir" => {
                if seen_data_dir {
                    return Err(CliError::usage(
                        "--data-dir cannot be provided multiple times".to_string(),
                    ));
                }
                seen_data_dir = true;
                data_dir = Some(next_arg_value(&mut args, "--data-dir")?);
            }
            _ => {
                return Err(CliError::usage(format!(
                    "unknown data verify argument: {arg}"
                )));
            }
        }
    }

    Ok(Some(Command::Data(DataCommand::Verify(
        DataVerifyOptions {
            format,
            target,
            source_id,
            data_dir,
        },
    ))))
}

fn parse_data_doctor_command(
    mut args: impl Iterator<Item = String>,
) -> Result<Option<Command>, CliError> {
    let mut format = env_output_format("LSTEG_FORMAT")?.unwrap_or(OutputFormat::Text);
    let mut target = None;
    let mut source_id = None;
    let mut fix = false;
    let mut seen_lang = false;
    let mut seen_source = false;
    let mut seen_fix = false;
    let mut data_dir = env_optional("LSTEG_DATA_DIR");
    let mut seen_data_dir = false;

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--help" | "-h" => return Ok(None),
            "--format" => {
                parse_output_format_arg(&mut args, &mut format)?;
            }
            "--lang" => {
                parse_discovery_lang_arg(&mut args, &mut target, &mut seen_lang)?;
            }
            "--source" => {
                if seen_source {
                    return Err(CliError::usage(
                        "--source cannot be provided multiple times".to_string(),
                    ));
                }
                seen_source = true;
                source_id = Some(next_arg_value(&mut args, "--source")?);
            }
            "--fix" => {
                if seen_fix {
                    return Err(CliError::usage(
                        "--fix cannot be provided multiple times".to_string(),
                    ));
                }
                seen_fix = true;
                fix = true;
            }
            "--data-dir" => {
                if seen_data_dir {
                    return Err(CliError::usage(
                        "--data-dir cannot be provided multiple times".to_string(),
                    ));
                }
                seen_data_dir = true;
                data_dir = Some(next_arg_value(&mut args, "--data-dir")?);
            }
            _ => {
                return Err(CliError::usage(format!(
                    "unknown data doctor argument: {arg}"
                )));
            }
        }
    }

    Ok(Some(Command::Data(DataCommand::Doctor(
        DataDoctorOptions {
            format,
            target,
            source_id,
            fix,
            data_dir,
        },
    ))))
}

fn parse_data_clean_command(
    mut args: impl Iterator<Item = String>,
) -> Result<Option<Command>, CliError> {
    let mut format = env_output_format("LSTEG_FORMAT")?.unwrap_or(OutputFormat::Text);
    let mut target = None;
    let mut source_id = None;
    let mut apply = false;
    let mut seen_lang = false;
    let mut seen_source = false;
    let mut seen_apply = false;
    let mut data_dir = env_optional("LSTEG_DATA_DIR");
    let mut seen_data_dir = false;

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--help" | "-h" => return Ok(None),
            "--format" => {
                parse_output_format_arg(&mut args, &mut format)?;
            }
            "--lang" => {
                parse_discovery_lang_arg(&mut args, &mut target, &mut seen_lang)?;
            }
            "--source" => {
                if seen_source {
                    return Err(CliError::usage(
                        "--source cannot be provided multiple times".to_string(),
                    ));
                }
                seen_source = true;
                source_id = Some(next_arg_value(&mut args, "--source")?);
            }
            "--apply" => {
                if seen_apply {
                    return Err(CliError::usage(
                        "--apply cannot be provided multiple times".to_string(),
                    ));
                }
                seen_apply = true;
                apply = true;
            }
            "--data-dir" => {
                if seen_data_dir {
                    return Err(CliError::usage(
                        "--data-dir cannot be provided multiple times".to_string(),
                    ));
                }
                seen_data_dir = true;
                data_dir = Some(next_arg_value(&mut args, "--data-dir")?);
            }
            _ => {
                return Err(CliError::usage(format!(
                    "unknown data clean argument: {arg}"
                )));
            }
        }
    }

    Ok(Some(Command::Data(DataCommand::Clean(DataCleanOptions {
        format,
        target,
        source_id,
        apply,
        data_dir,
    }))))
}

fn parse_data_pin_command(
    mut args: impl Iterator<Item = String>,
) -> Result<Option<Command>, CliError> {
    let mut format = env_output_format("LSTEG_FORMAT")?.unwrap_or(OutputFormat::Text);
    let mut target = None;
    let mut source_id = None;
    let mut checksum_sha256 = None;
    let mut seen_lang = false;
    let mut seen_source = false;
    let mut seen_checksum = false;
    let mut data_dir = env_optional("LSTEG_DATA_DIR");
    let mut seen_data_dir = false;

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--help" | "-h" => return Ok(None),
            "--format" => {
                parse_output_format_arg(&mut args, &mut format)?;
            }
            "--lang" => {
                parse_discovery_lang_arg(&mut args, &mut target, &mut seen_lang)?;
            }
            "--source" => {
                if seen_source {
                    return Err(CliError::usage(
                        "--source cannot be provided multiple times".to_string(),
                    ));
                }
                seen_source = true;
                source_id = Some(next_arg_value(&mut args, "--source")?);
            }
            "--checksum" => {
                if seen_checksum {
                    return Err(CliError::usage(
                        "--checksum cannot be provided multiple times".to_string(),
                    ));
                }
                seen_checksum = true;
                checksum_sha256 = Some(next_arg_value(&mut args, "--checksum")?);
            }
            "--data-dir" => {
                if seen_data_dir {
                    return Err(CliError::usage(
                        "--data-dir cannot be provided multiple times".to_string(),
                    ));
                }
                seen_data_dir = true;
                data_dir = Some(next_arg_value(&mut args, "--data-dir")?);
            }
            _ => {
                return Err(CliError::usage(format!("unknown data pin argument: {arg}")));
            }
        }
    }

    Ok(Some(Command::Data(DataCommand::Pin(DataPinOptions {
        format,
        target,
        source_id,
        checksum_sha256,
        data_dir,
    }))))
}

fn parse_data_export_manifest_command(
    mut args: impl Iterator<Item = String>,
) -> Result<Option<Command>, CliError> {
    let mut format = env_output_format("LSTEG_FORMAT")?.unwrap_or(OutputFormat::Text);
    let mut target = None;
    let mut source_id = None;
    let mut output_path = None;
    let mut seen_lang = false;
    let mut seen_source = false;
    let mut seen_output = false;
    let mut data_dir = env_optional("LSTEG_DATA_DIR");
    let mut seen_data_dir = false;

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--help" | "-h" => return Ok(None),
            "--format" => {
                parse_output_format_arg(&mut args, &mut format)?;
            }
            "--lang" => {
                parse_discovery_lang_arg(&mut args, &mut target, &mut seen_lang)?;
            }
            "--source" => {
                if seen_source {
                    return Err(CliError::usage(
                        "--source cannot be provided multiple times".to_string(),
                    ));
                }
                seen_source = true;
                source_id = Some(next_arg_value(&mut args, "--source")?);
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
            "--data-dir" => {
                if seen_data_dir {
                    return Err(CliError::usage(
                        "--data-dir cannot be provided multiple times".to_string(),
                    ));
                }
                seen_data_dir = true;
                data_dir = Some(next_arg_value(&mut args, "--data-dir")?);
            }
            _ => {
                return Err(CliError::usage(format!(
                    "unknown data export-manifest argument: {arg}"
                )));
            }
        }
    }

    Ok(Some(Command::Data(DataCommand::ExportManifest(
        DataExportManifestOptions {
            format,
            target,
            source_id,
            output_path,
            data_dir,
        },
    ))))
}

fn parse_data_import_manifest_command(
    mut args: impl Iterator<Item = String>,
) -> Result<Option<Command>, CliError> {
    let mut format = env_output_format("LSTEG_FORMAT")?.unwrap_or(OutputFormat::Text);
    let mut target = None;
    let mut source_id = None;
    let mut input_path = None;
    let mut seen_lang = false;
    let mut seen_source = false;
    let mut seen_input = false;
    let mut data_dir = env_optional("LSTEG_DATA_DIR");
    let mut seen_data_dir = false;

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--help" | "-h" => return Ok(None),
            "--format" => {
                parse_output_format_arg(&mut args, &mut format)?;
            }
            "--lang" => {
                parse_discovery_lang_arg(&mut args, &mut target, &mut seen_lang)?;
            }
            "--source" => {
                if seen_source {
                    return Err(CliError::usage(
                        "--source cannot be provided multiple times".to_string(),
                    ));
                }
                seen_source = true;
                source_id = Some(next_arg_value(&mut args, "--source")?);
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
            "--data-dir" => {
                if seen_data_dir {
                    return Err(CliError::usage(
                        "--data-dir cannot be provided multiple times".to_string(),
                    ));
                }
                seen_data_dir = true;
                data_dir = Some(next_arg_value(&mut args, "--data-dir")?);
            }
            _ => {
                return Err(CliError::usage(format!(
                    "unknown data import-manifest argument: {arg}"
                )));
            }
        }
    }

    let Some(input_path) = input_path else {
        return Err(CliError::usage(
            "data import-manifest requires --input <file>".to_string(),
        ));
    };

    Ok(Some(Command::Data(DataCommand::ImportManifest(
        DataImportManifestOptions {
            format,
            target,
            source_id,
            input_path,
            data_dir,
        },
    ))))
}

fn parse_data_install_command(
    mut args: impl Iterator<Item = String>,
    update: bool,
) -> Result<Option<Command>, CliError> {
    let mut format = env_output_format("LSTEG_FORMAT")?.unwrap_or(OutputFormat::Text);
    let mut targets: Option<Vec<ProtoTarget>> = None;
    let mut source_id = None;
    let mut artifact_url = None;
    let mut seen_lang = false;
    let mut seen_source = false;
    let mut seen_artifact_url = false;
    let mut data_dir = env_optional("LSTEG_DATA_DIR");
    let mut seen_data_dir = false;

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--help" | "-h" => return Ok(None),
            "--format" => {
                parse_output_format_arg(&mut args, &mut format)?;
            }
            "--lang" => {
                if seen_lang {
                    return Err(CliError::usage(
                        "--lang cannot be provided multiple times".to_string(),
                    ));
                }
                seen_lang = true;
                let value = next_arg_value(&mut args, "--lang")?;
                targets = Some(parse_data_lang_targets(&value)?);
            }
            "--source" => {
                if seen_source {
                    return Err(CliError::usage(
                        "--source cannot be provided multiple times".to_string(),
                    ));
                }
                seen_source = true;
                source_id = Some(next_arg_value(&mut args, "--source")?);
            }
            "--artifact-url" => {
                if seen_artifact_url {
                    return Err(CliError::usage(
                        "--artifact-url cannot be provided multiple times".to_string(),
                    ));
                }
                seen_artifact_url = true;
                artifact_url = Some(next_arg_value(&mut args, "--artifact-url")?);
            }
            "--data-dir" => {
                if seen_data_dir {
                    return Err(CliError::usage(
                        "--data-dir cannot be provided multiple times".to_string(),
                    ));
                }
                seen_data_dir = true;
                data_dir = Some(next_arg_value(&mut args, "--data-dir")?);
            }
            _ => {
                let operation = if update { "update" } else { "install" };
                return Err(CliError::usage(format!(
                    "unknown data {operation} argument: {arg}"
                )));
            }
        }
    }

    let Some(targets) = targets else {
        let operation = if update { "update" } else { "install" };
        return Err(CliError::usage(format!(
            "data {operation} requires --lang <code[,code...]>"
        )));
    };

    let options = DataInstallOptions {
        format,
        targets,
        source_id,
        artifact_url,
        data_dir,
    };
    let command = if update {
        DataCommand::Update(options)
    } else {
        DataCommand::Install(options)
    };

    Ok(Some(Command::Data(command)))
}

fn parse_data_lang_targets(value: &str) -> Result<Vec<ProtoTarget>, CliError> {
    let mut targets = Vec::new();
    for chunk in value.split(',') {
        let lang = chunk.trim();
        if lang.is_empty() {
            continue;
        }
        let target = parse_proto_target(lang)?;
        if !targets.contains(&target) {
            targets.push(target);
        }
    }

    if targets.is_empty() {
        return Err(CliError::usage(
            "--lang requires at least one language code".to_string(),
        ));
    }

    Ok(targets)
}

struct ParsedDiscoveryCommand {
    format: OutputFormat,
    target: Option<ProtoTarget>,
}

fn parse_discovery_command_args(
    mut args: impl Iterator<Item = String>,
    command: &str,
) -> Result<Option<ParsedDiscoveryCommand>, CliError> {
    let mut format = env_output_format("LSTEG_FORMAT")?.unwrap_or(OutputFormat::Text);
    let mut target = None;
    let mut seen_lang = false;

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--help" | "-h" => return Ok(None),
            "--format" => {
                parse_output_format_arg(&mut args, &mut format)?;
            }
            "--lang" => {
                parse_discovery_lang_arg(&mut args, &mut target, &mut seen_lang)?;
            }
            _ => {
                return Err(CliError::usage(format!(
                    "unknown {command} argument: {arg}"
                )));
            }
        }
    }

    Ok(Some(ParsedDiscoveryCommand { format, target }))
}

fn parse_proto_encode_command(
    args: impl Iterator<Item = String>,
) -> Result<Option<Command>, CliError> {
    let mut args = args.collect::<Vec<_>>();
    if args.is_empty() {
        return Err(CliError::usage(
            "proto-encode target is required (use language code like: fa, en, de)".to_string(),
        ));
    }
    let target = parse_proto_target(args[0].as_str())?;

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
    if args.is_empty() {
        return Err(CliError::usage(
            "proto-decode target is required (use language code like: fa, en, de)".to_string(),
        ));
    }
    let target = parse_proto_target(args[0].as_str())?;

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

struct EncodePayloadArgs {
    message: Option<String>,
    input_path: Option<String>,
    output_path: Option<String>,
    seen_message: bool,
    seen_input: bool,
    seen_output: bool,
}

impl EncodePayloadArgs {
    fn from_env() -> Self {
        Self {
            message: env_optional("LSTEG_ENCODE_MESSAGE"),
            input_path: env_optional("LSTEG_INPUT"),
            output_path: env_optional("LSTEG_OUTPUT"),
            seen_message: false,
            seen_input: false,
            seen_output: false,
        }
    }

    fn handle_flag(
        &mut self,
        arg: &str,
        args: &mut impl Iterator<Item = String>,
    ) -> Result<bool, CliError> {
        match arg {
            "--message" => {
                if self.seen_message {
                    return Err(CliError::usage(
                        "--message cannot be provided multiple times".to_string(),
                    ));
                }
                self.seen_message = true;
                self.message = Some(next_arg_value(args, "--message")?);
                Ok(true)
            }
            "--input" => {
                if self.seen_input {
                    return Err(CliError::usage(
                        "--input cannot be provided multiple times".to_string(),
                    ));
                }
                self.seen_input = true;
                self.input_path = Some(next_arg_value(args, "--input")?);
                Ok(true)
            }
            "--output" => {
                if self.seen_output {
                    return Err(CliError::usage(
                        "--output cannot be provided multiple times".to_string(),
                    ));
                }
                self.seen_output = true;
                self.output_path = Some(next_arg_value(args, "--output")?);
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    fn ensure_valid(&self) -> Result<(), CliError> {
        if self.message.is_some() && self.input_path.is_some() {
            return Err(CliError::usage(
                "encode accepts either --message or --input, not both".to_string(),
            ));
        }
        if self.message.is_none() && self.input_path.is_none() {
            return Err(CliError::usage(
                "encode requires --message <text> or --input <file>".to_string(),
            ));
        }
        Ok(())
    }

    fn into_parts(self) -> (Option<String>, Option<String>, Option<String>) {
        (self.message, self.input_path, self.output_path)
    }
}

struct SecretArgs {
    secret: Option<String>,
    secret_file: Option<String>,
    seen_secret: bool,
    seen_secret_file: bool,
    env_ambiguous: bool,
}

impl SecretArgs {
    fn from_env() -> Self {
        let secret = env_optional("LSTEG_SECRET");
        let secret_file = env_optional("LSTEG_SECRET_FILE");
        let env_ambiguous = secret.is_some() && secret_file.is_some();
        Self {
            secret,
            secret_file,
            seen_secret: false,
            seen_secret_file: false,
            env_ambiguous,
        }
    }

    fn handle_flag(
        &mut self,
        arg: &str,
        args: &mut impl Iterator<Item = String>,
        command: &str,
    ) -> Result<bool, CliError> {
        match arg {
            "--secret" => {
                if self.seen_secret {
                    return Err(CliError::usage(
                        "--secret cannot be provided multiple times".to_string(),
                    ));
                }
                if self.seen_secret_file {
                    return Err(CliError::usage(format!(
                        "{command} accepts either --secret or --secret-file, not both"
                    )));
                }
                self.seen_secret = true;
                self.secret = Some(next_arg_value(args, "--secret")?);
                self.secret_file = None;
                Ok(true)
            }
            "--secret-file" => {
                if self.seen_secret_file {
                    return Err(CliError::usage(
                        "--secret-file cannot be provided multiple times".to_string(),
                    ));
                }
                if self.seen_secret {
                    return Err(CliError::usage(format!(
                        "{command} accepts either --secret or --secret-file, not both"
                    )));
                }
                self.seen_secret_file = true;
                self.secret_file = Some(next_arg_value(args, "--secret-file")?);
                self.secret = None;
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    fn ensure_not_ambiguous(&self, command: &str) -> Result<(), CliError> {
        if self.secret.is_some() && self.secret_file.is_some() {
            if self.env_ambiguous {
                return Err(CliError::config(
                    "secret source is ambiguous; set only one of LSTEG_SECRET or LSTEG_SECRET_FILE, or override with --secret/--secret-file",
                ));
            }
            return Err(CliError::usage(format!(
                "{command} accepts either --secret or --secret-file, not both"
            )));
        }
        Ok(())
    }

    fn into_parts(self) -> (Option<String>, Option<String>) {
        (self.secret, self.secret_file)
    }
}

fn next_arg_value(args: &mut impl Iterator<Item = String>, flag: &str) -> Result<String, CliError> {
    args.next()
        .ok_or_else(|| CliError::usage(format!("{flag} requires a value")))
}

fn parse_output_format_arg(
    args: &mut impl Iterator<Item = String>,
    format: &mut OutputFormat,
) -> Result<(), CliError> {
    let value = next_arg_value(args, "--format")?;
    *format = parse_output_format(&value)?;
    Ok(())
}

fn parse_proto_lang_arg(
    args: &mut impl Iterator<Item = String>,
    target: &mut ProtoTarget,
) -> Result<(), CliError> {
    let value = next_arg_value(args, "--lang")?;
    *target = parse_proto_target(&value)?;
    Ok(())
}

fn parse_trace_lang_arg(
    args: &mut impl Iterator<Item = String>,
    target: &mut ProtoTarget,
    auto_detect_target: &mut bool,
) -> Result<(), CliError> {
    let value = next_arg_value(args, "--lang")?;
    let (resolved_lang, resolved_auto_detect) = parse_trace_proto_target(&value)?;
    *target = resolved_lang;
    *auto_detect_target = resolved_auto_detect;
    Ok(())
}

fn parse_discovery_lang_arg(
    args: &mut impl Iterator<Item = String>,
    target: &mut Option<ProtoTarget>,
    seen_lang: &mut bool,
) -> Result<(), CliError> {
    if *seen_lang {
        return Err(CliError::usage(
            "--lang cannot be provided multiple times".to_string(),
        ));
    }
    *seen_lang = true;
    let value = next_arg_value(args, "--lang")?;
    *target = Some(parse_proto_target(&value)?);
    Ok(())
}

fn parse_proto_target(value: &str) -> Result<ProtoTarget, CliError> {
    let normalized = normalize_language_code(value)?;
    Ok(ProtoTarget::from_language_code(&normalized))
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
        _ => Ok((parse_proto_target(value)?, false)),
    }
}

fn normalize_language_code(value: &str) -> Result<String, CliError> {
    let normalized = value.trim().to_ascii_lowercase();
    if normalized.is_empty() {
        return Err(CliError::config(
            "language code must not be empty".to_string(),
        ));
    }
    if normalized.starts_with('-') || normalized.ends_with('-') || normalized.contains("--") {
        return Err(CliError::config(format!(
            "invalid language code '{value}' (expected lowercase language code like 'fa', 'en', or 'de')"
        )));
    }
    if !normalized
        .bytes()
        .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
    {
        return Err(CliError::config(format!(
            "invalid language code '{value}' (expected lowercase language code like 'fa', 'en', or 'de')"
        )));
    }
    Ok(normalized)
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
        "Usage: lsteg <encode|decode|analyze|validate|languages|strategies|models|catalog|templates|profiles|schemas|data|demo|proto-encode|proto-decode>"
    )?;
    writeln!(
        writer,
        "       lsteg encode [--lang <code>] (--message <text> | --input <file>) [--source <id>] [--data-dir <path>] [--emit-trace] [--profile <id>] [--secret <value> | --secret-file <file>] [--format text|json] [--output <file>]"
    )?;
    writeln!(
        writer,
        "       lsteg decode [--lang auto|<code>] [--trace-input|--text-input] [--trace <text> | --input <file>] [--data-dir <path>] [--secret <value> | --secret-file <file>] [--format text|json] [--output <file>]"
    )?;
    writeln!(
        writer,
        "       lsteg analyze [--lang auto|<code>] [--trace-input|--text-input] [--trace <text> | --input <file>] [--data-dir <path>] [--secret <value> | --secret-file <file>] [--format text|json] [--output <file>]"
    )?;
    writeln!(
        writer,
        "       lsteg validate [--lang auto|<code>] [--trace-input|--text-input] [--trace <text> | --input <file>] [--data-dir <path>] [--secret <value> | --secret-file <file>] [--format text|json] [--output <file>]"
    )?;
    writeln!(writer, "       lsteg languages [--format text|json]")?;
    writeln!(writer, "       lsteg strategies [--format text|json]")?;
    writeln!(writer, "       lsteg models [--format text|json]")?;
    writeln!(
        writer,
        "       lsteg catalog [--lang <code>] [--format text|json]"
    )?;
    writeln!(
        writer,
        "       lsteg templates [--lang <code>] [--format text|json]"
    )?;
    writeln!(
        writer,
        "       lsteg profiles [--lang <code>] [--format text|json]"
    )?;
    writeln!(
        writer,
        "       lsteg schemas [--lang <code>] [--format text|json]"
    )?;
    writeln!(
        writer,
        "       lsteg data list [--lang <code>] [--data-dir <path>] [--format text|json]"
    )?;
    writeln!(
        writer,
        "       lsteg data status [--lang <code>] [--data-dir <path>] [--format text|json]"
    )?;
    writeln!(
        writer,
        "       lsteg data verify [--lang <code>] [--source <id>] [--data-dir <path>] [--format text|json]"
    )?;
    writeln!(
        writer,
        "       lsteg data doctor [--lang <code>] [--source <id>] [--fix] [--data-dir <path>] [--format text|json]"
    )?;
    writeln!(
        writer,
        "       lsteg data clean [--lang <code>] [--source <id>] [--apply] [--data-dir <path>] [--format text|json]"
    )?;
    writeln!(
        writer,
        "       lsteg data pin [--lang <code>] [--source <id>] [--checksum <sha256>] [--data-dir <path>] [--format text|json]"
    )?;
    writeln!(
        writer,
        "       lsteg data artifact validate --lang <code> --input <file> [--format text|json]"
    )?;
    writeln!(
        writer,
        "       lsteg data export-manifest [--lang <code>] [--source <id>] [--output <file>] [--data-dir <path>] [--format text|json]"
    )?;
    writeln!(
        writer,
        "       lsteg data import-manifest --input <file> [--lang <code>] [--source <id>] [--data-dir <path>] [--format text|json]"
    )?;
    writeln!(
        writer,
        "       lsteg data install --lang <code[,code...]> [--source <id>] [--artifact-url <url>] [--data-dir <path>] [--format text|json]"
    )?;
    writeln!(
        writer,
        "       lsteg data update --lang <code[,code...]> [--source <id>] [--artifact-url <url>] [--data-dir <path>] [--format text|json]"
    )?;
    writeln!(writer, "       lsteg demo <fa|en>")?;
    writeln!(
        writer,
        "       lsteg proto-encode <code> [message] [--json]"
    )?;
    writeln!(
        writer,
        "       lsteg proto-encode <code> [message] | lsteg proto-decode <code> [--json]"
    )?;
    writeln!(
        writer,
        "       lsteg proto-decode <code> \"<frame trace lines>\" [--json]"
    )?;
    writeln!(
        writer,
        "Env defaults: LSTEG_LANG (decode/analyze/validate accepts auto), LSTEG_FORMAT, LSTEG_INPUT, LSTEG_OUTPUT, LSTEG_ENCODE_MESSAGE, LSTEG_PROFILE, LSTEG_TRACE, LSTEG_SECRET, LSTEG_SECRET_FILE, LSTEG_DATA_DIR"
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_output_format_arg_sets_json_format() {
        let mut args = vec!["json".to_string()].into_iter();
        let mut format = OutputFormat::Text;

        parse_output_format_arg(&mut args, &mut format).expect("format parse should succeed");

        assert!(matches!(format, OutputFormat::Json));
    }

    #[test]
    fn parse_proto_lang_arg_sets_english_target() {
        let mut args = vec!["en".to_string()].into_iter();
        let mut target = ProtoTarget::Farsi;

        parse_proto_lang_arg(&mut args, &mut target).expect("lang parse should succeed");

        assert_eq!(target, ProtoTarget::English);
    }

    #[test]
    fn parse_proto_lang_arg_accepts_custom_language_code() {
        let mut args = vec!["de".to_string()].into_iter();
        let mut target = ProtoTarget::Farsi;

        parse_proto_lang_arg(&mut args, &mut target).expect("lang parse should succeed");

        assert_eq!(target, ProtoTarget::Other("de".to_string()));
    }

    #[test]
    fn parse_trace_lang_arg_sets_auto_detect_for_auto() {
        let mut args = vec!["auto".to_string()].into_iter();
        let mut target = ProtoTarget::English;
        let mut auto_detect_target = false;

        parse_trace_lang_arg(&mut args, &mut target, &mut auto_detect_target)
            .expect("trace lang parse should succeed");

        assert_eq!(target, ProtoTarget::Farsi);
        assert!(auto_detect_target);
    }

    #[test]
    fn parse_discovery_lang_arg_rejects_duplicate_lang_flag() {
        let mut args = vec!["fa".to_string()].into_iter();
        let mut target = None;
        let mut seen_lang = true;

        let error = parse_discovery_lang_arg(&mut args, &mut target, &mut seen_lang)
            .expect_err("duplicate lang should fail");

        assert_eq!(error.message(), "--lang cannot be provided multiple times");
    }

    #[test]
    fn encode_payload_args_rejects_missing_message_and_input() {
        let payload = EncodePayloadArgs {
            message: None,
            input_path: None,
            output_path: None,
            seen_message: false,
            seen_input: false,
            seen_output: false,
        };

        let error = payload
            .ensure_valid()
            .expect_err("missing message/input should fail");

        assert_eq!(
            error.message(),
            "encode requires --message <text> or --input <file>"
        );
    }

    #[test]
    fn secret_args_rejects_mixed_secret_and_secret_file() {
        let secrets = SecretArgs {
            secret: Some("value".to_string()),
            secret_file: Some("/tmp/secret.txt".to_string()),
            seen_secret: false,
            seen_secret_file: false,
            env_ambiguous: false,
        };

        let error = secrets
            .ensure_not_ambiguous("encode")
            .expect_err("mixed secret sources should fail");

        assert_eq!(
            error.message(),
            "encode accepts either --secret or --secret-file, not both"
        );
    }

    #[test]
    fn parse_decode_command_sets_trace_input_mode() {
        let command = parse_command(vec![
            "decode".to_string(),
            "--trace-input".to_string(),
            "--trace".to_string(),
            "frame 01 [fa-basic-sov] bits=0..18 values=subject:0,object:0,adjective:0,verb:21 => x"
                .to_string(),
        ])
        .expect("parse should succeed")
        .expect("command should exist");

        let Command::Decode(options) = command else {
            panic!("expected decode command");
        };

        assert!(matches!(options.input_mode, DecodeInputMode::Trace));
        assert!(options.trace.is_some());
    }

    #[test]
    fn parse_encode_command_sets_profile_option() {
        let command = parse_command(vec![
            "encode".to_string(),
            "--message".to_string(),
            "salam".to_string(),
            "--profile".to_string(),
            "fa-saadi-inspired-light".to_string(),
        ])
        .expect("parse should succeed")
        .expect("command should exist");

        let Command::Encode(options) = command else {
            panic!("expected encode command");
        };

        assert_eq!(options.profile.as_deref(), Some("fa-saadi-inspired-light"));
    }

    #[test]
    fn parse_encode_command_sets_source_and_data_dir() {
        let command = parse_command(vec![
            "encode".to_string(),
            "--lang".to_string(),
            "en".to_string(),
            "--message".to_string(),
            "hello".to_string(),
            "--source".to_string(),
            "en-wordlist-wordnik".to_string(),
            "--data-dir".to_string(),
            "/tmp/lsteg-data".to_string(),
        ])
        .expect("parse should succeed")
        .expect("command should exist");

        let Command::Encode(options) = command else {
            panic!("expected encode command");
        };

        assert_eq!(options.source_id.as_deref(), Some("en-wordlist-wordnik"));
        assert_eq!(options.data_dir.as_deref(), Some("/tmp/lsteg-data"));
    }

    #[test]
    fn parse_encode_command_rejects_duplicate_profile_flag() {
        let result = parse_command(vec![
            "encode".to_string(),
            "--message".to_string(),
            "salam".to_string(),
            "--profile".to_string(),
            "fa-neutral-formal".to_string(),
            "--profile".to_string(),
            "fa-saadi-inspired-light".to_string(),
        ]);
        let error = match result {
            Ok(_) => panic!("duplicate profile should fail"),
            Err(error) => error,
        };

        assert_eq!(
            error.message(),
            "--profile cannot be provided multiple times"
        );
    }

    #[test]
    fn parse_decode_command_sets_text_input_mode() {
        let command = parse_command(vec![
            "decode".to_string(),
            "--text-input".to_string(),
            "--trace".to_string(),
            "مرد کتاب زیبا را خرید.".to_string(),
        ])
        .expect("parse should succeed")
        .expect("command should exist");

        let Command::Decode(options) = command else {
            panic!("expected decode command");
        };

        assert!(matches!(options.input_mode, DecodeInputMode::Text));
        assert!(options.trace.is_some());
    }

    #[test]
    fn parse_decode_command_sets_data_dir() {
        let command = parse_command(vec![
            "decode".to_string(),
            "--trace-input".to_string(),
            "--trace".to_string(),
            "frame 01 [fa-basic-sov] bits=0..18 values=subject:0,object:0,adjective:0,verb:21 => x"
                .to_string(),
            "--data-dir".to_string(),
            "/tmp/lsteg-data".to_string(),
        ])
        .expect("parse should succeed")
        .expect("command should exist");

        let Command::Decode(options) = command else {
            panic!("expected decode command");
        };

        assert_eq!(options.data_dir.as_deref(), Some("/tmp/lsteg-data"));
    }

    #[test]
    fn parse_decode_command_rejects_mixed_trace_and_text_input_modes() {
        let result = parse_command(vec![
            "decode".to_string(),
            "--trace-input".to_string(),
            "--text-input".to_string(),
            "--trace".to_string(),
            "x".to_string(),
        ]);
        let error = match result {
            Ok(_) => panic!("mixed input modes should fail"),
            Err(error) => error,
        };

        assert_eq!(
            error.message(),
            "decode accepts either --trace-input or --text-input, not both"
        );
    }

    #[test]
    fn parse_analyze_command_sets_text_input_mode() {
        let command = parse_command(vec![
            "analyze".to_string(),
            "--text-input".to_string(),
            "--trace".to_string(),
            "مرد کتاب زیبا را خرید.".to_string(),
        ])
        .expect("parse should succeed")
        .expect("command should exist");

        let Command::Analyze(options) = command else {
            panic!("expected analyze command");
        };

        assert!(matches!(options.input_mode, DecodeInputMode::Text));
    }

    #[test]
    fn parse_validate_command_sets_trace_input_mode() {
        let command = parse_command(vec![
            "validate".to_string(),
            "--trace-input".to_string(),
            "--trace".to_string(),
            "frame 01 [fa-basic-sov] bits=0..18 values=subject:0,object:0,adjective:0,verb:21 => x"
                .to_string(),
        ])
        .expect("parse should succeed")
        .expect("command should exist");

        let Command::Validate(options) = command else {
            panic!("expected validate command");
        };

        assert!(matches!(options.input_mode, DecodeInputMode::Trace));
    }

    #[test]
    fn parse_proto_encode_command_accepts_custom_language_code() {
        let command = parse_command(vec![
            "proto-encode".to_string(),
            "de".to_string(),
            "hallo".to_string(),
        ])
        .expect("parse should succeed")
        .expect("command should exist");

        let Command::ProtoEncode(target, payload, json) = command else {
            panic!("expected proto encode command");
        };

        assert_eq!(target, ProtoTarget::Other("de".to_string()));
        assert_eq!(payload, "hallo".to_string());
        assert!(!json);
    }

    #[test]
    fn parse_proto_decode_command_accepts_custom_language_code() {
        let command = parse_command(vec![
            "proto-decode".to_string(),
            "de".to_string(),
            "frame 01 [de-basic] bits=0..1 values=x:0 => x".to_string(),
        ])
        .expect("parse should succeed")
        .expect("command should exist");

        let Command::ProtoDecode(target, trace, json) = command else {
            panic!("expected proto decode command");
        };

        assert_eq!(target, ProtoTarget::Other("de".to_string()));
        assert_eq!(
            trace.as_deref(),
            Some("frame 01 [de-basic] bits=0..1 values=x:0 => x")
        );
        assert!(!json);
    }

    #[test]
    fn parse_data_install_command_accepts_lang_list() {
        let command = parse_command(vec![
            "data".to_string(),
            "install".to_string(),
            "--lang".to_string(),
            "fa,en".to_string(),
        ])
        .expect("parse should succeed")
        .expect("command should exist");

        let Command::Data(DataCommand::Install(options)) = command else {
            panic!("expected data install command");
        };

        assert_eq!(options.targets.len(), 2);
        assert_eq!(options.targets[0], ProtoTarget::Farsi);
        assert_eq!(options.targets[1], ProtoTarget::English);
        assert!(options.source_id.is_none());
    }

    #[test]
    fn parse_data_install_command_requires_lang() {
        let result = parse_command(vec!["data".to_string(), "install".to_string()]);
        let error = match result {
            Ok(_) => panic!("data install without lang should fail"),
            Err(error) => error,
        };

        assert_eq!(
            error.message(),
            "data install requires --lang <code[,code...]>"
        );
    }

    #[test]
    fn parse_data_install_command_accepts_custom_language_code() {
        let command = parse_command(vec![
            "data".to_string(),
            "install".to_string(),
            "--lang".to_string(),
            "de,en".to_string(),
        ])
        .expect("parse should succeed")
        .expect("command should exist");

        let Command::Data(DataCommand::Install(options)) = command else {
            panic!("expected data install command");
        };

        assert_eq!(options.targets.len(), 2);
        assert_eq!(options.targets[0], ProtoTarget::Other("de".to_string()));
        assert_eq!(options.targets[1], ProtoTarget::English);
    }

    #[test]
    fn parse_data_list_command_sets_filter_and_data_dir() {
        let command = parse_command(vec![
            "data".to_string(),
            "list".to_string(),
            "--lang".to_string(),
            "en".to_string(),
            "--data-dir".to_string(),
            "/tmp/lsteg-data".to_string(),
            "--format".to_string(),
            "json".to_string(),
        ])
        .expect("parse should succeed")
        .expect("command should exist");

        let Command::Data(DataCommand::List(options)) = command else {
            panic!("expected data list command");
        };

        assert_eq!(options.target, Some(ProtoTarget::English));
        assert_eq!(options.data_dir.as_deref(), Some("/tmp/lsteg-data"));
        assert!(matches!(options.format, OutputFormat::Json));
    }

    #[test]
    fn parse_data_status_command_sets_filter_and_data_dir() {
        let command = parse_command(vec![
            "data".to_string(),
            "status".to_string(),
            "--lang".to_string(),
            "fa".to_string(),
            "--data-dir".to_string(),
            "/tmp/lsteg-status".to_string(),
            "--format".to_string(),
            "json".to_string(),
        ])
        .expect("parse should succeed")
        .expect("command should exist");

        let Command::Data(DataCommand::Status(options)) = command else {
            panic!("expected data status command");
        };

        assert_eq!(options.target, Some(ProtoTarget::Farsi));
        assert_eq!(options.data_dir.as_deref(), Some("/tmp/lsteg-status"));
        assert!(matches!(options.format, OutputFormat::Json));
    }

    #[test]
    fn parse_data_verify_command_sets_source_and_filter() {
        let command = parse_command(vec![
            "data".to_string(),
            "verify".to_string(),
            "--lang".to_string(),
            "en".to_string(),
            "--source".to_string(),
            "en-wordlist-wordnik".to_string(),
            "--data-dir".to_string(),
            "/tmp/lsteg-verify".to_string(),
            "--format".to_string(),
            "json".to_string(),
        ])
        .expect("parse should succeed")
        .expect("command should exist");

        let Command::Data(DataCommand::Verify(options)) = command else {
            panic!("expected data verify command");
        };

        assert_eq!(options.target, Some(ProtoTarget::English));
        assert_eq!(options.source_id.as_deref(), Some("en-wordlist-wordnik"));
        assert_eq!(options.data_dir.as_deref(), Some("/tmp/lsteg-verify"));
        assert!(matches!(options.format, OutputFormat::Json));
    }

    #[test]
    fn parse_data_doctor_command_sets_fix_and_source() {
        let command = parse_command(vec![
            "data".to_string(),
            "doctor".to_string(),
            "--lang".to_string(),
            "en".to_string(),
            "--source".to_string(),
            "en-wordlist-wordnik".to_string(),
            "--fix".to_string(),
            "--data-dir".to_string(),
            "/tmp/lsteg-doctor".to_string(),
            "--format".to_string(),
            "json".to_string(),
        ])
        .expect("parse should succeed")
        .expect("command should exist");

        let Command::Data(DataCommand::Doctor(options)) = command else {
            panic!("expected data doctor command");
        };

        assert_eq!(options.target, Some(ProtoTarget::English));
        assert_eq!(options.source_id.as_deref(), Some("en-wordlist-wordnik"));
        assert!(options.fix);
        assert_eq!(options.data_dir.as_deref(), Some("/tmp/lsteg-doctor"));
        assert!(matches!(options.format, OutputFormat::Json));
    }

    #[test]
    fn parse_data_clean_command_sets_apply_and_source() {
        let command = parse_command(vec![
            "data".to_string(),
            "clean".to_string(),
            "--lang".to_string(),
            "en".to_string(),
            "--source".to_string(),
            "en-wordlist-wordnik".to_string(),
            "--apply".to_string(),
            "--data-dir".to_string(),
            "/tmp/lsteg-clean".to_string(),
            "--format".to_string(),
            "json".to_string(),
        ])
        .expect("parse should succeed")
        .expect("command should exist");

        let Command::Data(DataCommand::Clean(options)) = command else {
            panic!("expected data clean command");
        };

        assert_eq!(options.target, Some(ProtoTarget::English));
        assert_eq!(options.source_id.as_deref(), Some("en-wordlist-wordnik"));
        assert!(options.apply);
        assert_eq!(options.data_dir.as_deref(), Some("/tmp/lsteg-clean"));
        assert!(matches!(options.format, OutputFormat::Json));
    }

    #[test]
    fn parse_data_pin_command_sets_checksum_and_source() {
        let command = parse_command(vec![
            "data".to_string(),
            "pin".to_string(),
            "--lang".to_string(),
            "en".to_string(),
            "--source".to_string(),
            "en-wordlist-wordnik".to_string(),
            "--checksum".to_string(),
            "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_string(),
            "--data-dir".to_string(),
            "/tmp/lsteg-pin".to_string(),
            "--format".to_string(),
            "json".to_string(),
        ])
        .expect("parse should succeed")
        .expect("command should exist");

        let Command::Data(DataCommand::Pin(options)) = command else {
            panic!("expected data pin command");
        };

        assert_eq!(options.target, Some(ProtoTarget::English));
        assert_eq!(options.source_id.as_deref(), Some("en-wordlist-wordnik"));
        assert_eq!(
            options.checksum_sha256.as_deref(),
            Some("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")
        );
        assert_eq!(options.data_dir.as_deref(), Some("/tmp/lsteg-pin"));
        assert!(matches!(options.format, OutputFormat::Json));
    }

    #[test]
    fn parse_data_artifact_validate_command_sets_lang_and_input() {
        let command = parse_command(vec![
            "data".to_string(),
            "artifact".to_string(),
            "validate".to_string(),
            "--lang".to_string(),
            "en".to_string(),
            "--input".to_string(),
            "/tmp/lexicon.json".to_string(),
            "--format".to_string(),
            "json".to_string(),
        ])
        .expect("parse should succeed")
        .expect("command should exist");

        let Command::Data(DataCommand::ArtifactValidate(options)) = command else {
            panic!("expected data artifact validate command");
        };

        assert_eq!(options.target, ProtoTarget::English);
        assert_eq!(options.input_path, "/tmp/lexicon.json");
        assert!(matches!(options.format, OutputFormat::Json));
    }

    #[test]
    fn parse_data_artifact_validate_command_requires_input() {
        let result = parse_command(vec![
            "data".to_string(),
            "artifact".to_string(),
            "validate".to_string(),
            "--lang".to_string(),
            "en".to_string(),
        ]);
        let error = match result {
            Ok(_) => panic!("missing input should fail"),
            Err(error) => error,
        };

        assert_eq!(
            error.message(),
            "data artifact validate requires --input <file>"
        );
    }

    #[test]
    fn parse_data_export_manifest_command_sets_output_and_source() {
        let command = parse_command(vec![
            "data".to_string(),
            "export-manifest".to_string(),
            "--lang".to_string(),
            "fa".to_string(),
            "--source".to_string(),
            "fa-wordlist-mit".to_string(),
            "--output".to_string(),
            "/tmp/lsteg-export.json".to_string(),
            "--data-dir".to_string(),
            "/tmp/lsteg-data".to_string(),
            "--format".to_string(),
            "json".to_string(),
        ])
        .expect("parse should succeed")
        .expect("command should exist");

        let Command::Data(DataCommand::ExportManifest(options)) = command else {
            panic!("expected data export-manifest command");
        };

        assert_eq!(options.target, Some(ProtoTarget::Farsi));
        assert_eq!(options.source_id.as_deref(), Some("fa-wordlist-mit"));
        assert_eq!(
            options.output_path.as_deref(),
            Some("/tmp/lsteg-export.json")
        );
        assert_eq!(options.data_dir.as_deref(), Some("/tmp/lsteg-data"));
        assert!(matches!(options.format, OutputFormat::Json));
    }

    #[test]
    fn parse_data_import_manifest_command_sets_input_and_source() {
        let command = parse_command(vec![
            "data".to_string(),
            "import-manifest".to_string(),
            "--input".to_string(),
            "/tmp/lsteg-export.json".to_string(),
            "--lang".to_string(),
            "en".to_string(),
            "--source".to_string(),
            "en-wordlist-wordnik".to_string(),
            "--data-dir".to_string(),
            "/tmp/lsteg-data".to_string(),
            "--format".to_string(),
            "json".to_string(),
        ])
        .expect("parse should succeed")
        .expect("command should exist");

        let Command::Data(DataCommand::ImportManifest(options)) = command else {
            panic!("expected data import-manifest command");
        };

        assert_eq!(options.target, Some(ProtoTarget::English));
        assert_eq!(options.source_id.as_deref(), Some("en-wordlist-wordnik"));
        assert_eq!(options.input_path, "/tmp/lsteg-export.json");
        assert_eq!(options.data_dir.as_deref(), Some("/tmp/lsteg-data"));
        assert!(matches!(options.format, OutputFormat::Json));
    }

    #[test]
    fn parse_data_update_command_sets_source_override() {
        let command = parse_command(vec![
            "data".to_string(),
            "update".to_string(),
            "--lang".to_string(),
            "fa".to_string(),
            "--source".to_string(),
            "fa-wordlist-mit".to_string(),
            "--artifact-url".to_string(),
            "file:///tmp/fa-words.txt".to_string(),
        ])
        .expect("parse should succeed")
        .expect("command should exist");

        let Command::Data(DataCommand::Update(options)) = command else {
            panic!("expected data update command");
        };

        assert_eq!(options.targets, vec![ProtoTarget::Farsi]);
        assert_eq!(options.source_id.as_deref(), Some("fa-wordlist-mit"));
        assert_eq!(
            options.artifact_url.as_deref(),
            Some("file:///tmp/fa-words.txt")
        );
    }
}
