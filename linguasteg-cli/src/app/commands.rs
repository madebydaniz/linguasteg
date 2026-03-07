use std::io::Read;

use linguasteg_core::{
    CryptoEnvelopeInspection, DecodeRequest, EncodeRequest, FixedWidthPlanningOptions, LanguageTag,
    RealizationPlan, SlotAssignment, SlotId, StyleInspiration, StyleProfileId,
    StyleProfileRegistry, StyleStrength, SymbolicFramePlan, SymbolicFrameSchema, TemplateId,
    TemplateRegistry, WritingRegister, inspect_envelope, open_payload, seal_payload,
};

use super::analysis::{analyze_trace_summary, render_trace_analysis_output};
use super::data::{
    resolve_active_data_source_selection, resolve_active_data_source_variant_catalog,
    resolve_effective_data_dir, run_data_command,
};
use super::dataset::LexiconVariantCatalog;
use super::formatters::{build_proto_decode_json, build_proto_encode_json, json_escape};
use super::language::resolve_trace_target;
use super::runtime::{
    PrototypeRuntime, initialize_runtime, supported_languages, supported_models,
    supported_strategies,
};
use super::symbol_mix::apply_secret_symbolic_mix;
use super::trace::{frame_sequence_error, parse_frames_from_trace, schema_for_template};
use super::types::{
    AnalyzeOptions, CatalogQueryOptions, CliError, Command, DecodeInputMode, DecodeOptions,
    DemoTarget, EncodeOptions, OutputFormat, ProfileQueryOptions, ProtoTarget, SchemaQueryOptions,
    TemplateQueryOptions, ValidateOptions,
};

pub(crate) fn execute(command: Command) -> Result<(), CliError> {
    match command {
        Command::Encode(options) => run_encode(options)?,
        Command::Decode(options) => run_decode(options)?,
        Command::Analyze(options) => run_analyze(options)?,
        Command::Validate(options) => run_validate(options)?,
        Command::Languages(format) => run_languages(format),
        Command::Strategies(format) => run_strategies(format),
        Command::Models(format) => run_models(format),
        Command::Catalog(options) => run_catalog(options)?,
        Command::Templates(options) => run_templates(options)?,
        Command::Profiles(options) => run_profiles(options)?,
        Command::Schemas(options) => run_schemas(options)?,
        Command::Data(command) => run_data_command(command)?,
        Command::Demo(DemoTarget::Farsi) => run_demo(ProtoTarget::Farsi)?,
        Command::Demo(DemoTarget::English) => run_demo(ProtoTarget::English)?,
        Command::Demo(DemoTarget::German) => run_demo(ProtoTarget::Other("de".to_string()))?,
        Command::Demo(DemoTarget::Italian) => run_demo(ProtoTarget::Other("it".to_string()))?,
        Command::ProtoEncode(target, payload_text, json) => {
            run_proto_encode(target, &payload_text, json)?
        }
        Command::ProtoDecode(target, trace_input, json) => {
            run_proto_decode(target, trace_input, json)?
        }
    }

    Ok(())
}

fn run_catalog(options: CatalogQueryOptions) -> Result<(), CliError> {
    let language_filter = options.target.as_ref().map(ProtoTarget::as_str);
    let all_languages = supported_languages();
    let all_models = supported_models();
    let languages = all_languages
        .iter()
        .filter(|item| language_filter.is_none_or(|code| item.code == code))
        .collect::<Vec<_>>();
    let strategies = supported_strategies();
    let models = all_models
        .iter()
        .filter(|item| {
            language_filter
                .is_none_or(|code| item.languages.iter().any(|language| *language == code))
        })
        .collect::<Vec<_>>();
    let template_items = collect_template_items(options.target.clone())?;
    let profile_items = collect_profile_items(options.target.clone())?;
    let schema_items = collect_schema_items(options.target)?;

    if matches!(options.format, OutputFormat::Json) {
        let language_items = languages
            .iter()
            .map(|item| {
                format!(
                    "{{\"code\":\"{}\",\"display\":\"{}\",\"direction\":\"{}\"}}",
                    json_escape(item.code),
                    json_escape(item.display),
                    json_escape(item.direction)
                )
            })
            .collect::<Vec<_>>()
            .join(",");
        let strategy_items = strategies
            .iter()
            .map(|item| {
                let capabilities = item
                    .required_capabilities
                    .iter()
                    .map(|capability| format!("\"{}\"", json_escape(capability)))
                    .collect::<Vec<_>>()
                    .join(",");
                format!(
                    "{{\"id\":\"{}\",\"display\":\"{}\",\"required_capabilities\":[{}]}}",
                    json_escape(item.id),
                    json_escape(item.display),
                    capabilities
                )
            })
            .collect::<Vec<_>>()
            .join(",");
        let model_items = models
            .iter()
            .map(|item| {
                let languages = item
                    .languages
                    .iter()
                    .map(|language| format!("\"{}\"", json_escape(language)))
                    .collect::<Vec<_>>()
                    .join(",");
                let capabilities = item
                    .capabilities
                    .iter()
                    .map(|capability| format!("\"{}\"", json_escape(capability)))
                    .collect::<Vec<_>>()
                    .join(",");
                format!(
                    "{{\"provider\":\"{}\",\"id\":\"{}\",\"display\":\"{}\",\"languages\":[{}],\"capabilities\":[{}]}}",
                    json_escape(item.provider),
                    json_escape(item.id),
                    json_escape(item.display),
                    languages,
                    capabilities
                )
            })
            .collect::<Vec<_>>()
            .join(",");
        let template_json_items = template_items
            .iter()
            .map(|item| {
                format!(
                    "{{\"language\":\"{}\",\"language_display\":\"{}\",\"id\":\"{}\",\"display\":\"{}\",\"slot_count\":{}}}",
                    json_escape(&item.language_code),
                    json_escape(&item.language_display),
                    json_escape(&item.template_id),
                    json_escape(&item.template_display),
                    item.slot_count
                )
            })
            .collect::<Vec<_>>()
            .join(",");
        let profile_json_items = profile_items
            .iter()
            .map(|item| {
                let inspiration_label = item.inspiration_label.as_ref().map_or_else(
                    || "null".to_string(),
                    |value| format!("\"{}\"", json_escape(value)),
                );
                format!(
                    "{{\"language\":\"{}\",\"language_display\":\"{}\",\"id\":\"{}\",\"display\":\"{}\",\"register\":\"{}\",\"strength\":\"{}\",\"inspiration_kind\":\"{}\",\"inspiration_label\":{}}}",
                    json_escape(&item.language_code),
                    json_escape(&item.language_display),
                    json_escape(&item.profile_id),
                    json_escape(&item.profile_display),
                    json_escape(register_label(item.register)),
                    json_escape(strength_label(item.strength)),
                    json_escape(&item.inspiration_kind),
                    inspiration_label
                )
            })
            .collect::<Vec<_>>()
            .join(",");
        let schema_json_items = schema_items
            .iter()
            .map(|item| {
                let fields = item
                    .fields
                    .iter()
                    .map(|field| {
                        format!(
                            "{{\"slot\":\"{}\",\"bit_width\":{}}}",
                            json_escape(&field.slot),
                            field.bit_width
                        )
                    })
                    .collect::<Vec<_>>()
                    .join(",");
                format!(
                    "{{\"language\":\"{}\",\"language_display\":\"{}\",\"template_id\":\"{}\",\"total_bits\":{},\"fields\":[{}]}}",
                    json_escape(&item.language_code),
                    json_escape(&item.language_display),
                    json_escape(&item.template_id),
                    item.total_bits,
                    fields
                )
            })
            .collect::<Vec<_>>()
            .join(",");

        println!(
            "{{\"mode\":\"catalog\",\"languages\":[{}],\"strategies\":[{}],\"models\":[{}],\"templates\":[{}],\"profiles\":[{}],\"schemas\":[{}]}}",
            language_items,
            strategy_items,
            model_items,
            template_json_items,
            profile_json_items,
            schema_json_items
        );
        return Ok(());
    }

    println!("catalog:");
    println!("languages:");
    for item in languages {
        println!("- {} ({}, {})", item.code, item.display, item.direction);
    }
    println!("strategies:");
    for item in strategies {
        let capabilities = if item.required_capabilities.is_empty() {
            "<none>".to_string()
        } else {
            item.required_capabilities.join(",")
        };
        println!(
            "- {} ({}) capabilities: {}",
            item.id, item.display, capabilities
        );
    }
    println!("models:");
    for item in models {
        let languages = if item.languages.is_empty() {
            "<none>".to_string()
        } else {
            item.languages.join(",")
        };
        let capabilities = if item.capabilities.is_empty() {
            "<none>".to_string()
        } else {
            item.capabilities.join(",")
        };
        println!(
            "- {}/{} ({}) languages: {} capabilities: {}",
            item.provider, item.id, item.display, languages, capabilities
        );
    }
    println!("templates:");
    for item in &template_items {
        println!(
            "- {}/{} ({}) slots: {}",
            item.language_code, item.template_id, item.template_display, item.slot_count
        );
    }
    println!("profiles:");
    for item in &profile_items {
        let inspiration_label = item
            .inspiration_label
            .as_ref()
            .map_or("<none>", String::as_str);
        println!(
            "- {}/{} ({}) register: {} strength: {} inspiration: {} ({})",
            item.language_code,
            item.profile_id,
            item.profile_display,
            register_label(item.register),
            strength_label(item.strength),
            item.inspiration_kind,
            inspiration_label
        );
    }
    println!("schemas:");
    for item in &schema_items {
        let fields = item
            .fields
            .iter()
            .map(|field| format!("{}:{}", field.slot, field.bit_width))
            .collect::<Vec<_>>()
            .join(",");
        println!(
            "- {}/{} total_bits: {} fields: {}",
            item.language_code, item.template_id, item.total_bits, fields
        );
    }

    Ok(())
}

struct TemplateCatalogItem {
    language_code: String,
    language_display: String,
    template_id: String,
    template_display: String,
    slot_count: usize,
}

fn run_templates(options: TemplateQueryOptions) -> Result<(), CliError> {
    let items = collect_template_items(options.target)?;

    if matches!(options.format, OutputFormat::Json) {
        let json_items = items
            .iter()
            .map(|item| {
                format!(
                    "{{\"language\":\"{}\",\"language_display\":\"{}\",\"id\":\"{}\",\"display\":\"{}\",\"slot_count\":{}}}",
                    json_escape(&item.language_code),
                    json_escape(&item.language_display),
                    json_escape(&item.template_id),
                    json_escape(&item.template_display),
                    item.slot_count
                )
            })
            .collect::<Vec<_>>()
            .join(",");
        println!("{{\"mode\":\"templates\",\"items\":[{json_items}]}}");
        return Ok(());
    }

    println!("supported templates:");
    for item in items {
        println!(
            "- {}/{} ({}) slots: {}",
            item.language_code, item.template_id, item.template_display, item.slot_count
        );
    }

    Ok(())
}

struct ProfileCatalogItem {
    language_code: String,
    language_display: String,
    profile_id: String,
    profile_display: String,
    register: WritingRegister,
    strength: StyleStrength,
    inspiration_kind: String,
    inspiration_label: Option<String>,
}

struct SchemaFieldItem {
    slot: String,
    bit_width: u8,
}

struct SchemaCatalogItem {
    language_code: String,
    language_display: String,
    template_id: String,
    total_bits: usize,
    fields: Vec<SchemaFieldItem>,
}

fn selected_targets(target: Option<ProtoTarget>) -> Vec<ProtoTarget> {
    match target {
        Some(value) => vec![value],
        None => supported_languages()
            .iter()
            .map(|item| ProtoTarget::from_language_code(item.code))
            .collect(),
    }
}

fn for_each_runtime(
    target: Option<ProtoTarget>,
    mut visitor: impl FnMut(&PrototypeRuntime) -> Result<(), CliError>,
) -> Result<(), CliError> {
    for target in selected_targets(target) {
        let runtime = runtime_for_target(target)?;
        visitor(&runtime)?;
    }
    Ok(())
}

fn collect_template_items(
    target: Option<ProtoTarget>,
) -> Result<Vec<TemplateCatalogItem>, CliError> {
    let mut items = Vec::new();

    for_each_runtime(target, |runtime| {
        let language = map_domain(
            LanguageTag::new(runtime.language_code),
            "invalid language tag",
        )?;
        for template in runtime.pack.templates_for_language(&language) {
            items.push(TemplateCatalogItem {
                language_code: runtime.language_code.to_string(),
                language_display: runtime.language_display.to_string(),
                template_id: template.id.to_string(),
                template_display: template.display_name.clone(),
                slot_count: template.slots.len(),
            });
        }
        Ok(())
    })?;

    items.sort_by(|left, right| {
        (&left.language_code, &left.template_id).cmp(&(&right.language_code, &right.template_id))
    });

    Ok(items)
}

fn collect_profile_items(target: Option<ProtoTarget>) -> Result<Vec<ProfileCatalogItem>, CliError> {
    let mut items = Vec::new();

    for_each_runtime(target, |runtime| {
        let language = map_domain(
            LanguageTag::new(runtime.language_code),
            "invalid language tag",
        )?;
        for profile in runtime.pack.style_profiles_for_language(&language) {
            let (inspiration_kind, inspiration_label) = inspiration_metadata(&profile.inspiration);
            items.push(ProfileCatalogItem {
                language_code: runtime.language_code.to_string(),
                language_display: runtime.language_display.to_string(),
                profile_id: profile.id.to_string(),
                profile_display: profile.display_name.clone(),
                register: profile.register,
                strength: profile.strength,
                inspiration_kind,
                inspiration_label,
            });
        }
        Ok(())
    })?;

    items.sort_by(|left, right| {
        (&left.language_code, &left.profile_id).cmp(&(&right.language_code, &right.profile_id))
    });

    Ok(items)
}

fn collect_schema_items(target: Option<ProtoTarget>) -> Result<Vec<SchemaCatalogItem>, CliError> {
    let mut items = Vec::new();

    for_each_runtime(target, |runtime| {
        let schemas = runtime.mapper.frame_schemas();
        for schema in schemas {
            let fields = schema
                .fields
                .iter()
                .map(|field| SchemaFieldItem {
                    slot: field.slot.to_string(),
                    bit_width: field.bit_width,
                })
                .collect::<Vec<_>>();
            items.push(SchemaCatalogItem {
                language_code: runtime.language_code.to_string(),
                language_display: runtime.language_display.to_string(),
                template_id: schema.template_id.to_string(),
                total_bits: schema.total_bits(),
                fields,
            });
        }
        Ok(())
    })?;

    items.sort_by(|left, right| {
        (&left.language_code, &left.template_id).cmp(&(&right.language_code, &right.template_id))
    });
    Ok(items)
}

fn run_profiles(options: ProfileQueryOptions) -> Result<(), CliError> {
    let items = collect_profile_items(options.target)?;

    if matches!(options.format, OutputFormat::Json) {
        let json_items = items
            .iter()
            .map(|item| {
                let inspiration_label = item.inspiration_label.as_ref().map_or_else(
                    || "null".to_string(),
                    |value| format!("\"{}\"", json_escape(value)),
                );
                format!(
                    "{{\"language\":\"{}\",\"language_display\":\"{}\",\"id\":\"{}\",\"display\":\"{}\",\"register\":\"{}\",\"strength\":\"{}\",\"inspiration_kind\":\"{}\",\"inspiration_label\":{}}}",
                    json_escape(&item.language_code),
                    json_escape(&item.language_display),
                    json_escape(&item.profile_id),
                    json_escape(&item.profile_display),
                    json_escape(register_label(item.register)),
                    json_escape(strength_label(item.strength)),
                    json_escape(&item.inspiration_kind),
                    inspiration_label
                )
            })
            .collect::<Vec<_>>()
            .join(",");
        println!("{{\"mode\":\"profiles\",\"items\":[{json_items}]}}");
        return Ok(());
    }

    println!("supported profiles:");
    for item in items {
        let inspiration_label = item
            .inspiration_label
            .as_ref()
            .map_or("<none>", String::as_str);
        println!(
            "- {}/{} ({}) register: {} strength: {} inspiration: {} ({})",
            item.language_code,
            item.profile_id,
            item.profile_display,
            register_label(item.register),
            strength_label(item.strength),
            item.inspiration_kind,
            inspiration_label
        );
    }

    Ok(())
}

fn run_schemas(options: SchemaQueryOptions) -> Result<(), CliError> {
    let items = collect_schema_items(options.target)?;

    if matches!(options.format, OutputFormat::Json) {
        let json_items = items
            .iter()
            .map(|item| {
                let fields = item
                    .fields
                    .iter()
                    .map(|field| {
                        format!(
                            "{{\"slot\":\"{}\",\"bit_width\":{}}}",
                            json_escape(&field.slot),
                            field.bit_width
                        )
                    })
                    .collect::<Vec<_>>()
                    .join(",");
                format!(
                    "{{\"language\":\"{}\",\"language_display\":\"{}\",\"template_id\":\"{}\",\"total_bits\":{},\"fields\":[{}]}}",
                    json_escape(&item.language_code),
                    json_escape(&item.language_display),
                    json_escape(&item.template_id),
                    item.total_bits,
                    fields
                )
            })
            .collect::<Vec<_>>()
            .join(",");
        println!("{{\"mode\":\"schemas\",\"items\":[{json_items}]}}");
        return Ok(());
    }

    println!("supported schemas:");
    for item in items {
        let fields = item
            .fields
            .iter()
            .map(|field| format!("{}:{}", field.slot, field.bit_width))
            .collect::<Vec<_>>()
            .join(",");
        println!(
            "- {}/{} total_bits: {} fields: {}",
            item.language_code, item.template_id, item.total_bits, fields
        );
    }

    Ok(())
}

fn register_label(register: WritingRegister) -> &'static str {
    match register {
        WritingRegister::Neutral => "neutral",
        WritingRegister::Formal => "formal",
        WritingRegister::Colloquial => "colloquial",
        WritingRegister::Literary => "literary",
        WritingRegister::Academic => "academic",
    }
}

fn strength_label(strength: StyleStrength) -> &'static str {
    match strength {
        StyleStrength::Light => "light",
        StyleStrength::Medium => "medium",
        StyleStrength::Strong => "strong",
    }
}

fn inspiration_metadata(inspiration: &StyleInspiration) -> (String, Option<String>) {
    match inspiration {
        StyleInspiration::Neutral => ("neutral".to_string(), None),
        StyleInspiration::RegisterOnly => ("register-only".to_string(), None),
        StyleInspiration::EraInspired { era_label } => {
            ("era-inspired".to_string(), Some(era_label.clone()))
        }
        StyleInspiration::PublicDomainAuthorInspired { author_label } => {
            ("author-inspired".to_string(), Some(author_label.clone()))
        }
    }
}

fn run_models(format: OutputFormat) {
    let items = supported_models();
    if matches!(format, OutputFormat::Json) {
        let json_items = items
            .iter()
            .map(|item| {
                let languages = item
                    .languages
                    .iter()
                    .map(|language| format!("\"{}\"", json_escape(language)))
                    .collect::<Vec<_>>()
                    .join(",");
                let capabilities = item
                    .capabilities
                    .iter()
                    .map(|capability| format!("\"{}\"", json_escape(capability)))
                    .collect::<Vec<_>>()
                    .join(",");
                format!(
                    "{{\"provider\":\"{}\",\"id\":\"{}\",\"display\":\"{}\",\"languages\":[{}],\"capabilities\":[{}]}}",
                    json_escape(item.provider),
                    json_escape(item.id),
                    json_escape(item.display),
                    languages,
                    capabilities
                )
            })
            .collect::<Vec<_>>()
            .join(",");
        println!("{{\"mode\":\"models\",\"items\":[{json_items}]}}");
        return;
    }

    println!("supported models:");
    for item in items {
        let languages = if item.languages.is_empty() {
            "<none>".to_string()
        } else {
            item.languages.join(",")
        };
        let capabilities = if item.capabilities.is_empty() {
            "<none>".to_string()
        } else {
            item.capabilities.join(",")
        };
        println!(
            "- {}/{} ({}) languages: {} capabilities: {}",
            item.provider, item.id, item.display, languages, capabilities
        );
    }
}

fn run_strategies(format: OutputFormat) {
    let items = supported_strategies();
    if matches!(format, OutputFormat::Json) {
        let json_items = items
            .iter()
            .map(|item| {
                let capabilities = item
                    .required_capabilities
                    .iter()
                    .map(|capability| format!("\"{}\"", json_escape(capability)))
                    .collect::<Vec<_>>()
                    .join(",");
                format!(
                    "{{\"id\":\"{}\",\"display\":\"{}\",\"required_capabilities\":[{}]}}",
                    json_escape(item.id),
                    json_escape(item.display),
                    capabilities
                )
            })
            .collect::<Vec<_>>()
            .join(",");
        println!("{{\"mode\":\"strategies\",\"items\":[{json_items}]}}");
        return;
    }

    println!("supported strategies:");
    for item in items {
        let capabilities = if item.required_capabilities.is_empty() {
            "<none>".to_string()
        } else {
            item.required_capabilities.join(",")
        };
        println!(
            "- {} ({}) capabilities: {}",
            item.id, item.display, capabilities
        );
    }
}

fn run_languages(format: OutputFormat) {
    let items = supported_languages();
    if matches!(format, OutputFormat::Json) {
        let json_items = items
            .iter()
            .map(|item| {
                format!(
                    "{{\"code\":\"{}\",\"display\":\"{}\",\"direction\":\"{}\"}}",
                    json_escape(item.code),
                    json_escape(item.display),
                    json_escape(item.direction)
                )
            })
            .collect::<Vec<_>>()
            .join(",");
        println!("{{\"mode\":\"languages\",\"items\":[{json_items}]}}");
        return;
    }

    println!("supported languages:");
    for item in items {
        println!("- {} ({}, {})", item.code, item.display, item.direction);
    }
}

fn run_encode(options: EncodeOptions) -> Result<(), CliError> {
    let payload_text = resolve_encode_payload(&options)?;
    let secret = resolve_required_secret_bytes(
        options.secret.as_deref(),
        options.secret_file.as_deref(),
        "encode",
    )?;
    let active_data_source = resolve_active_data_source_selection(
        options.target.clone(),
        options.source_id.as_deref(),
        options.data_dir.as_deref(),
    )?;
    let active_variant_catalog = resolve_active_data_source_variant_catalog(
        options.target.clone(),
        active_data_source
            .as_ref()
            .map(|source| source.source_id.as_str()),
        options.data_dir.as_deref(),
    )?;
    emit_dataset_hint_if_unavailable(
        &options.target,
        active_data_source
            .as_ref()
            .map(|source| source.source_id.as_str()),
        active_variant_catalog.as_ref(),
        options.data_dir.as_deref(),
    );
    let output = render_proto_encode_output(
        options.target,
        &payload_text,
        options.format,
        Some(&secret),
        options.emit_trace,
        options.profile.as_deref(),
        active_data_source
            .as_ref()
            .map(|source| source.source_id.as_str()),
        active_variant_catalog.as_ref(),
    )?;
    write_output(&output, options.output_path.as_deref())
}

fn emit_dataset_hint_if_unavailable(
    target: &ProtoTarget,
    active_source_id: Option<&str>,
    variant_catalog: Option<&LexiconVariantCatalog>,
    data_dir: Option<&str>,
) {
    if variant_catalog.is_some() {
        return;
    }

    let language_code = target.as_str();
    if let Some(source_id) = active_source_id {
        let starter_path = resolve_effective_data_dir(data_dir)
            .join(language_code)
            .join(source_id)
            .join("dataset.json");
        eprintln!(
            "notice: dataset variants are not active for language '{}'. edit '{}' then run: lsteg data update --lang {} --source {}",
            language_code,
            starter_path.to_string_lossy(),
            language_code,
            source_id
        );
        return;
    }

    eprintln!(
        "notice: no dataset source is installed for language '{}'. for better lexical variation run: lsteg data install --lang {} --download",
        language_code, language_code
    );
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
        options.input_mode,
        &trace_text,
        options.format,
        Some(&secret),
        options.data_dir.as_deref(),
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
        options.input_mode,
        &trace_text,
        options.format,
        secret.as_deref(),
        options.data_dir.as_deref(),
    )?;
    write_output(&output, options.output_path.as_deref())
}

fn run_validate(options: ValidateOptions) -> Result<(), CliError> {
    let trace_text = resolve_trace_input(options.trace.as_deref(), options.input_path.as_deref())?;
    let secret =
        resolve_optional_secret_bytes(options.secret.as_deref(), options.secret_file.as_deref())?;
    let summary = analyze_trace_summary(
        "validate",
        options.target,
        options.auto_detect_target,
        options.input_mode,
        &trace_text,
        secret.as_deref(),
        options.data_dir.as_deref(),
    )?;
    let output = render_validate_output(&summary, options.format);
    write_output(&output, options.output_path.as_deref())?;

    if summary.integrity_ok {
        Ok(())
    } else {
        let reason = summary
            .integrity_error
            .clone()
            .unwrap_or_else(|| "trace integrity check failed".to_string());
        Err(CliError::trace(format!("validation failed: {reason}")))
    }
}

fn render_validate_output(
    summary: &super::types::TraceAnalysisSummary,
    format: OutputFormat,
) -> String {
    if matches!(format, OutputFormat::Json) {
        let integrity_error = summary.integrity_error.as_ref().map_or_else(
            || "null".to_string(),
            |value| format!("\"{}\"", json_escape(value)),
        );
        return format!(
            "{{\"mode\":\"validate\",\"language\":\"{}\",\"frame_count\":{},\"contiguous_ranges\":{},\"integrity_ok\":{},\"integrity_error\":{}}}",
            json_escape(summary.language),
            summary.frame_count,
            summary.contiguous_ranges,
            summary.integrity_ok,
            integrity_error
        );
    }

    let mut lines = Vec::new();
    lines.push(format!("{} trace validation", summary.language_display));
    lines.push(format!("language: {}", summary.language));
    lines.push(format!("frames: {}", summary.frame_count));
    lines.push(format!(
        "contiguous ranges: {}",
        if summary.contiguous_ranges {
            "yes"
        } else {
            "no"
        }
    ));
    lines.push(format!(
        "integrity: {}",
        if summary.integrity_ok { "ok" } else { "failed" }
    ));
    if let Some(error) = &summary.integrity_error {
        lines.push(format!("error: {error}"));
    }
    lines.join("\n")
}

fn run_demo(target: ProtoTarget) -> Result<(), CliError> {
    let runtime = runtime_for_target(target.clone())?;
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
        TemplateId::new(time_location_template_id(&target)?),
        "invalid template identifier",
    )?;
    let template = runtime
        .pack
        .template(&template_id)
        .ok_or_else(|| CliError::domain(format!("missing template: {template_id}")))?;

    let valid_plan = RealizationPlan {
        template_id: template_id.clone(),
        assignments: demo_assignments(&target, true)?
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
        assignments: demo_assignments(&target, false)?
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
    target: &ProtoTarget,
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
        (ProtoTarget::Other(code), true) if code == "de" => vec![
            ("subject", "der student"),
            ("time", "heute"),
            ("location", "in der bibliothek"),
            ("verb", "schreibt"),
            ("object", "brief"),
        ],
        (ProtoTarget::Other(code), false) if code == "de" => vec![
            ("subject", " "),
            ("time", "heute"),
            ("location", "in der bibliothek"),
            ("verb", "schreibt"),
            ("object", "brief"),
        ],
        (ProtoTarget::Other(code), true) if code == "it" => vec![
            ("subject", "lo studente"),
            ("time", "oggi"),
            ("location", "in biblioteca"),
            ("verb", "scrive"),
            ("object", "lettera"),
        ],
        (ProtoTarget::Other(code), false) if code == "it" => vec![
            ("subject", " "),
            ("time", "oggi"),
            ("location", "in biblioteca"),
            ("verb", "scrive"),
            ("object", "lettera"),
        ],
        (ProtoTarget::Other(_), _) => {
            return Err(CliError::config(
                "demo supports only built-in targets: fa, en, de, it".to_string(),
            ));
        }
    };
    Ok(data)
}

fn time_location_template_id(target: &ProtoTarget) -> Result<&'static str, CliError> {
    match target {
        ProtoTarget::Farsi => Ok("fa-time-location-sov"),
        ProtoTarget::English => Ok("en-time-location-svo"),
        ProtoTarget::Other(code) if code == "de" => Ok("de-time-location-svo"),
        ProtoTarget::Other(code) if code == "it" => Ok("it-time-location-svo"),
        ProtoTarget::Other(code) => Err(CliError::config(format!(
            "no demo template is registered for language '{code}'"
        ))),
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
    let output =
        render_proto_encode_output(target, payload_text, format, None, true, None, None, None)?;
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
    let output = render_proto_decode_output(
        target,
        false,
        DecodeInputMode::Trace,
        &trace_text,
        format,
        None,
        None,
    )?;
    println!("{output}");
    Ok(())
}

fn render_proto_encode_output(
    target: ProtoTarget,
    payload_text: &str,
    format: OutputFormat,
    secret: Option<&[u8]>,
    emit_trace: bool,
    profile: Option<&str>,
    data_source_id: Option<&str>,
    variant_catalog: Option<&LexiconVariantCatalog>,
) -> Result<String, CliError> {
    let payload = payload_text.as_bytes();
    let symbolic_payload = match secret {
        Some(secret) => seal_payload(payload, secret)
            .map_err(|_| CliError::security("failed to encrypt payload with provided secret"))?,
        None => payload.to_vec(),
    };
    let runtime = runtime_for_target(target)?;
    let profile_id = resolve_encode_profile_id(&runtime, profile)?;
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
    let mut payload_plan = orchestration.symbolic_plan;
    if let Some(secret) = secret {
        apply_secret_symbolic_mix(&mut payload_plan.frames, secret);
    }
    let mut realization_plans = map_domain(
        runtime
            .mapper
            .map_payload_to_plans_with_profile(&payload_plan, profile_id.as_ref()),
        "failed to map payload to realization plans",
    )?;
    apply_secret_surface_variants(
        runtime.language_code,
        &mut realization_plans,
        secret,
        data_source_id,
        variant_catalog,
    );

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
    if matches!(format, OutputFormat::Json) {
        return Ok(build_proto_encode_json(
            runtime.language_code,
            payload_text,
            profile_id.as_ref().map(StyleProfileId::as_str),
            payload.len(),
            payload_plan.encoded_len_bytes,
            payload_plan.padding_bits,
            &payload_plan.frames,
            &sentences,
            &final_text,
            None,
        ));
    }

    if !emit_trace {
        return Ok(final_text);
    }

    let mut report_lines = Vec::new();
    report_lines.push(format!("{} prototype encode", runtime.language_display));
    report_lines.push(format!("input text: {payload_text}"));
    if let Some(profile_id) = &profile_id {
        report_lines.push(format!("style profile: {profile_id}"));
    }
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

    Ok(report_lines.join("\n"))
}

fn apply_secret_surface_variants(
    language_code: &str,
    plans: &mut [RealizationPlan],
    secret: Option<&[u8]>,
    data_source_id: Option<&str>,
    variant_catalog: Option<&LexiconVariantCatalog>,
) {
    let Some(secret_bytes) = secret else {
        return;
    };
    if secret_bytes.is_empty() {
        return;
    }

    let data_source_mix = data_source_id
        .map(|source_id| fnv1a64(source_id.as_bytes()))
        .unwrap_or(0);
    let data_source_len = data_source_id.map_or(0, str::len);
    let seed = fnv1a64(secret_bytes) ^ data_source_mix.rotate_left(11);
    let secret_len = u64::try_from(secret_bytes.len()).unwrap_or(u64::MAX);
    let intro_seed = seed
        ^ secret_len.wrapping_mul(0xA24B_AED4_963E_E407)
        ^ 0xC6A4_A793_5BD1_E995
        ^ data_source_mix.rotate_right(9);
    for (frame_index, plan) in plans.iter_mut().enumerate() {
        for assignment in &mut plan.assignments {
            let slot = assignment.slot.as_str();
            let selector = surface_selector(seed, frame_index, slot, &assignment.surface);
            if frame_index == 0 {
                if let Some(variant) = dataset_surface_variant(
                    variant_catalog,
                    slot,
                    assignment.surface.as_str(),
                    selector,
                ) {
                    assignment.surface = variant;
                    continue;
                }
            }
            if frame_index == 0 {
                if let Some(variant) = secret_intro_surface_variant(
                    language_code,
                    slot,
                    assignment.surface.as_str(),
                    intro_seed,
                    secret_bytes.len(),
                    data_source_len,
                ) {
                    assignment.surface = variant.to_string();
                    continue;
                }
            }
            if !should_use_secret_variant(seed, frame_index, slot, &assignment.surface) {
                continue;
            }
            let selector = surface_selector(seed, frame_index, slot, &assignment.surface);
            if let Some(variant) = dataset_surface_variant(
                variant_catalog,
                slot,
                assignment.surface.as_str(),
                selector,
            ) {
                assignment.surface = variant;
                continue;
            }
            if let Some(variant) =
                secret_surface_variant(language_code, slot, assignment.surface.as_str())
            {
                assignment.surface = variant.to_string();
            }
        }
    }
}

fn secret_intro_surface_variant(
    language_code: &str,
    slot: &str,
    surface: &str,
    intro_seed: u64,
    secret_len: usize,
    data_source_len: usize,
) -> Option<&'static str> {
    let selector = intro_seed
        ^ fnv1a64(slot.as_bytes()).rotate_left(19)
        ^ fnv1a64(surface.as_bytes()).rotate_left(7);
    match (language_code, slot, surface) {
        ("en", "object", "letter") => Some(if data_source_len > 0 {
            if (((selector & 1) as usize) ^ (data_source_len & 1)) == 0 {
                "letter"
            } else {
                "missive"
            }
        } else {
            match selector % 3 {
                0 => "letter",
                1 => "missive",
                _ => "epistle",
            }
        }),
        ("en", "adjective", "quiet") => Some(if (selector & 1) == 0 {
            "quiet"
        } else {
            "concise"
        }),
        ("en", "verb", "writes") => Some(if (selector & 1) == 0 {
            "writes"
        } else {
            "composes"
        }),
        ("fa", "object", "نامه") => Some(if (secret_len & 1) == 0 {
            "نامه"
        } else {
            "مکتوب"
        }),
        ("fa", "object", "پیام") => Some(if (secret_len & 1) == 0 {
            "پیام"
        } else {
            "پیغام"
        }),
        ("fa", "object", "داستان") => Some(if (secret_len & 1) == 0 {
            "داستان"
        } else {
            "حکایت"
        }),
        ("fa", "object", "غذا") => Some(if (secret_len & 1) == 0 {
            "غذا"
        } else {
            "طعام"
        }),
        ("fa", "adjective", "زیبا") => Some(if (selector & 1) == 0 {
            "زیبا"
        } else {
            "خوش"
        }),
        ("fa", "adjective", "قدیمی") => Some(if (selector & 1) == 0 {
            "قدیمی"
        } else {
            "کهن"
        }),
        ("fa", "adjective", "تازه") => Some(if (selector & 1) == 0 {
            "تازه"
        } else {
            "نو"
        }),
        ("fa", "verb", "نوشت") => Some(if (selector & 1) == 0 {
            "نوشت"
        } else {
            "نگاشت"
        }),
        ("fa", "verb", "دید") => Some(if (selector & 1) == 0 {
            "دید"
        } else {
            "نگریست"
        }),
        ("fa", "verb", "گفت") => Some(if (selector & 1) == 0 {
            "گفت"
        } else {
            "فرمود"
        }),
        _ => None,
    }
}

fn secret_surface_variant(language_code: &str, slot: &str, surface: &str) -> Option<&'static str> {
    match (language_code, slot, surface) {
        ("en", "object", "letter") => Some("missive"),
        ("en", "object", "record") => Some("entry"),
        ("en", "object", "draft") => Some("outline"),
        ("en", "object", "review") => Some("assessment"),
        ("en", "adjective", "quiet") => Some("concise"),
        ("en", "adjective", "warm") => Some("recent"),
        ("en", "adjective", "fresh") => Some("current"),
        ("fa", "object", "نامه") => Some("مکتوب"),
        ("fa", "object", "پیام") => Some("پیغام"),
        ("fa", "object", "داستان") => Some("حکایت"),
        ("fa", "object", "غذا") => Some("طعام"),
        ("fa", "adjective", "زیبا") => Some("خوش"),
        ("fa", "adjective", "قدیمی") => Some("کهن"),
        ("fa", "adjective", "تازه") => Some("نو"),
        _ => None,
    }
}

fn should_use_secret_variant(seed: u64, frame_index: usize, slot: &str, surface: &str) -> bool {
    (surface_selector(seed, frame_index, slot, surface) & 1) == 1
}

fn surface_selector(seed: u64, frame_index: usize, slot: &str, surface: &str) -> u64 {
    let frame_mix = (frame_index as u64)
        .wrapping_add(1)
        .wrapping_mul(0x9E37_79B9_7F4A_7C15);
    let slot_mix = fnv1a64(slot.as_bytes()).rotate_left(17);
    let surface_mix = fnv1a64(surface.as_bytes()).rotate_left(33);
    seed ^ frame_mix ^ slot_mix ^ surface_mix
}

fn dataset_surface_variant(
    variant_catalog: Option<&LexiconVariantCatalog>,
    slot: &str,
    surface: &str,
    selector: u64,
) -> Option<String> {
    variant_catalog
        .and_then(|catalog| catalog.select_variant(slot, surface, selector))
        .map(ToString::to_string)
}

fn fnv1a64(bytes: &[u8]) -> u64 {
    let mut hash = 0xcbf2_9ce4_8422_2325u64;
    for &byte in bytes {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(0x0000_0001_0000_01b3);
    }
    hash
}

fn render_proto_decode_output(
    target: ProtoTarget,
    auto_detect_target: bool,
    input_mode: DecodeInputMode,
    trace_text: &str,
    format: OutputFormat,
    secret: Option<&[u8]>,
    data_dir: Option<&str>,
) -> Result<String, CliError> {
    if trace_text.trim().is_empty() {
        return Err(operation_requires_trace_or_text_input_error("decode"));
    }

    let target = resolve_trace_target(target, auto_detect_target, trace_text)?;
    let mut runtime = runtime_for_target(target.clone())?;
    let schemas = runtime.mapper.frame_schemas();
    let parsed_trace_frames = parse_frames_from_trace(trace_text, &schemas)
        .map_err(|error| CliError::trace(format!("failed to parse trace frames: {error}")))?;
    let (frames, used_extractor_frames) = match input_mode {
        DecodeInputMode::Trace => {
            if parsed_trace_frames.is_empty() {
                return Err(operation_trace_mode_requires_trace_input_error("decode"));
            }
            (parsed_trace_frames, false)
        }
        DecodeInputMode::Text => {
            let frames = resolve_text_frames_with_auto_fallback(
                &mut runtime,
                target,
                auto_detect_target,
                trace_text,
                "decode",
                operation_text_mode_requires_canonical_text_error,
                data_dir,
            )?;
            (frames, true)
        }
        DecodeInputMode::Auto => {
            if parsed_trace_frames.is_empty() {
                let frames = resolve_text_frames_with_auto_fallback(
                    &mut runtime,
                    target,
                    auto_detect_target,
                    trace_text,
                    "decode",
                    operation_auto_requires_trace_or_text_error,
                    data_dir,
                )?;
                (frames, true)
            } else {
                (parsed_trace_frames, false)
            }
        }
    };

    if frames.is_empty() {
        return Err(CliError::trace("no frame lines were found in trace input"));
    }
    if let Some(sequence_error) = frame_sequence_error(&frames) {
        return Err(CliError::trace(format!(
            "invalid trace frame sequence: {sequence_error}"
        )));
    }

    let active_schemas = runtime.mapper.frame_schemas();
    let ordered_schemas = frames
        .iter()
        .map(|frame| schema_for_template(&active_schemas, &frame.template_id))
        .collect::<Result<Vec<_>, String>>()
        .map_err(|error| CliError::trace(format!("failed to resolve trace schemas: {error}")))?;

    let raw_payload =
        match decode_raw_payload_from_frames(&runtime, trace_text, &frames, &ordered_schemas) {
            Ok(result) => result,
            Err(primary_error) => {
                if let Some(secret) = secret {
                    let mut unmixed_frames = frames.clone();
                    apply_secret_symbolic_mix(&mut unmixed_frames, secret);
                    match decode_raw_payload_from_frames(
                        &runtime,
                        trace_text,
                        &unmixed_frames,
                        &ordered_schemas,
                    ) {
                        Ok(result) => result,
                        Err(_) => return Err(primary_error),
                    }
                } else {
                    return Err(primary_error);
                }
            }
        };
    let payload = match secret {
        Some(secret) => {
            match decrypt_payload_with_secret(&raw_payload, secret, used_extractor_frames) {
                Ok(payload) => payload,
                Err(primary_error) => {
                    let mut unmixed_frames = frames.clone();
                    apply_secret_symbolic_mix(&mut unmixed_frames, secret);
                    let unmixed_payload = decode_raw_payload_from_frames(
                        &runtime,
                        trace_text,
                        &unmixed_frames,
                        &ordered_schemas,
                    )?;
                    match decrypt_payload_with_secret(
                        &unmixed_payload,
                        secret,
                        used_extractor_frames,
                    ) {
                        Ok(payload) => payload,
                        Err(_) => return Err(primary_error),
                    }
                }
            }
        }
        None => raw_payload,
    };
    let hex_payload = payload
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<Vec<_>>()
        .join("");

    let utf8_text = String::from_utf8(payload.clone()).ok();
    if matches!(format, OutputFormat::Json) {
        return Ok(build_proto_decode_json(
            runtime.language_code,
            payload.len(),
            &hex_payload,
            utf8_text.as_deref(),
            None,
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

    Ok(report_lines.join("\n"))
}

fn decode_raw_payload_from_frames(
    runtime: &PrototypeRuntime,
    trace_text: &str,
    frames: &[SymbolicFramePlan],
    ordered_schemas: &[SymbolicFrameSchema],
) -> Result<Vec<u8>, CliError> {
    let options = runtime
        .pipeline_options()
        .map_err(|error| CliError::config(format!("invalid pipeline options: {error}")))?;
    let orchestration = map_domain(
        runtime
            .orchestrator()
            .with_symbolic_options(FixedWidthPlanningOptions::default())
            .orchestrate_decode(
                DecodeRequest {
                    stego_text: trace_text.to_string(),
                    options,
                },
                frames,
                ordered_schemas,
            ),
        "decode orchestration failed",
    )?;
    Ok(orchestration.payload)
}

fn decrypt_payload_with_secret(
    raw_payload: &[u8],
    secret: &[u8],
    used_extractor_frames: bool,
) -> Result<Vec<u8>, CliError> {
    match inspect_envelope(raw_payload) {
        CryptoEnvelopeInspection::Metadata(_) => open_payload(raw_payload, secret).map_err(|_| {
            if used_extractor_frames {
                CliError::security(
                    "failed to decrypt payload from extracted text; use proto-encode trace input for lossless decode",
                )
            } else {
                CliError::security("failed to decrypt payload; verify provided secret")
            }
        }),
        CryptoEnvelopeInspection::NotEnvelope => Err(CliError::security(
            "failed to decrypt payload; payload is not a valid secure envelope",
        )),
        CryptoEnvelopeInspection::Invalid(message) => Err(CliError::security(format!(
            "failed to decrypt payload; invalid secure envelope metadata: {message}"
        ))),
    }
}

fn text_decode_not_lossless_error(language_display: &str, operation: &str) -> CliError {
    CliError::input(format!(
        "{language_display} text decode is not lossless yet; rerun encode with --emit-trace and use {operation} --trace-input"
    ))
}

fn operation_requires_trace_or_text_input_error(operation: &str) -> CliError {
    CliError::input(format!(
        "{operation} requires input from proto-encode trace output or canonical stego text"
    ))
}

fn operation_trace_mode_requires_trace_input_error(operation: &str) -> CliError {
    CliError::input(format!(
        "{operation} trace mode requires proto-encode trace input (rerun encode with --emit-trace)"
    ))
}

fn operation_text_mode_requires_canonical_text_error(operation: &str) -> CliError {
    CliError::input(format!(
        "{operation} text mode requires canonical stego text compatible with active language extractor"
    ))
}

fn operation_auto_requires_trace_or_text_error(operation: &str) -> CliError {
    CliError::input(format!(
        "{operation} requires parseable trace frames or canonical stego text"
    ))
}

fn resolve_text_frames_with_auto_fallback(
    runtime: &mut PrototypeRuntime,
    target: ProtoTarget,
    auto_detect_target: bool,
    trace_text: &str,
    operation: &str,
    missing_input_error: fn(&str) -> CliError,
    data_dir: Option<&str>,
) -> Result<Vec<SymbolicFramePlan>, CliError> {
    let variant_catalog = resolve_variant_catalog_for_target(&target, data_dir)?;
    if runtime.text_decode_lossless {
        if let Some(frames) = extract_text_frames(runtime, trace_text, variant_catalog.as_ref()) {
            return Ok(frames);
        }
    } else if !auto_detect_target {
        return Err(text_decode_not_lossless_error(
            runtime.language_display,
            operation,
        ));
    }

    if auto_detect_target {
        let fallback_target = alternate_target(target);
        let fallback_catalog = resolve_variant_catalog_for_target(&fallback_target, data_dir)?;
        let fallback_runtime = runtime_for_target(fallback_target)?;
        if fallback_runtime.text_decode_lossless {
            if let Some(frames) =
                extract_text_frames(&fallback_runtime, trace_text, fallback_catalog.as_ref())
            {
                *runtime = fallback_runtime;
                return Ok(frames);
            }
        }
    }

    if !runtime.text_decode_lossless {
        return Err(text_decode_not_lossless_error(
            runtime.language_display,
            operation,
        ));
    }

    Err(missing_input_error(operation))
}

fn extract_text_frames(
    runtime: &PrototypeRuntime,
    trace_text: &str,
    variant_catalog: Option<&LexiconVariantCatalog>,
) -> Option<Vec<SymbolicFramePlan>> {
    let normalized_trace = variant_catalog.map(|catalog| catalog.normalize_text(trace_text));
    let extraction_input = normalized_trace.as_deref().unwrap_or(trace_text);
    runtime
        .extract_plans(extraction_input)
        .ok()
        .filter(|plans| !plans.is_empty())
        .and_then(|plans| runtime.mapper.map_plans_to_frames(&plans).ok())
}

fn resolve_variant_catalog_for_target(
    target: &ProtoTarget,
    data_dir: Option<&str>,
) -> Result<Option<LexiconVariantCatalog>, CliError> {
    let active_source = resolve_active_data_source_selection(target.clone(), None, data_dir)?;
    resolve_active_data_source_variant_catalog(
        target.clone(),
        active_source.as_ref().map(|item| item.source_id.as_str()),
        data_dir,
    )
}

fn alternate_target(target: ProtoTarget) -> ProtoTarget {
    match target {
        ProtoTarget::Farsi => ProtoTarget::English,
        ProtoTarget::English => ProtoTarget::Farsi,
        ProtoTarget::Other(code) => ProtoTarget::Other(code),
    }
}

fn resolve_encode_profile_id(
    runtime: &PrototypeRuntime,
    profile: Option<&str>,
) -> Result<Option<StyleProfileId>, CliError> {
    let Some(raw_profile_id) = profile else {
        return Ok(None);
    };

    let profile_id = StyleProfileId::new(raw_profile_id).map_err(|_| {
        CliError::config(format!(
            "invalid style profile identifier '{raw_profile_id}'"
        ))
    })?;

    let descriptor = runtime.pack.style_profile(&profile_id).ok_or_else(|| {
        CliError::config(format!(
            "unsupported profile '{}' for language '{}'",
            profile_id, runtime.language_code
        ))
    })?;

    if descriptor.language.as_str() != runtime.language_code {
        return Err(CliError::config(format!(
            "profile '{}' is not available for language '{}'",
            profile_id, runtime.language_code
        )));
    }

    Ok(Some(profile_id))
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
            "{command} requires --secret <value> or --secret-file <file> (or LSTEG_SECRET/LSTEG_SECRET_FILE)"
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
    initialize_runtime(target)
}

fn map_domain<T, E>(result: Result<T, E>, context: &str) -> Result<T, CliError> {
    result.map_err(|_| CliError::domain(context.to_string()))
}
