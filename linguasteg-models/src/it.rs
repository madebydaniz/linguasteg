use linguasteg_core::{
    BitRange, CoreError, CoreResult, GrammarConstraintChecker, LanguageDescriptor,
    LanguageRealizer, LanguageRegistry, LanguageTag, RealizationPlan,
    RealizationTemplateDescriptor, SlotAssignment, SlotId, SlotRole, StyleInspiration,
    StyleProfileDescriptor, StyleProfileId, StyleProfileRegistry, StyleStrength, SymbolicFieldSpec,
    SymbolicFramePlan, SymbolicFrameSchema, SymbolicPayloadPlan, SymbolicSlotValue, TemplateId,
    TemplateRegistry, TemplateSlotDescriptor, TemplateToken, TextDirection, WritingRegister,
    render_realization_plan, validate_realization_plan,
};

#[derive(Debug, Clone)]
pub struct ItalianPrototypeLanguagePack {
    languages: Vec<LanguageDescriptor>,
    style_profiles: Vec<StyleProfileDescriptor>,
    templates: Vec<RealizationTemplateDescriptor>,
}

impl Default for ItalianPrototypeLanguagePack {
    fn default() -> Self {
        Self {
            languages: italian_languages(),
            style_profiles: italian_style_profiles(),
            templates: italian_templates(),
        }
    }
}

impl LanguageRegistry for ItalianPrototypeLanguagePack {
    fn all_languages(&self) -> &[LanguageDescriptor] {
        &self.languages
    }
}

impl StyleProfileRegistry for ItalianPrototypeLanguagePack {
    fn all_style_profiles(&self) -> &[StyleProfileDescriptor] {
        &self.style_profiles
    }
}

impl TemplateRegistry for ItalianPrototypeLanguagePack {
    fn all_templates(&self) -> &[RealizationTemplateDescriptor] {
        &self.templates
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct ItalianPrototypeConstraintChecker;

impl GrammarConstraintChecker for ItalianPrototypeConstraintChecker {
    fn validate_plan(
        &self,
        template: &RealizationTemplateDescriptor,
        plan: &RealizationPlan,
    ) -> CoreResult<()> {
        validate_realization_plan(template, plan)?;

        if template.language.as_str() != "it" {
            return Err(CoreError::UnsupportedLanguage(
                template.language.to_string(),
            ));
        }

        for assignment in &plan.assignments {
            if assignment.surface.trim().is_empty() {
                return Err(CoreError::InvalidTemplate(format!(
                    "slot '{}' cannot be empty in italian realization plan",
                    assignment.slot
                )));
            }
        }

        Ok(())
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct ItalianPrototypeRealizer;

impl LanguageRealizer for ItalianPrototypeRealizer {
    fn render(
        &self,
        template: &RealizationTemplateDescriptor,
        plan: &RealizationPlan,
    ) -> CoreResult<String> {
        ItalianPrototypeConstraintChecker.validate_plan(template, plan)?;
        let rendered = render_realization_plan(template, plan)?;
        Ok(normalize_italian_spacing(&rendered))
    }
}

fn normalize_italian_spacing(input: &str) -> String {
    let collapsed = input.split_whitespace().collect::<Vec<_>>().join(" ");
    collapsed.replace(" ,", ",")
}

#[derive(Debug, Default, Clone, Copy)]
pub struct ItalianPrototypeSymbolicMapper;

const IT_PROFILE_NEUTRAL: &str = "it-neutral-prototype";
const IT_PROFILE_SHAKESPEARE_LIGHT: &str = "it-shakespeare-inspired-light";
const IT_PROFILE_DICKENS_LIGHT: &str = "it-dickens-inspired-light";
const IT_PROFILE_AUSTEN_LIGHT: &str = "it-austen-inspired-light";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ItalianEncodeProfile {
    NeutralPrototype,
    ShakespeareInspiredLight,
    DickensInspiredLight,
    AustenInspiredLight,
}

impl ItalianEncodeProfile {
    fn from_profile_id(profile_id: Option<&StyleProfileId>) -> CoreResult<Self> {
        match profile_id.map(StyleProfileId::as_str) {
            None | Some(IT_PROFILE_NEUTRAL) => Ok(Self::NeutralPrototype),
            Some(IT_PROFILE_SHAKESPEARE_LIGHT) => Ok(Self::ShakespeareInspiredLight),
            Some(IT_PROFILE_DICKENS_LIGHT) => Ok(Self::DickensInspiredLight),
            Some(IT_PROFILE_AUSTEN_LIGHT) => Ok(Self::AustenInspiredLight),
            Some(value) => Err(CoreError::InvalidTemplate(format!(
                "unsupported italian style profile '{value}'"
            ))),
        }
    }
}

impl ItalianPrototypeSymbolicMapper {
    pub fn frame_schemas(&self) -> Vec<SymbolicFrameSchema> {
        italian_symbolic_frame_schemas()
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
        let profile = ItalianEncodeProfile::from_profile_id(profile_id)?;
        payload_plan
            .frames
            .iter()
            .map(|frame| self.map_frame_to_plan_with_profile(frame, profile))
            .collect()
    }

    pub fn map_frame_to_plan(&self, frame: &SymbolicFramePlan) -> CoreResult<RealizationPlan> {
        self.map_frame_to_plan_with_profile(frame, ItalianEncodeProfile::NeutralPrototype)
    }

    fn map_frame_to_plan_with_profile(
        &self,
        frame: &SymbolicFramePlan,
        profile: ItalianEncodeProfile,
    ) -> CoreResult<RealizationPlan> {
        match frame.template_id.as_str() {
            "it-basic-svo" => map_basic_svo_frame(frame, profile),
            "it-time-location-svo" => map_time_location_svo_frame(frame, profile),
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

pub fn parse_italian_prototype_text(stego_text: &str) -> CoreResult<Vec<RealizationPlan>> {
    let text = select_italian_text_body(stego_text);
    let mut plans = Vec::new();

    for raw_sentence in split_sentences(text) {
        let sentence = raw_sentence.trim();
        if sentence.is_empty() {
            continue;
        }
        if sentence.starts_with("gateway response:") {
            continue;
        }
        plans.push(parse_italian_sentence_to_plan(sentence)?);
    }

    if plans.is_empty() {
        return Err(CoreError::InvalidTemplate(
            "italian text extractor could not detect canonical prototype sentences".to_string(),
        ));
    }

    Ok(plans)
}

fn map_basic_svo_frame(
    frame: &SymbolicFramePlan,
    profile: ItalianEncodeProfile,
) -> CoreResult<RealizationPlan> {
    let object_value = symbolic_value_for_slot(frame, "object")?;
    let adjective_value = symbolic_value_for_slot(frame, "adjective")?;
    let verb_value = symbolic_value_for_slot(frame, "verb")?;
    let verb = select_verb_surface(profile, verb_value, object_value);
    let subject = select_form(subject_forms(), symbolic_value_for_slot(frame, "subject")?);
    let object = select_object_surface(profile, object_value, verb_value);
    let object_class = object_class(object_lexeme_for_value(object_value).canonical);
    let adjective = select_adjective_surface(profile, adjective_value, object_class, object_value);

    Ok(RealizationPlan {
        template_id: TemplateId::new("it-basic-svo")?,
        assignments: vec![
            create_assignment("subject", subject)?,
            create_assignment("verb", verb)?,
            create_assignment("adjective", adjective)?,
            create_assignment("object", object)?,
        ],
    })
}

fn map_time_location_svo_frame(
    frame: &SymbolicFramePlan,
    profile: ItalianEncodeProfile,
) -> CoreResult<RealizationPlan> {
    let object_value = symbolic_value_for_slot(frame, "object")?;
    let verb_value = symbolic_value_for_slot(frame, "verb")?;
    let verb = select_verb_surface(profile, verb_value, object_value);
    let subject = select_form(subject_forms(), symbolic_value_for_slot(frame, "subject")?);
    let time = select_form(time_forms(), symbolic_value_for_slot(frame, "time")?);
    let location = select_form(
        location_forms(),
        symbolic_value_for_slot(frame, "location")?,
    );
    let object = select_object_surface(profile, object_value, verb_value);

    Ok(RealizationPlan {
        template_id: TemplateId::new("it-time-location-svo")?,
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

fn create_assignment(slot: &str, surface: String) -> CoreResult<linguasteg_core::SlotAssignment> {
    Ok(linguasteg_core::SlotAssignment {
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
                "it-basic-svo" => symbolic_value_for_basic_plan_slot(plan, field.slot.as_str())?,
                "it-time-location-svo" => {
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
        "subject" => {
            let assignment = assignment_by_slot(plan, "subject")?;
            surface_index(subject_forms(), &assignment.surface)
        }
        "object" => {
            let assignment = assignment_by_slot(plan, "object")?;
            object_surface_index(&assignment.surface)
        }
        "adjective" => {
            let assignment = assignment_by_slot(plan, "adjective")?;
            adjective_surface_index(&assignment.surface)
        }
        "verb" => {
            let assignment = assignment_by_slot(plan, "verb")?;
            verb_surface_index(&assignment.surface)
        }
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
        "subject" => {
            let assignment = assignment_by_slot(plan, "subject")?;
            surface_index(subject_forms(), &assignment.surface)
        }
        "time" => {
            let assignment = assignment_by_slot(plan, "time")?;
            surface_index(time_forms(), &assignment.surface)
        }
        "location" => {
            let assignment = assignment_by_slot(plan, "location")?;
            surface_index(location_forms(), &assignment.surface)
        }
        "object" => {
            let assignment = assignment_by_slot(plan, "object")?;
            object_surface_index(&assignment.surface)
        }
        "verb" => {
            let assignment = assignment_by_slot(plan, "verb")?;
            verb_surface_index(&assignment.surface)
        }
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
    let idx = values
        .iter()
        .position(|candidate| candidate.eq_ignore_ascii_case(normalized))
        .ok_or_else(|| {
            CoreError::InvalidSymbolicPlan(format!(
                "unknown surface value '{normalized}' in symbolic inventory"
            ))
        })?;
    u32::try_from(idx).map_err(|_| {
        CoreError::InvalidSymbolicPlan(format!(
            "surface index {idx} is too large for symbolic value conversion"
        ))
    })
}

fn select_italian_text_body(input: &str) -> &str {
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

fn parse_italian_sentence_to_plan(sentence: &str) -> CoreResult<RealizationPlan> {
    if sentence.contains(',') {
        return parse_time_location_svo_sentence(sentence);
    }

    parse_basic_svo_sentence(sentence)
}

fn parse_basic_svo_sentence(sentence: &str) -> CoreResult<RealizationPlan> {
    let (subject, rest) = consume_form_prefix(sentence, subject_forms())
        .ok_or_else(|| unsupported_shape(sentence))?;
    let (verb, rest) = consume_verb_prefix(rest).ok_or_else(|| unsupported_shape(sentence))?;
    let (adjective, rest) =
        consume_adjective_prefix(rest).ok_or_else(|| unsupported_shape(sentence))?;
    let (object, rest) = consume_object_prefix(rest).ok_or_else(|| unsupported_shape(sentence))?;

    if !rest.trim().is_empty() {
        return Err(unsupported_shape(sentence));
    }

    Ok(RealizationPlan {
        template_id: TemplateId::new("it-basic-svo")?,
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

    let (verb, rest) = consume_verb_prefix(right).ok_or_else(|| unsupported_shape(sentence))?;
    let (object, rest) = consume_object_prefix(rest).ok_or_else(|| unsupported_shape(sentence))?;

    if !rest.trim().is_empty() {
        return Err(unsupported_shape(sentence));
    }

    Ok(RealizationPlan {
        template_id: TemplateId::new("it-time-location-svo")?,
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

fn consume_object_prefix<'a>(input: &'a str) -> Option<(&'static str, &'a str)> {
    let trimmed = input.trim_start();
    let mut best_match: Option<(&'static str, &'a str, usize)> = None;

    for lexeme in object_lexemes() {
        for &form in lexeme.accepted_forms {
            let Some(rest) = trimmed.strip_prefix(form) else {
                continue;
            };

            if !rest.is_empty() && !rest.starts_with(' ') {
                continue;
            }

            let candidate = (lexeme.canonical, rest.trim_start(), form.len());
            let should_replace = match best_match {
                Some((_, _, best_len)) => form.len() > best_len,
                None => true,
            };
            if should_replace {
                best_match = Some(candidate);
            }
        }
    }

    best_match.map(|(canonical, rest, _)| (canonical, rest))
}

fn consume_adjective_prefix<'a>(input: &'a str) -> Option<(&'static str, &'a str)> {
    let trimmed = input.trim_start();
    let mut best_match: Option<(&'static str, &'a str, usize)> = None;

    for lexeme in adjective_lexemes() {
        for &form in lexeme.accepted_forms {
            let Some(rest) = trimmed.strip_prefix(form) else {
                continue;
            };

            if !rest.is_empty() && !rest.starts_with(' ') {
                continue;
            }

            let candidate = (lexeme.canonical, rest.trim_start(), form.len());
            let should_replace = match best_match {
                Some((_, _, best_len)) => form.len() > best_len,
                None => true,
            };
            if should_replace {
                best_match = Some(candidate);
            }
        }
    }

    best_match.map(|(canonical, rest, _)| (canonical, rest))
}

fn consume_verb_prefix<'a>(input: &'a str) -> Option<(&'static str, &'a str)> {
    let trimmed = input.trim_start();
    let mut best_match: Option<(&'static str, &'a str, usize)> = None;

    for lexeme in verb_lexemes() {
        for &form in lexeme.accepted_forms {
            let Some(rest) = trimmed.strip_prefix(form) else {
                continue;
            };

            if !rest.is_empty() && !rest.starts_with(' ') {
                continue;
            }

            let candidate = (lexeme.canonical, rest.trim_start(), form.len());
            let should_replace = match best_match {
                Some((_, _, best_len)) => form.len() > best_len,
                None => true,
            };
            if should_replace {
                best_match = Some(candidate);
            }
        }
    }

    best_match.map(|(canonical, rest, _)| (canonical, rest))
}

fn unsupported_shape(sentence: &str) -> CoreError {
    CoreError::InvalidTemplate(format!(
        "unsupported canonical italian sentence shape: {sentence}"
    ))
}

fn italian_languages() -> Vec<LanguageDescriptor> {
    vec![LanguageDescriptor {
        tag: LanguageTag::new("it").expect("valid language tag"),
        display_name: "Italian".to_string(),
        direction: TextDirection::LeftToRight,
    }]
}

fn italian_style_profiles() -> Vec<StyleProfileDescriptor> {
    vec![
        StyleProfileDescriptor {
            id: StyleProfileId::new(IT_PROFILE_NEUTRAL).expect("valid style profile id"),
            language: LanguageTag::new("it").expect("valid language tag"),
            display_name: "Neutral Italian Prototype".to_string(),
            register: WritingRegister::Neutral,
            strength: StyleStrength::Light,
            inspiration: StyleInspiration::RegisterOnly,
        },
        StyleProfileDescriptor {
            id: StyleProfileId::new(IT_PROFILE_SHAKESPEARE_LIGHT).expect("valid style profile id"),
            language: LanguageTag::new("it").expect("valid language tag"),
            display_name: "Shakespeare-inspired (Light)".to_string(),
            register: WritingRegister::Literary,
            strength: StyleStrength::Light,
            inspiration: StyleInspiration::PublicDomainAuthorInspired {
                author_label: "William Shakespeare".to_string(),
            },
        },
        StyleProfileDescriptor {
            id: StyleProfileId::new(IT_PROFILE_DICKENS_LIGHT).expect("valid style profile id"),
            language: LanguageTag::new("it").expect("valid language tag"),
            display_name: "Dickens-inspired (Light)".to_string(),
            register: WritingRegister::Literary,
            strength: StyleStrength::Light,
            inspiration: StyleInspiration::PublicDomainAuthorInspired {
                author_label: "Charles Dickens".to_string(),
            },
        },
        StyleProfileDescriptor {
            id: StyleProfileId::new(IT_PROFILE_AUSTEN_LIGHT).expect("valid style profile id"),
            language: LanguageTag::new("it").expect("valid language tag"),
            display_name: "Austen-inspired (Light)".to_string(),
            register: WritingRegister::Literary,
            strength: StyleStrength::Light,
            inspiration: StyleInspiration::PublicDomainAuthorInspired {
                author_label: "Jane Austen".to_string(),
            },
        },
    ]
}

fn italian_templates() -> Vec<RealizationTemplateDescriptor> {
    vec![
        RealizationTemplateDescriptor {
            id: TemplateId::new("it-basic-svo").expect("valid template id"),
            language: LanguageTag::new("it").expect("valid language tag"),
            display_name: "Italian Basic SVO".to_string(),
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
                literal_token(" "),
                slot_token("adjective"),
                literal_token(" "),
                slot_token("object"),
            ],
        },
        RealizationTemplateDescriptor {
            id: TemplateId::new("it-time-location-svo").expect("valid template id"),
            language: LanguageTag::new("it").expect("valid language tag"),
            display_name: "Italian Time Location SVO".to_string(),
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
                literal_token(" "),
                slot_token("object"),
            ],
        },
    ]
}

fn italian_symbolic_frame_schemas() -> Vec<SymbolicFrameSchema> {
    vec![
        SymbolicFrameSchema {
            template_id: TemplateId::new("it-basic-svo").expect("valid template id"),
            fields: vec![
                symbolic_field("subject", 5),
                symbolic_field("object", 5),
                symbolic_field("adjective", 3),
                symbolic_field("verb", 5),
            ],
        },
        SymbolicFrameSchema {
            template_id: TemplateId::new("it-time-location-svo").expect("valid template id"),
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
        "the writer",
        "the student",
        "the artist",
        "the teacher",
        "the researcher",
        "the engineer",
        "the visitor",
        "the manager",
        "the analyst",
        "the editor",
        "the designer",
        "the planner",
        "the operator",
        "the curator",
        "the mentor",
        "the reviewer",
        "the architect",
        "the librarian",
        "the doctor",
        "the lawyer",
        "the chef",
        "the pilot",
        "the nurse",
        "the trader",
        "the farmer",
        "the driver",
        "the clerk",
        "the coach",
        "the director",
        "the inspector",
        "the producer",
        "the scientist",
    ]
}

#[cfg(test)]
fn object_forms() -> &'static [&'static str] {
    &[
        "book", "letter", "photo", "journal", "briefing", "dossier", "note", "report", "article",
        "memo", "contract", "ticket", "canvas", "record", "invoice", "plan", "diagram", "manual",
        "parcel", "sample", "device", "folder", "archive", "dataset", "summary", "script", "draft",
        "review", "proposal", "schedule", "catalog", "brief",
    ]
}

#[derive(Debug, Clone, Copy)]
struct ItalianObjectLexeme {
    canonical: &'static str,
    accepted_forms: &'static [&'static str],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ItalianObjectClass {
    Document,
    Artifact,
    Package,
    Device,
    Data,
}

fn object_lexemes() -> &'static [ItalianObjectLexeme] {
    const OBJECT_LEXEMES: [ItalianObjectLexeme; 32] = [
        ItalianObjectLexeme {
            canonical: "book",
            accepted_forms: &["book", "volume"],
        },
        ItalianObjectLexeme {
            canonical: "letter",
            accepted_forms: &["letter", "missive", "epistle"],
        },
        ItalianObjectLexeme {
            canonical: "photo",
            accepted_forms: &["photo"],
        },
        ItalianObjectLexeme {
            canonical: "journal",
            accepted_forms: &["journal", "tea"],
        },
        ItalianObjectLexeme {
            canonical: "briefing",
            accepted_forms: &["briefing", "food"],
        },
        ItalianObjectLexeme {
            canonical: "dossier",
            accepted_forms: &["dossier", "flower"],
        },
        ItalianObjectLexeme {
            canonical: "note",
            accepted_forms: &["note"],
        },
        ItalianObjectLexeme {
            canonical: "report",
            accepted_forms: &["report"],
        },
        ItalianObjectLexeme {
            canonical: "article",
            accepted_forms: &["article"],
        },
        ItalianObjectLexeme {
            canonical: "memo",
            accepted_forms: &["memo"],
        },
        ItalianObjectLexeme {
            canonical: "contract",
            accepted_forms: &["contract", "agreement"],
        },
        ItalianObjectLexeme {
            canonical: "ticket",
            accepted_forms: &["ticket"],
        },
        ItalianObjectLexeme {
            canonical: "canvas",
            accepted_forms: &["canvas"],
        },
        ItalianObjectLexeme {
            canonical: "record",
            accepted_forms: &["record", "entry", "chronicle"],
        },
        ItalianObjectLexeme {
            canonical: "invoice",
            accepted_forms: &["invoice"],
        },
        ItalianObjectLexeme {
            canonical: "plan",
            accepted_forms: &["plan", "scheme"],
        },
        ItalianObjectLexeme {
            canonical: "diagram",
            accepted_forms: &["diagram"],
        },
        ItalianObjectLexeme {
            canonical: "manual",
            accepted_forms: &["manual"],
        },
        ItalianObjectLexeme {
            canonical: "parcel",
            accepted_forms: &["parcel"],
        },
        ItalianObjectLexeme {
            canonical: "sample",
            accepted_forms: &["sample"],
        },
        ItalianObjectLexeme {
            canonical: "device",
            accepted_forms: &["device"],
        },
        ItalianObjectLexeme {
            canonical: "folder",
            accepted_forms: &["folder"],
        },
        ItalianObjectLexeme {
            canonical: "archive",
            accepted_forms: &["archive"],
        },
        ItalianObjectLexeme {
            canonical: "dataset",
            accepted_forms: &["dataset"],
        },
        ItalianObjectLexeme {
            canonical: "summary",
            accepted_forms: &["summary", "abstract"],
        },
        ItalianObjectLexeme {
            canonical: "script",
            accepted_forms: &["script"],
        },
        ItalianObjectLexeme {
            canonical: "draft",
            accepted_forms: &["draft", "outline"],
        },
        ItalianObjectLexeme {
            canonical: "review",
            accepted_forms: &["review", "assessment", "critique"],
        },
        ItalianObjectLexeme {
            canonical: "proposal",
            accepted_forms: &["proposal"],
        },
        ItalianObjectLexeme {
            canonical: "schedule",
            accepted_forms: &["schedule"],
        },
        ItalianObjectLexeme {
            canonical: "catalog",
            accepted_forms: &["catalog"],
        },
        ItalianObjectLexeme {
            canonical: "brief",
            accepted_forms: &["brief"],
        },
    ];

    &OBJECT_LEXEMES
}

fn object_lexeme_for_value(value: u32) -> &'static ItalianObjectLexeme {
    let index = (value as usize) % object_lexemes().len();
    &object_lexemes()[index]
}

fn select_object_surface(profile: ItalianEncodeProfile, value: u32, verb_value: u32) -> String {
    let lexeme = object_lexeme_for_value(value);
    let verb_index = (verb_value as usize) % verb_lexemes().len();
    let base_surface = match (lexeme.canonical, verb_index) {
        ("record", 16) => "entry",
        ("draft", 10) => "outline",
        ("review", 11) => "assessment",
        _ => lexeme.canonical,
    };

    let profile_surface = author_object_variant(profile, lexeme.canonical, value, verb_index);
    profile_surface.unwrap_or(base_surface).to_string()
}

fn author_object_variant(
    profile: ItalianEncodeProfile,
    canonical: &str,
    value: u32,
    verb_index: usize,
) -> Option<&'static str> {
    if !is_light_profile_variant(value, verb_index, 2) {
        return None;
    }

    match profile {
        ItalianEncodeProfile::ShakespeareInspiredLight => match canonical {
            "letter" => Some("epistle"),
            "record" => Some("chronicle"),
            "book" => Some("volume"),
            _ => None,
        },
        ItalianEncodeProfile::DickensInspiredLight => match canonical {
            "contract" => Some("agreement"),
            "summary" => Some("abstract"),
            "review" => Some("critique"),
            _ => None,
        },
        ItalianEncodeProfile::AustenInspiredLight => match canonical {
            "plan" => Some("scheme"),
            "letter" => Some("epistle"),
            _ => None,
        },
        ItalianEncodeProfile::NeutralPrototype => None,
    }
}

fn object_class(canonical: &str) -> ItalianObjectClass {
    match canonical {
        "photo" | "canvas" | "sample" => ItalianObjectClass::Artifact,
        "parcel" => ItalianObjectClass::Package,
        "device" => ItalianObjectClass::Device,
        "record" | "archive" | "dataset" => ItalianObjectClass::Data,
        _ => ItalianObjectClass::Document,
    }
}

fn object_surface_index(surface: &str) -> CoreResult<u32> {
    let normalized = surface.trim();
    let idx = object_lexemes()
        .iter()
        .position(|lexeme| {
            lexeme
                .accepted_forms
                .iter()
                .any(|form| form.eq_ignore_ascii_case(normalized))
        })
        .ok_or_else(|| {
            CoreError::InvalidSymbolicPlan(format!(
                "unknown surface value '{normalized}' in symbolic inventory"
            ))
        })?;

    u32::try_from(idx).map_err(|_| {
        CoreError::InvalidSymbolicPlan(format!(
            "surface index {idx} is too large for symbolic value conversion"
        ))
    })
}

#[derive(Debug, Clone, Copy)]
struct ItalianAdjectiveLexeme {
    canonical: &'static str,
    accepted_forms: &'static [&'static str],
}

fn adjective_lexemes() -> &'static [ItalianAdjectiveLexeme] {
    const ADJECTIVE_LEXEMES: [ItalianAdjectiveLexeme; 8] = [
        ItalianAdjectiveLexeme {
            canonical: "old",
            accepted_forms: &["old", "aged"],
        },
        ItalianAdjectiveLexeme {
            canonical: "new",
            accepted_forms: &["new"],
        },
        ItalianAdjectiveLexeme {
            canonical: "quiet",
            accepted_forms: &["quiet", "concise"],
        },
        ItalianAdjectiveLexeme {
            canonical: "bright",
            accepted_forms: &["bright", "luminous"],
        },
        ItalianAdjectiveLexeme {
            canonical: "warm",
            accepted_forms: &["warm", "recent", "hearty"],
        },
        ItalianAdjectiveLexeme {
            canonical: "fresh",
            accepted_forms: &["fresh", "current", "novel"],
        },
        ItalianAdjectiveLexeme {
            canonical: "small",
            accepted_forms: &["small", "modest"],
        },
        ItalianAdjectiveLexeme {
            canonical: "clear",
            accepted_forms: &["clear", "lucid"],
        },
    ];

    &ADJECTIVE_LEXEMES
}

fn adjective_lexeme_for_value(value: u32) -> &'static ItalianAdjectiveLexeme {
    let index = (value as usize) % adjective_lexemes().len();
    &adjective_lexemes()[index]
}

fn select_adjective_surface(
    profile: ItalianEncodeProfile,
    value: u32,
    class: ItalianObjectClass,
    object_value: u32,
) -> String {
    let lexeme = adjective_lexeme_for_value(value);
    let base_surface = match (lexeme.canonical, class) {
        ("quiet", ItalianObjectClass::Document | ItalianObjectClass::Data) => "concise",
        ("warm", ItalianObjectClass::Document | ItalianObjectClass::Data) => "recent",
        ("fresh", ItalianObjectClass::Document | ItalianObjectClass::Data) => "current",
        _ => lexeme.canonical,
    };

    let profile_surface = author_adjective_variant(profile, lexeme.canonical, value, object_value);
    profile_surface.unwrap_or(base_surface).to_string()
}

fn author_adjective_variant(
    profile: ItalianEncodeProfile,
    canonical: &str,
    value: u32,
    object_value: u32,
) -> Option<&'static str> {
    if !is_light_profile_variant(value, object_value as usize, 1) {
        return None;
    }

    match profile {
        ItalianEncodeProfile::ShakespeareInspiredLight => match canonical {
            "bright" => Some("luminous"),
            "old" => Some("aged"),
            _ => None,
        },
        ItalianEncodeProfile::DickensInspiredLight => match canonical {
            "warm" => Some("hearty"),
            "small" => Some("modest"),
            _ => None,
        },
        ItalianEncodeProfile::AustenInspiredLight => match canonical {
            "clear" => Some("lucid"),
            "fresh" => Some("novel"),
            _ => None,
        },
        ItalianEncodeProfile::NeutralPrototype => None,
    }
}

fn is_light_profile_variant(left: u32, right: usize, salt: u64) -> bool {
    let mix = (u64::from(left)).wrapping_mul(0x9E37_79B9_7F4A_7C15)
        ^ (right as u64).wrapping_mul(0xD1B5_4A32_D192_ED03)
        ^ salt;
    (mix & 0b11) == 0
}

fn adjective_surface_index(surface: &str) -> CoreResult<u32> {
    let normalized = surface.trim();
    let idx = adjective_lexemes()
        .iter()
        .position(|lexeme| {
            lexeme
                .accepted_forms
                .iter()
                .any(|form| form.eq_ignore_ascii_case(normalized))
        })
        .ok_or_else(|| {
            CoreError::InvalidSymbolicPlan(format!(
                "unknown surface value '{normalized}' in symbolic inventory"
            ))
        })?;

    u32::try_from(idx).map_err(|_| {
        CoreError::InvalidSymbolicPlan(format!(
            "surface index {idx} is too large for symbolic value conversion"
        ))
    })
}

#[cfg(test)]
fn adjective_forms() -> &'static [&'static str] {
    &[
        "old", "new", "quiet", "bright", "warm", "fresh", "small", "clear",
    ]
}

#[derive(Debug, Clone, Copy)]
struct ItalianVerbLexeme {
    canonical: &'static str,
    accepted_forms: &'static [&'static str],
}

fn verb_lexemes() -> &'static [ItalianVerbLexeme] {
    const VERB_LEXEMES: [ItalianVerbLexeme; 32] = [
        ItalianVerbLexeme {
            canonical: "buys",
            accepted_forms: &["buys", "acquires"],
        },
        ItalianVerbLexeme {
            canonical: "writes",
            accepted_forms: &["writes", "composes"],
        },
        ItalianVerbLexeme {
            canonical: "reads",
            accepted_forms: &["reads", "peruses"],
        },
        ItalianVerbLexeme {
            canonical: "sees",
            accepted_forms: &["sees", "observes"],
        },
        ItalianVerbLexeme {
            canonical: "keeps",
            accepted_forms: &["keeps", "retains"],
        },
        ItalianVerbLexeme {
            canonical: "builds",
            accepted_forms: &["builds", "crafts"],
        },
        ItalianVerbLexeme {
            canonical: "finds",
            accepted_forms: &["finds"],
        },
        ItalianVerbLexeme {
            canonical: "moves",
            accepted_forms: &["moves"],
        },
        ItalianVerbLexeme {
            canonical: "sends",
            accepted_forms: &["sends"],
        },
        ItalianVerbLexeme {
            canonical: "brings",
            accepted_forms: &["brings"],
        },
        ItalianVerbLexeme {
            canonical: "drafts",
            accepted_forms: &["drafts"],
        },
        ItalianVerbLexeme {
            canonical: "reviews",
            accepted_forms: &["reviews", "considers"],
        },
        ItalianVerbLexeme {
            canonical: "files",
            accepted_forms: &["files"],
        },
        ItalianVerbLexeme {
            canonical: "studies",
            accepted_forms: &["studies", "examines"],
        },
        ItalianVerbLexeme {
            canonical: "cleans",
            accepted_forms: &["cleans"],
        },
        ItalianVerbLexeme {
            canonical: "paints",
            accepted_forms: &["paints"],
        },
        ItalianVerbLexeme {
            canonical: "records",
            accepted_forms: &["records", "chronicles"],
        },
        ItalianVerbLexeme {
            canonical: "prepares",
            accepted_forms: &["prepares"],
        },
        ItalianVerbLexeme {
            canonical: "arranges",
            accepted_forms: &["arranges"],
        },
        ItalianVerbLexeme {
            canonical: "orders",
            accepted_forms: &["orders", "requests"],
        },
        ItalianVerbLexeme {
            canonical: "packs",
            accepted_forms: &["packs"],
        },
        ItalianVerbLexeme {
            canonical: "repairs",
            accepted_forms: &["repairs"],
        },
        ItalianVerbLexeme {
            canonical: "tracks",
            accepted_forms: &["tracks"],
        },
        ItalianVerbLexeme {
            canonical: "updates",
            accepted_forms: &["updates"],
        },
        ItalianVerbLexeme {
            canonical: "shares",
            accepted_forms: &["shares"],
        },
        ItalianVerbLexeme {
            canonical: "stores",
            accepted_forms: &["stores"],
        },
        ItalianVerbLexeme {
            canonical: "edits",
            accepted_forms: &["edits"],
        },
        ItalianVerbLexeme {
            canonical: "prints",
            accepted_forms: &["prints"],
        },
        ItalianVerbLexeme {
            canonical: "checks",
            accepted_forms: &["checks"],
        },
        ItalianVerbLexeme {
            canonical: "labels",
            accepted_forms: &["labels"],
        },
        ItalianVerbLexeme {
            canonical: "folds",
            accepted_forms: &["folds"],
        },
        ItalianVerbLexeme {
            canonical: "copies",
            accepted_forms: &["copies"],
        },
    ];

    &VERB_LEXEMES
}

fn verb_lexeme_for_value(value: u32) -> &'static ItalianVerbLexeme {
    let index = (value as usize) % verb_lexemes().len();
    &verb_lexemes()[index]
}

fn select_verb_surface(profile: ItalianEncodeProfile, value: u32, object_value: u32) -> String {
    let lexeme = verb_lexeme_for_value(value);
    let profile_surface = author_verb_variant(profile, lexeme.canonical, value, object_value);
    profile_surface.unwrap_or(lexeme.canonical).to_string()
}

fn author_verb_variant(
    profile: ItalianEncodeProfile,
    canonical: &str,
    value: u32,
    object_value: u32,
) -> Option<&'static str> {
    if !is_light_profile_variant(value, object_value as usize, 3) {
        return None;
    }

    match profile {
        ItalianEncodeProfile::ShakespeareInspiredLight => match canonical {
            "writes" => Some("composes"),
            "reads" => Some("peruses"),
            "records" => Some("chronicles"),
            _ => None,
        },
        ItalianEncodeProfile::DickensInspiredLight => match canonical {
            "builds" => Some("crafts"),
            "studies" => Some("examines"),
            _ => None,
        },
        ItalianEncodeProfile::AustenInspiredLight => match canonical {
            "reviews" => Some("considers"),
            "orders" => Some("requests"),
            _ => None,
        },
        ItalianEncodeProfile::NeutralPrototype => None,
    }
}

fn verb_surface_index(surface: &str) -> CoreResult<u32> {
    let normalized = surface.trim();
    let idx = verb_lexemes()
        .iter()
        .position(|lexeme| {
            lexeme
                .accepted_forms
                .iter()
                .any(|form| form.eq_ignore_ascii_case(normalized))
        })
        .ok_or_else(|| {
            CoreError::InvalidSymbolicPlan(format!(
                "unknown surface value '{normalized}' in symbolic inventory"
            ))
        })?;

    u32::try_from(idx).map_err(|_| {
        CoreError::InvalidSymbolicPlan(format!(
            "surface index {idx} is too large for symbolic value conversion"
        ))
    })
}

#[cfg(test)]
fn verb_forms() -> &'static [&'static str] {
    &[
        "buys", "writes", "reads", "sees", "keeps", "builds", "finds", "moves", "sends", "brings",
        "drafts", "reviews", "files", "studies", "cleans", "paints", "records", "prepares",
        "arranges", "orders", "packs", "repairs", "tracks", "updates", "shares", "stores", "edits",
        "prints", "checks", "labels", "folds", "copies",
    ]
}

fn time_forms() -> &'static [&'static str] {
    &[
        "today",
        "yesterday",
        "at dawn",
        "at noon",
        "in spring",
        "tonight",
        "in winter",
        "tomorrow",
    ]
}

fn location_forms() -> &'static [&'static str] {
    &[
        "at home",
        "in the library",
        "in the market",
        "at school",
        "in the park",
        "at the office",
        "in the kitchen",
        "on the street",
    ]
}

#[cfg(test)]
mod tests {
    use linguasteg_core::{
        BitRange, GrammarConstraintChecker, LanguageRealizer, LanguageTag, RealizationPlan,
        SlotAssignment, SlotId, StyleProfileId, StyleProfileRegistry, SymbolicFramePlan,
        SymbolicPayloadPlan, SymbolicSlotValue, TemplateId, TemplateRegistry,
    };

    use super::{
        IT_PROFILE_SHAKESPEARE_LIGHT, ItalianPrototypeConstraintChecker,
        ItalianPrototypeLanguagePack, ItalianPrototypeRealizer, ItalianPrototypeSymbolicMapper,
        adjective_forms, location_forms, object_forms, parse_italian_prototype_text, subject_forms,
        time_forms, verb_forms,
    };

    #[test]
    fn italian_pack_exposes_templates_and_style_profiles() {
        let pack = ItalianPrototypeLanguagePack::default();
        let en = LanguageTag::new("it").expect("valid language");
        let profiles = pack.style_profiles_for_language(&en);

        assert_eq!(pack.templates_for_language(&en).len(), 2);
        assert_eq!(profiles.len(), 4);
        assert!(
            profiles
                .iter()
                .any(|profile| profile.id.as_str() == IT_PROFILE_SHAKESPEARE_LIGHT)
        );
    }

    #[test]
    fn italian_mapper_maps_frame_to_plan() {
        let mapper = ItalianPrototypeSymbolicMapper;
        let schemas = mapper.frame_schemas();
        let frame = linguasteg_core::SymbolicFramePlan {
            template_id: TemplateId::new("it-basic-svo").expect("valid template"),
            source: linguasteg_core::BitRange {
                start_bit: 0,
                consumed_bits: 18,
            },
            values: vec![
                linguasteg_core::SymbolicSlotValue {
                    slot: SlotId::new("subject").expect("slot"),
                    bit_width: 5,
                    value: 1,
                },
                linguasteg_core::SymbolicSlotValue {
                    slot: SlotId::new("object").expect("slot"),
                    bit_width: 5,
                    value: 2,
                },
                linguasteg_core::SymbolicSlotValue {
                    slot: SlotId::new("adjective").expect("slot"),
                    bit_width: 3,
                    value: 3,
                },
                linguasteg_core::SymbolicSlotValue {
                    slot: SlotId::new("verb").expect("slot"),
                    bit_width: 5,
                    value: 4,
                },
            ],
        };

        let plan = mapper
            .map_frame_to_plan(&frame)
            .expect("mapping should succeed");
        assert_eq!(plan.template_id.as_str(), "it-basic-svo");
        assert_eq!(plan.assignments.len(), schemas[0].fields.len());
    }

    #[test]
    fn italian_checker_rejects_empty_surface() {
        let pack = ItalianPrototypeLanguagePack::default();
        let checker = ItalianPrototypeConstraintChecker;
        let template = pack
            .template(&TemplateId::new("it-basic-svo").expect("valid template"))
            .expect("template should exist");
        let plan = RealizationPlan {
            template_id: TemplateId::new("it-basic-svo").expect("valid template"),
            assignments: vec![
                SlotAssignment {
                    slot: SlotId::new("subject").expect("slot"),
                    surface: " ".to_string(),
                    lemma: None,
                },
                SlotAssignment {
                    slot: SlotId::new("verb").expect("slot"),
                    surface: "writes".to_string(),
                    lemma: None,
                },
                SlotAssignment {
                    slot: SlotId::new("adjective").expect("slot"),
                    surface: "old".to_string(),
                    lemma: None,
                },
                SlotAssignment {
                    slot: SlotId::new("object").expect("slot"),
                    surface: "book".to_string(),
                    lemma: None,
                },
            ],
        };

        assert!(checker.validate_plan(template, &plan).is_err());
    }

    #[test]
    fn italian_realizer_renders_sentence() {
        let pack = ItalianPrototypeLanguagePack::default();
        let realizer = ItalianPrototypeRealizer;
        let template = pack
            .template(&TemplateId::new("it-basic-svo").expect("valid template"))
            .expect("template should exist");
        let plan = RealizationPlan {
            template_id: TemplateId::new("it-basic-svo").expect("valid template"),
            assignments: vec![
                SlotAssignment {
                    slot: SlotId::new("subject").expect("slot"),
                    surface: "the writer".to_string(),
                    lemma: None,
                },
                SlotAssignment {
                    slot: SlotId::new("verb").expect("slot"),
                    surface: "writes".to_string(),
                    lemma: None,
                },
                SlotAssignment {
                    slot: SlotId::new("adjective").expect("slot"),
                    surface: "old".to_string(),
                    lemma: None,
                },
                SlotAssignment {
                    slot: SlotId::new("object").expect("slot"),
                    surface: "book".to_string(),
                    lemma: None,
                },
            ],
        };

        let rendered = realizer
            .render(template, &plan)
            .expect("realization should succeed");
        assert_eq!(rendered, "the writer writes old book");
    }

    #[test]
    fn italian_symbolic_inventories_match_bit_width_capacity() {
        assert_eq!(subject_forms().len(), 32);
        assert_eq!(object_forms().len(), 32);
        assert_eq!(verb_forms().len(), 32);
        assert_eq!(adjective_forms().len(), 8);
        assert_eq!(time_forms().len(), 8);
        assert_eq!(location_forms().len(), 8);
    }

    #[test]
    fn italian_mapper_maps_plans_back_to_frames_with_canonical_values() {
        let mapper = ItalianPrototypeSymbolicMapper;
        let frames = vec![
            SymbolicFramePlan {
                template_id: TemplateId::new("it-basic-svo").expect("template"),
                source: BitRange {
                    start_bit: 0,
                    consumed_bits: 18,
                },
                values: vec![
                    SymbolicSlotValue {
                        slot: SlotId::new("subject").expect("slot"),
                        bit_width: 5,
                        value: 31,
                    },
                    SymbolicSlotValue {
                        slot: SlotId::new("object").expect("slot"),
                        bit_width: 5,
                        value: 30,
                    },
                    SymbolicSlotValue {
                        slot: SlotId::new("adjective").expect("slot"),
                        bit_width: 3,
                        value: 7,
                    },
                    SymbolicSlotValue {
                        slot: SlotId::new("verb").expect("slot"),
                        bit_width: 5,
                        value: 29,
                    },
                ],
            },
            SymbolicFramePlan {
                template_id: TemplateId::new("it-time-location-svo").expect("template"),
                source: BitRange {
                    start_bit: 18,
                    consumed_bits: 21,
                },
                values: vec![
                    SymbolicSlotValue {
                        slot: SlotId::new("subject").expect("slot"),
                        bit_width: 5,
                        value: 18,
                    },
                    SymbolicSlotValue {
                        slot: SlotId::new("time").expect("slot"),
                        bit_width: 3,
                        value: 6,
                    },
                    SymbolicSlotValue {
                        slot: SlotId::new("location").expect("slot"),
                        bit_width: 3,
                        value: 5,
                    },
                    SymbolicSlotValue {
                        slot: SlotId::new("object").expect("slot"),
                        bit_width: 5,
                        value: 17,
                    },
                    SymbolicSlotValue {
                        slot: SlotId::new("verb").expect("slot"),
                        bit_width: 5,
                        value: 16,
                    },
                ],
            },
        ];

        let plans = frames
            .iter()
            .map(|frame| mapper.map_frame_to_plan(frame))
            .collect::<Result<Vec<_>, _>>()
            .expect("forward mapping should work");

        let recovered = mapper
            .map_plans_to_frames(&plans)
            .expect("reverse mapping should work");

        assert_eq!(recovered, frames);
    }

    #[test]
    fn italian_text_parser_parses_canonical_sentences() {
        let text = "the manager labels clear report. the architect in winter at the office, records manual.";
        let plans = parse_italian_prototype_text(text).expect("text should parse");

        assert_eq!(plans.len(), 2);
        assert_eq!(plans[0].template_id.as_str(), "it-basic-svo");
        assert_eq!(plans[1].template_id.as_str(), "it-time-location-svo");
    }

    #[test]
    fn italian_text_parser_accepts_legacy_object_aliases() {
        let text = "the writer writes quiet tea. the teacher today at home, keeps flower.";
        let plans = parse_italian_prototype_text(text).expect("legacy aliases should parse");

        assert_eq!(plans.len(), 2);
        let first_object = plans[0]
            .assignments
            .iter()
            .find(|assignment| assignment.slot.as_str() == "object")
            .expect("object assignment should exist");
        let second_object = plans[1]
            .assignments
            .iter()
            .find(|assignment| assignment.slot.as_str() == "object")
            .expect("object assignment should exist");

        assert_eq!(first_object.surface, "journal");
        assert_eq!(second_object.surface, "dossier");
    }

    #[test]
    fn italian_text_parser_accepts_profile_verb_aliases() {
        let text = "the writer composes quiet letter. the teacher today at home, considers report.";
        let plans = parse_italian_prototype_text(text).expect("profile verb aliases should parse");

        assert_eq!(plans.len(), 2);
        let first_verb = plans[0]
            .assignments
            .iter()
            .find(|assignment| assignment.slot.as_str() == "verb")
            .expect("verb assignment should exist");
        let second_verb = plans[1]
            .assignments
            .iter()
            .find(|assignment| assignment.slot.as_str() == "verb")
            .expect("verb assignment should exist");

        assert_eq!(first_verb.surface, "writes");
        assert_eq!(second_verb.surface, "reviews");
    }

    #[test]
    fn italian_mapper_emits_upgraded_object_surface_for_legacy_slot_index() {
        let mapper = ItalianPrototypeSymbolicMapper;
        let frame = SymbolicFramePlan {
            template_id: TemplateId::new("it-basic-svo").expect("template"),
            source: BitRange {
                start_bit: 0,
                consumed_bits: 18,
            },
            values: vec![
                SymbolicSlotValue {
                    slot: SlotId::new("subject").expect("slot"),
                    bit_width: 5,
                    value: 0,
                },
                SymbolicSlotValue {
                    slot: SlotId::new("object").expect("slot"),
                    bit_width: 5,
                    value: 3,
                },
                SymbolicSlotValue {
                    slot: SlotId::new("adjective").expect("slot"),
                    bit_width: 3,
                    value: 2,
                },
                SymbolicSlotValue {
                    slot: SlotId::new("verb").expect("slot"),
                    bit_width: 5,
                    value: 1,
                },
            ],
        };

        let plan = mapper
            .map_frame_to_plan(&frame)
            .expect("mapping should succeed");
        let object = plan
            .assignments
            .iter()
            .find(|assignment| assignment.slot.as_str() == "object")
            .expect("object assignment should exist");
        assert_eq!(object.surface, "journal");
    }

    #[test]
    fn italian_mapper_profile_variants_remain_symbolically_reversible() {
        let mapper = ItalianPrototypeSymbolicMapper;
        let object_value = 1u32;
        let verb_value = (0u32..32)
            .find(|value| super::is_light_profile_variant(object_value, *value as usize, 2))
            .expect("a profile variant gate should exist");
        let payload_plan = SymbolicPayloadPlan {
            original_len_bytes: 2,
            encoded_len_bytes: 2,
            length_prefix_bytes: 2,
            padding_bits: 0,
            frames: vec![SymbolicFramePlan {
                template_id: TemplateId::new("it-basic-svo").expect("template"),
                source: BitRange {
                    start_bit: 0,
                    consumed_bits: 18,
                },
                values: vec![
                    SymbolicSlotValue {
                        slot: SlotId::new("subject").expect("slot"),
                        bit_width: 5,
                        value: 0,
                    },
                    SymbolicSlotValue {
                        slot: SlotId::new("object").expect("slot"),
                        bit_width: 5,
                        value: object_value,
                    },
                    SymbolicSlotValue {
                        slot: SlotId::new("adjective").expect("slot"),
                        bit_width: 3,
                        value: 0,
                    },
                    SymbolicSlotValue {
                        slot: SlotId::new("verb").expect("slot"),
                        bit_width: 5,
                        value: verb_value,
                    },
                ],
            }],
        };
        let profile_id =
            StyleProfileId::new(IT_PROFILE_SHAKESPEARE_LIGHT).expect("valid style profile id");

        let plans = mapper
            .map_payload_to_plans_with_profile(&payload_plan, Some(&profile_id))
            .expect("profile mapping should succeed");
        let object_surface = plans[0]
            .assignments
            .iter()
            .find(|assignment| assignment.slot.as_str() == "object")
            .expect("object assignment should exist")
            .surface
            .as_str();
        assert_eq!(object_surface, "epistle");

        let recovered = mapper
            .map_plans_to_frames(&plans)
            .expect("reverse mapping should preserve symbolic frame");
        assert_eq!(recovered, payload_plan.frames);
    }
}
