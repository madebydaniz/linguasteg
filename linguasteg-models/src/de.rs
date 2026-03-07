use linguasteg_core::{
    BitRange, CoreError, CoreResult, GrammarConstraintChecker, LanguageDescriptor,
    LanguageRealizer, LanguageRegistry, LanguageTag, RealizationPlan,
    RealizationTemplateDescriptor, SlotAssignment, SlotId, SlotRole, StyleInspiration,
    StyleProfileDescriptor, StyleProfileId, StyleProfileRegistry, StyleStrength, SymbolicFieldSpec,
    SymbolicFramePlan, SymbolicFrameSchema, SymbolicPayloadPlan, SymbolicSlotValue, TemplateId,
    TemplateRegistry, TemplateSlotDescriptor, TemplateToken, TextDirection, WritingRegister,
    render_realization_plan, validate_realization_plan,
};

const DE_PROFILE_NEUTRAL: &str = "de-neutral-prototype";

#[derive(Debug, Clone)]
pub struct GermanPrototypeLanguagePack {
    languages: Vec<LanguageDescriptor>,
    style_profiles: Vec<StyleProfileDescriptor>,
    templates: Vec<RealizationTemplateDescriptor>,
}

impl Default for GermanPrototypeLanguagePack {
    fn default() -> Self {
        Self {
            languages: german_languages(),
            style_profiles: german_style_profiles(),
            templates: german_templates(),
        }
    }
}

impl LanguageRegistry for GermanPrototypeLanguagePack {
    fn all_languages(&self) -> &[LanguageDescriptor] {
        &self.languages
    }
}

impl StyleProfileRegistry for GermanPrototypeLanguagePack {
    fn all_style_profiles(&self) -> &[StyleProfileDescriptor] {
        &self.style_profiles
    }
}

impl TemplateRegistry for GermanPrototypeLanguagePack {
    fn all_templates(&self) -> &[RealizationTemplateDescriptor] {
        &self.templates
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct GermanPrototypeConstraintChecker;

impl GrammarConstraintChecker for GermanPrototypeConstraintChecker {
    fn validate_plan(
        &self,
        template: &RealizationTemplateDescriptor,
        plan: &RealizationPlan,
    ) -> CoreResult<()> {
        validate_realization_plan(template, plan)?;

        if template.language.as_str() != "de" {
            return Err(CoreError::UnsupportedLanguage(
                template.language.to_string(),
            ));
        }

        for assignment in &plan.assignments {
            if assignment.surface.trim().is_empty() {
                return Err(CoreError::InvalidTemplate(format!(
                    "slot '{}' cannot be empty in german realization plan",
                    assignment.slot
                )));
            }
        }

        Ok(())
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct GermanPrototypeRealizer;

impl LanguageRealizer for GermanPrototypeRealizer {
    fn render(
        &self,
        template: &RealizationTemplateDescriptor,
        plan: &RealizationPlan,
    ) -> CoreResult<String> {
        GermanPrototypeConstraintChecker.validate_plan(template, plan)?;
        let rendered = render_realization_plan(template, plan)?;
        Ok(normalize_german_spacing(&rendered))
    }
}

fn normalize_german_spacing(input: &str) -> String {
    let collapsed = input.split_whitespace().collect::<Vec<_>>().join(" ");
    collapsed.replace(" ,", ",")
}

#[derive(Debug, Default, Clone, Copy)]
pub struct GermanPrototypeSymbolicMapper;

impl GermanPrototypeSymbolicMapper {
    pub fn frame_schemas(&self) -> Vec<SymbolicFrameSchema> {
        german_symbolic_frame_schemas()
    }

    pub fn map_payload_to_plans(
        &self,
        payload_plan: &SymbolicPayloadPlan,
    ) -> CoreResult<Vec<RealizationPlan>> {
        self.map_payload_to_plans_with_profile(payload_plan, None)
    }

    pub fn map_payload_to_plans_with_profile(
        &self,
        payload_plan: &SymbolicPayloadPlan,
        profile_id: Option<&StyleProfileId>,
    ) -> CoreResult<Vec<RealizationPlan>> {
        validate_profile(profile_id)?;
        payload_plan
            .frames
            .iter()
            .map(|frame| self.map_frame_to_plan(frame))
            .collect()
    }

    pub fn map_frame_to_plan(&self, frame: &SymbolicFramePlan) -> CoreResult<RealizationPlan> {
        match frame.template_id.as_str() {
            "de-basic-svo" => map_basic_svo_frame(frame),
            "de-time-location-svo" => map_time_location_svo_frame(frame),
            _ => Err(CoreError::UnsupportedTemplate(
                frame.template_id.to_string(),
            )),
        }
    }

    pub fn map_plans_to_frames(
        &self,
        plans: &[RealizationPlan],
    ) -> CoreResult<Vec<SymbolicFramePlan>> {
        let schemas = self.frame_schemas();
        let mut frames = Vec::with_capacity(plans.len());
        let mut bit_cursor = 0usize;

        for plan in plans {
            let frame = map_plan_to_frame(plan, &schemas, bit_cursor)?;
            bit_cursor += frame.source.consumed_bits;
            frames.push(frame);
        }

        Ok(frames)
    }
}

pub fn parse_german_prototype_text(stego_text: &str) -> CoreResult<Vec<RealizationPlan>> {
    let text = select_german_text_body(stego_text);
    let mut plans = Vec::new();

    for raw_sentence in split_sentences(text) {
        let sentence = raw_sentence.trim();
        if sentence.is_empty() || sentence.starts_with("gateway response:") {
            continue;
        }
        plans.push(parse_german_sentence_to_plan(sentence)?);
    }

    if plans.is_empty() {
        return Err(CoreError::InvalidTemplate(
            "german text extractor could not detect canonical prototype sentences".to_string(),
        ));
    }

    Ok(plans)
}

fn validate_profile(profile_id: Option<&StyleProfileId>) -> CoreResult<()> {
    match profile_id.map(StyleProfileId::as_str) {
        None | Some(DE_PROFILE_NEUTRAL) => Ok(()),
        Some(value) => Err(CoreError::InvalidTemplate(format!(
            "unsupported german style profile '{value}'"
        ))),
    }
}

fn map_basic_svo_frame(frame: &SymbolicFramePlan) -> CoreResult<RealizationPlan> {
    let subject = select_form(subject_forms(), symbolic_value_for_slot(frame, "subject")?);
    let verb = select_form(verb_forms(), symbolic_value_for_slot(frame, "verb")?);
    let adjective = select_form(
        adjective_forms(),
        symbolic_value_for_slot(frame, "adjective")?,
    );
    let object = select_form(object_forms(), symbolic_value_for_slot(frame, "object")?);

    Ok(RealizationPlan {
        template_id: TemplateId::new("de-basic-svo")?,
        assignments: vec![
            create_assignment("subject", subject)?,
            create_assignment("verb", verb)?,
            create_assignment("adjective", adjective)?,
            create_assignment("object", object)?,
        ],
    })
}

fn map_time_location_svo_frame(frame: &SymbolicFramePlan) -> CoreResult<RealizationPlan> {
    let subject = select_form(subject_forms(), symbolic_value_for_slot(frame, "subject")?);
    let time = select_form(time_forms(), symbolic_value_for_slot(frame, "time")?);
    let location = select_form(
        location_forms(),
        symbolic_value_for_slot(frame, "location")?,
    );
    let verb = select_form(verb_forms(), symbolic_value_for_slot(frame, "verb")?);
    let object = select_form(object_forms(), symbolic_value_for_slot(frame, "object")?);

    Ok(RealizationPlan {
        template_id: TemplateId::new("de-time-location-svo")?,
        assignments: vec![
            create_assignment("subject", subject)?,
            create_assignment("time", time)?,
            create_assignment("location", location)?,
            create_assignment("verb", verb)?,
            create_assignment("object", object)?,
        ],
    })
}

fn symbolic_value_for_slot(frame: &SymbolicFramePlan, slot_name: &str) -> CoreResult<u32> {
    frame
        .values
        .iter()
        .find(|value| value.slot.as_str() == slot_name)
        .map(|value| value.value)
        .ok_or_else(|| {
            CoreError::InvalidSymbolicPlan(format!(
                "missing symbolic slot '{slot_name}' in template '{}'",
                frame.template_id
            ))
        })
}

fn select_form(forms: &[&str], value: u32) -> String {
    let index = (value as usize) % forms.len();
    forms[index].to_string()
}

fn create_assignment(slot: &str, surface: String) -> CoreResult<SlotAssignment> {
    Ok(SlotAssignment {
        slot: SlotId::new(slot)?,
        surface,
        lemma: None,
    })
}

fn map_plan_to_frame(
    plan: &RealizationPlan,
    schemas: &[SymbolicFrameSchema],
    start_bit: usize,
) -> CoreResult<SymbolicFramePlan> {
    let schema = schema_for_template(&plan.template_id, schemas)?;

    let values = schema
        .fields
        .iter()
        .map(|field| {
            let value = match plan.template_id.as_str() {
                "de-basic-svo" => symbolic_value_for_basic_plan_slot(plan, field.slot.as_str())?,
                "de-time-location-svo" => {
                    symbolic_value_for_time_location_plan_slot(plan, field.slot.as_str())?
                }
                _ => return Err(CoreError::UnsupportedTemplate(plan.template_id.to_string())),
            };

            Ok(SymbolicSlotValue {
                slot: field.slot.clone(),
                bit_width: field.bit_width,
                value,
            })
        })
        .collect::<CoreResult<Vec<_>>>()?;

    Ok(SymbolicFramePlan {
        template_id: schema.template_id.clone(),
        source: BitRange {
            start_bit,
            consumed_bits: schema.total_bits(),
        },
        values,
    })
}

fn symbolic_value_for_basic_plan_slot(plan: &RealizationPlan, slot: &str) -> CoreResult<u32> {
    match slot {
        "subject" => surface_index(
            subject_forms(),
            &assignment_by_slot(plan, "subject")?.surface,
        ),
        "object" => surface_index(object_forms(), &assignment_by_slot(plan, "object")?.surface),
        "adjective" => surface_index(
            adjective_forms(),
            &assignment_by_slot(plan, "adjective")?.surface,
        ),
        "verb" => surface_index(verb_forms(), &assignment_by_slot(plan, "verb")?.surface),
        _ => Err(CoreError::InvalidSymbolicPlan(format!(
            "unsupported slot '{slot}' for template '{}'",
            plan.template_id
        ))),
    }
}

fn symbolic_value_for_time_location_plan_slot(
    plan: &RealizationPlan,
    slot: &str,
) -> CoreResult<u32> {
    match slot {
        "subject" => surface_index(
            subject_forms(),
            &assignment_by_slot(plan, "subject")?.surface,
        ),
        "time" => surface_index(time_forms(), &assignment_by_slot(plan, "time")?.surface),
        "location" => surface_index(
            location_forms(),
            &assignment_by_slot(plan, "location")?.surface,
        ),
        "object" => surface_index(object_forms(), &assignment_by_slot(plan, "object")?.surface),
        "verb" => surface_index(verb_forms(), &assignment_by_slot(plan, "verb")?.surface),
        _ => Err(CoreError::InvalidSymbolicPlan(format!(
            "unsupported slot '{slot}' for template '{}'",
            plan.template_id
        ))),
    }
}

fn schema_for_template<'a>(
    template_id: &TemplateId,
    schemas: &'a [SymbolicFrameSchema],
) -> CoreResult<&'a SymbolicFrameSchema> {
    schemas
        .iter()
        .find(|schema| schema.template_id == *template_id)
        .ok_or_else(|| CoreError::UnsupportedTemplate(template_id.to_string()))
}

fn assignment_by_slot<'a>(plan: &'a RealizationPlan, slot: &str) -> CoreResult<&'a SlotAssignment> {
    plan.assignments
        .iter()
        .find(|assignment| assignment.slot.as_str() == slot)
        .ok_or_else(|| {
            CoreError::InvalidSymbolicPlan(format!(
                "missing slot '{slot}' in plan '{}'",
                plan.template_id
            ))
        })
}

fn surface_index(values: &[&str], surface: &str) -> CoreResult<u32> {
    let normalized = surface.trim();
    let index = values
        .iter()
        .position(|candidate| candidate.eq_ignore_ascii_case(normalized))
        .ok_or_else(|| {
            CoreError::InvalidSymbolicPlan(format!(
                "unknown surface value '{normalized}' in symbolic inventory"
            ))
        })?;
    u32::try_from(index).map_err(|_| {
        CoreError::InvalidSymbolicPlan(format!(
            "surface index {index} is too large for symbolic value conversion"
        ))
    })
}

fn select_german_text_body(input: &str) -> &str {
    let marker = "final prototype text:";
    if let Some(index) = input.find(marker) {
        let start = index + marker.len();
        return input[start..].trim();
    }

    input.trim()
}

fn split_sentences(input: &str) -> impl Iterator<Item = &str> {
    input.split(['.', '\n'])
}

fn parse_german_sentence_to_plan(sentence: &str) -> CoreResult<RealizationPlan> {
    if sentence.contains(',') {
        return parse_time_location_svo_sentence(sentence);
    }

    parse_basic_svo_sentence(sentence)
}

fn parse_basic_svo_sentence(sentence: &str) -> CoreResult<RealizationPlan> {
    let (subject, rest) = consume_form_prefix(sentence, subject_forms())
        .ok_or_else(|| unsupported_shape(sentence))?;
    let (verb, rest) =
        consume_form_prefix(rest, verb_forms()).ok_or_else(|| unsupported_shape(sentence))?;
    let rest = consume_fixed_prefix(rest, "das ").ok_or_else(|| unsupported_shape(sentence))?;
    let (adjective, rest) =
        consume_form_prefix(rest, adjective_forms()).ok_or_else(|| unsupported_shape(sentence))?;
    let (object, rest) =
        consume_form_prefix(rest, object_forms()).ok_or_else(|| unsupported_shape(sentence))?;
    if !rest.trim().is_empty() {
        return Err(unsupported_shape(sentence));
    }

    Ok(RealizationPlan {
        template_id: TemplateId::new("de-basic-svo")?,
        assignments: vec![
            create_assignment("subject", subject.to_string())?,
            create_assignment("verb", verb.to_string())?,
            create_assignment("adjective", adjective.to_string())?,
            create_assignment("object", object.to_string())?,
        ],
    })
}

fn parse_time_location_svo_sentence(sentence: &str) -> CoreResult<RealizationPlan> {
    let (left, right) = sentence
        .split_once(',')
        .ok_or_else(|| unsupported_shape(sentence))?;

    let (subject, rest) =
        consume_form_prefix(left, subject_forms()).ok_or_else(|| unsupported_shape(sentence))?;
    let (time, rest) =
        consume_form_prefix(rest, time_forms()).ok_or_else(|| unsupported_shape(sentence))?;
    let (location, rest) =
        consume_form_prefix(rest, location_forms()).ok_or_else(|| unsupported_shape(sentence))?;
    if !rest.trim().is_empty() {
        return Err(unsupported_shape(sentence));
    }

    let (verb, rest) =
        consume_form_prefix(right, verb_forms()).ok_or_else(|| unsupported_shape(sentence))?;
    let rest = consume_fixed_prefix(rest, "das ").ok_or_else(|| unsupported_shape(sentence))?;
    let (object, rest) =
        consume_form_prefix(rest, object_forms()).ok_or_else(|| unsupported_shape(sentence))?;
    if !rest.trim().is_empty() {
        return Err(unsupported_shape(sentence));
    }

    Ok(RealizationPlan {
        template_id: TemplateId::new("de-time-location-svo")?,
        assignments: vec![
            create_assignment("subject", subject.to_string())?,
            create_assignment("time", time.to_string())?,
            create_assignment("location", location.to_string())?,
            create_assignment("verb", verb.to_string())?,
            create_assignment("object", object.to_string())?,
        ],
    })
}

fn consume_form_prefix<'a>(
    input: &'a str,
    forms: &[&'static str],
) -> Option<(&'static str, &'a str)> {
    let trimmed = input.trim_start();
    let mut best_match: Option<(&'static str, &'a str)> = None;

    for &form in forms {
        let Some(rest) = trimmed.strip_prefix(form) else {
            continue;
        };
        if !rest.is_empty() && !rest.starts_with(' ') {
            continue;
        }
        let candidate = (form, rest.trim_start());
        let should_replace = match best_match {
            Some((best, _)) => form.len() > best.len(),
            None => true,
        };
        if should_replace {
            best_match = Some(candidate);
        }
    }

    best_match
}

fn consume_fixed_prefix<'a>(input: &'a str, prefix: &str) -> Option<&'a str> {
    input.trim_start().strip_prefix(prefix)
}

fn unsupported_shape(sentence: &str) -> CoreError {
    CoreError::InvalidTemplate(format!(
        "unsupported canonical german sentence shape: {sentence}"
    ))
}

fn german_languages() -> Vec<LanguageDescriptor> {
    vec![LanguageDescriptor {
        tag: LanguageTag::new("de").expect("valid language tag"),
        display_name: "German".to_string(),
        direction: TextDirection::LeftToRight,
    }]
}

fn german_style_profiles() -> Vec<StyleProfileDescriptor> {
    vec![StyleProfileDescriptor {
        id: StyleProfileId::new(DE_PROFILE_NEUTRAL).expect("valid style profile id"),
        language: LanguageTag::new("de").expect("valid language tag"),
        display_name: "Neutral German Prototype".to_string(),
        register: WritingRegister::Neutral,
        strength: StyleStrength::Light,
        inspiration: StyleInspiration::RegisterOnly,
    }]
}

fn german_templates() -> Vec<RealizationTemplateDescriptor> {
    vec![
        RealizationTemplateDescriptor {
            id: TemplateId::new("de-basic-svo").expect("valid template id"),
            language: LanguageTag::new("de").expect("valid language tag"),
            display_name: "German Basic SVO".to_string(),
            slots: vec![
                slot("subject", SlotRole::Subject, true),
                slot("verb", SlotRole::Verb, true),
                slot("adjective", SlotRole::Adjective, true),
                slot("object", SlotRole::DirectObject, true),
            ],
            tokens: vec![
                slot_token("subject"),
                literal_token(" "),
                slot_token("verb"),
                literal_token(" das "),
                slot_token("adjective"),
                literal_token(" "),
                slot_token("object"),
            ],
        },
        RealizationTemplateDescriptor {
            id: TemplateId::new("de-time-location-svo").expect("valid template id"),
            language: LanguageTag::new("de").expect("valid language tag"),
            display_name: "German Time Location SVO".to_string(),
            slots: vec![
                slot("subject", SlotRole::Subject, true),
                slot("time", SlotRole::Time, true),
                slot("location", SlotRole::Location, true),
                slot("verb", SlotRole::Verb, true),
                slot("object", SlotRole::DirectObject, true),
            ],
            tokens: vec![
                slot_token("subject"),
                literal_token(" "),
                slot_token("time"),
                literal_token(" "),
                slot_token("location"),
                literal_token(", "),
                slot_token("verb"),
                literal_token(" das "),
                slot_token("object"),
            ],
        },
    ]
}

fn german_symbolic_frame_schemas() -> Vec<SymbolicFrameSchema> {
    vec![
        SymbolicFrameSchema {
            template_id: TemplateId::new("de-basic-svo").expect("valid template id"),
            fields: vec![
                symbolic_field("subject", 5),
                symbolic_field("object", 5),
                symbolic_field("adjective", 3),
                symbolic_field("verb", 5),
            ],
        },
        SymbolicFrameSchema {
            template_id: TemplateId::new("de-time-location-svo").expect("valid template id"),
            fields: vec![
                symbolic_field("subject", 5),
                symbolic_field("time", 3),
                symbolic_field("location", 3),
                symbolic_field("object", 5),
                symbolic_field("verb", 5),
            ],
        },
    ]
}

fn slot(id: &str, role: SlotRole, required: bool) -> TemplateSlotDescriptor {
    TemplateSlotDescriptor {
        id: SlotId::new(id).expect("valid slot id"),
        role,
        required,
    }
}

fn slot_token(id: &str) -> TemplateToken {
    TemplateToken::Slot(SlotId::new(id).expect("valid slot id"))
}

fn literal_token(value: &str) -> TemplateToken {
    TemplateToken::Literal(value.to_string())
}

fn symbolic_field(slot: &str, bit_width: u8) -> SymbolicFieldSpec {
    SymbolicFieldSpec {
        slot: SlotId::new(slot).expect("valid slot id"),
        bit_width,
    }
}

fn subject_forms() -> &'static [&'static str] {
    &[
        "der autor",
        "der student",
        "der kuenstler",
        "der lehrer",
        "der forscher",
        "der ingenieur",
        "der gast",
        "der manager",
        "der analyst",
        "der redakteur",
        "der designer",
        "der planer",
        "der operator",
        "der kurator",
        "der mentor",
        "der pruefer",
        "der architekt",
        "der bibliothekar",
        "der arzt",
        "der jurist",
        "der koch",
        "der pilot",
        "der pfleger",
        "der haendler",
        "der bauer",
        "der fahrer",
        "der sachbearbeiter",
        "der trainer",
        "der direktor",
        "der inspektor",
        "der produzent",
        "der wissenschaftler",
    ]
}

fn object_forms() -> &'static [&'static str] {
    &[
        "buch",
        "brief",
        "foto",
        "journal",
        "blume",
        "dossier",
        "notiz",
        "bericht",
        "artikel",
        "memo",
        "vertrag",
        "ticket",
        "bild",
        "eintrag",
        "rechnung",
        "plan",
        "diagramm",
        "handbuch",
        "paket",
        "probe",
        "geraet",
        "ordner",
        "archiv",
        "datensatz",
        "zusammenfassung",
        "skript",
        "entwurf",
        "rezension",
        "angebot",
        "zeitplan",
        "katalog",
        "protokoll",
    ]
}

fn adjective_forms() -> &'static [&'static str] {
    &[
        "alte", "neue", "klare", "helle", "warme", "frische", "kleine", "stille",
    ]
}

fn verb_forms() -> &'static [&'static str] {
    &[
        "kauft",
        "schreibt",
        "liest",
        "sieht",
        "haelt",
        "baut",
        "findet",
        "bewegt",
        "sendet",
        "bringt",
        "entwirft",
        "prueft",
        "studiert",
        "reinigt",
        "malt",
        "ordnet",
        "zeichnet",
        "bereitet",
        "teilt",
        "packt",
        "repariert",
        "speichert",
        "aktualisiert",
        "meldet",
        "markiert",
        "archiviert",
        "bearbeitet",
        "druckt",
        "zaehlt",
        "sortiert",
        "kopiert",
        "plant",
    ]
}

fn time_forms() -> &'static [&'static str] {
    &[
        "heute",
        "gestern",
        "morgen",
        "mittags",
        "im fruehling",
        "abends",
        "im winter",
        "frueh",
    ]
}

fn location_forms() -> &'static [&'static str] {
    &[
        "zu hause",
        "in der bibliothek",
        "im markt",
        "in der schule",
        "im park",
        "im buero",
        "in der kueche",
        "auf der strasse",
    ]
}

#[cfg(test)]
mod tests {
    use super::{
        GermanPrototypeLanguagePack, GermanPrototypeRealizer, GermanPrototypeSymbolicMapper,
        parse_german_prototype_text,
    };
    use linguasteg_core::{
        BitRange, LanguageRealizer, LanguageRegistry, SlotId, SymbolicFramePlan, SymbolicSlotValue,
        TemplateId, TemplateRegistry,
    };

    #[test]
    fn german_pack_exposes_language_templates_and_profile() {
        let pack = GermanPrototypeLanguagePack::default();
        assert_eq!(pack.all_languages().len(), 1);
        assert_eq!(pack.all_languages()[0].tag.as_str(), "de");
        assert_eq!(pack.all_templates().len(), 2);
    }

    #[test]
    fn german_realizer_renders_sentence() {
        let pack = GermanPrototypeLanguagePack::default();
        let template = pack
            .template(&TemplateId::new("de-basic-svo").expect("valid id"))
            .expect("template should exist");
        let plan = super::map_basic_svo_frame(&SymbolicFramePlan {
            template_id: TemplateId::new("de-basic-svo").expect("valid template id"),
            source: BitRange {
                start_bit: 0,
                consumed_bits: 18,
            },
            values: vec![
                SymbolicSlotValue {
                    slot: SlotId::new("subject").expect("valid slot"),
                    bit_width: 5,
                    value: 0,
                },
                SymbolicSlotValue {
                    slot: SlotId::new("object").expect("valid slot"),
                    bit_width: 5,
                    value: 1,
                },
                SymbolicSlotValue {
                    slot: SlotId::new("adjective").expect("valid slot"),
                    bit_width: 3,
                    value: 2,
                },
                SymbolicSlotValue {
                    slot: SlotId::new("verb").expect("valid slot"),
                    bit_width: 5,
                    value: 3,
                },
            ],
        })
        .expect("mapping should succeed");

        let rendered = GermanPrototypeRealizer
            .render(template, &plan)
            .expect("render should succeed");
        assert_eq!(rendered, "der autor sieht das klare brief");
    }

    #[test]
    fn german_mapper_maps_plans_back_to_frames() {
        let mapper = GermanPrototypeSymbolicMapper;
        let frame = SymbolicFramePlan {
            template_id: TemplateId::new("de-time-location-svo").expect("valid template id"),
            source: BitRange {
                start_bit: 0,
                consumed_bits: 21,
            },
            values: vec![
                SymbolicSlotValue {
                    slot: SlotId::new("subject").expect("valid slot"),
                    bit_width: 5,
                    value: 2,
                },
                SymbolicSlotValue {
                    slot: SlotId::new("time").expect("valid slot"),
                    bit_width: 3,
                    value: 4,
                },
                SymbolicSlotValue {
                    slot: SlotId::new("location").expect("valid slot"),
                    bit_width: 3,
                    value: 1,
                },
                SymbolicSlotValue {
                    slot: SlotId::new("object").expect("valid slot"),
                    bit_width: 5,
                    value: 7,
                },
                SymbolicSlotValue {
                    slot: SlotId::new("verb").expect("valid slot"),
                    bit_width: 5,
                    value: 10,
                },
            ],
        };
        let plan = mapper
            .map_frame_to_plan(&frame)
            .expect("frame mapping should succeed");
        let frames = mapper
            .map_plans_to_frames(&[plan])
            .expect("reverse mapping should succeed");
        assert_eq!(frames.len(), 1);
        assert_eq!(frames[0].template_id.as_str(), "de-time-location-svo");
    }

    #[test]
    fn german_text_parser_parses_canonical_sentences() {
        let text = "der autor schreibt das klare brief. der student heute in der bibliothek, liest das foto.";
        let plans = parse_german_prototype_text(text).expect("text should parse");
        assert_eq!(plans.len(), 2);
        assert_eq!(plans[0].template_id.as_str(), "de-basic-svo");
        assert_eq!(plans[1].template_id.as_str(), "de-time-location-svo");
    }
}
