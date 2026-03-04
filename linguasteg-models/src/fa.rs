use linguasteg_core::{
    BitRange, CoreError, CoreResult, FixedWidthPlanningOptions, GrammarConstraintChecker,
    LanguageDescriptor, LanguageRealizer, LanguageRegistry, LanguageTag, RealizationPlan,
    RealizationTemplateDescriptor, SlotAssignment, SlotId, SlotRole, StyleInspiration,
    StyleProfileDescriptor, StyleProfileId, StyleProfileRegistry, StyleStrength, SymbolicFieldSpec,
    SymbolicFramePlan, SymbolicFrameSchema, SymbolicPayloadPlan, SymbolicSlotValue, TemplateId,
    TemplateRegistry, TemplateSlotDescriptor, TemplateToken, TextDirection, WritingRegister,
    decode_payload_from_symbolic_frames, render_realization_plan, validate_realization_plan,
};

#[derive(Debug, Clone)]
pub struct FarsiPrototypeLanguagePack {
    languages: Vec<LanguageDescriptor>,
    style_profiles: Vec<StyleProfileDescriptor>,
    templates: Vec<RealizationTemplateDescriptor>,
}

impl Default for FarsiPrototypeLanguagePack {
    fn default() -> Self {
        Self {
            languages: farsi_languages(),
            style_profiles: farsi_style_profiles(),
            templates: farsi_templates(),
        }
    }
}

impl LanguageRegistry for FarsiPrototypeLanguagePack {
    fn all_languages(&self) -> &[LanguageDescriptor] {
        &self.languages
    }
}

impl StyleProfileRegistry for FarsiPrototypeLanguagePack {
    fn all_style_profiles(&self) -> &[StyleProfileDescriptor] {
        &self.style_profiles
    }
}

impl TemplateRegistry for FarsiPrototypeLanguagePack {
    fn all_templates(&self) -> &[RealizationTemplateDescriptor] {
        &self.templates
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct FarsiPrototypeLexicon;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct FarsiNounLexeme {
    canonical: &'static str,
    accepted_forms: &'static [&'static str],
    semantic_tags: &'static [&'static str],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct FarsiVerbLexeme {
    canonical: &'static str,
    accepted_forms: &'static [&'static str],
    accepted_object_tags: &'static [&'static str],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct FarsiAdjectiveLexeme {
    canonical: &'static str,
    accepted_forms: &'static [&'static str],
    accepted_noun_tags: &'static [&'static str],
}

impl FarsiPrototypeLexicon {
    pub fn is_known_object_noun(surface_or_lemma: &str) -> bool {
        find_noun_lexeme(surface_or_lemma).is_some()
    }

    pub fn is_known_verb(surface_or_lemma: &str) -> bool {
        find_verb_lexeme(surface_or_lemma).is_some()
    }

    pub fn is_known_adjective(surface_or_lemma: &str) -> bool {
        find_adjective_lexeme(surface_or_lemma).is_some()
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct FarsiPrototypeConstraintChecker;

impl GrammarConstraintChecker for FarsiPrototypeConstraintChecker {
    fn validate_plan(
        &self,
        template: &RealizationTemplateDescriptor,
        plan: &RealizationPlan,
    ) -> CoreResult<()> {
        validate_realization_plan(template, plan)?;

        if template.language.as_str() != "fa" {
            return Err(CoreError::UnsupportedLanguage(
                template.language.to_string(),
            ));
        }

        let mut has_subject_role = false;
        let mut has_verb_role = false;

        for slot in &template.slots {
            match slot.role {
                SlotRole::Subject => has_subject_role = true,
                SlotRole::Verb => has_verb_role = true,
                _ => {}
            }
        }

        if !has_subject_role {
            return Err(CoreError::InvalidTemplate(
                "farsi prototype template must include a subject slot".to_string(),
            ));
        }

        if !has_verb_role {
            return Err(CoreError::InvalidTemplate(
                "farsi prototype template must include a verb slot".to_string(),
            ));
        }

        validate_assignment_surfaces(template, &plan.assignments)?;
        validate_lexical_compatibility(template, &plan.assignments)
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct FarsiPrototypeRealizer;

impl LanguageRealizer for FarsiPrototypeRealizer {
    fn render(
        &self,
        template: &RealizationTemplateDescriptor,
        plan: &RealizationPlan,
    ) -> CoreResult<String> {
        FarsiPrototypeConstraintChecker.validate_plan(template, plan)?;
        let rendered = render_realization_plan(template, plan)?;
        Ok(normalize_farsi_spacing(&rendered))
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct FarsiPrototypeSymbolicMapper;

const FA_PROFILE_NEUTRAL_FORMAL: &str = "fa-neutral-formal";
const FA_PROFILE_LITERARY_CLASSIC: &str = "fa-literary-classic-inspired";
const FA_PROFILE_SAADI_LIGHT: &str = "fa-saadi-inspired-light";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FarsiEncodeProfile {
    NeutralFormal,
    LiteraryClassicInspired,
    SaadiInspiredLight,
}

impl FarsiEncodeProfile {
    fn from_profile_id(profile_id: Option<&StyleProfileId>) -> CoreResult<Self> {
        match profile_id.map(StyleProfileId::as_str) {
            None | Some(FA_PROFILE_NEUTRAL_FORMAL) => Ok(Self::NeutralFormal),
            Some(FA_PROFILE_LITERARY_CLASSIC) => Ok(Self::LiteraryClassicInspired),
            Some(FA_PROFILE_SAADI_LIGHT) => Ok(Self::SaadiInspiredLight),
            Some(value) => Err(CoreError::InvalidTemplate(format!(
                "unsupported farsi style profile '{value}'"
            ))),
        }
    }
}

impl FarsiPrototypeSymbolicMapper {
    pub fn frame_schemas(&self) -> Vec<SymbolicFrameSchema> {
        farsi_symbolic_frame_schemas()
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
        let profile = FarsiEncodeProfile::from_profile_id(profile_id)?;
        payload_plan
            .frames
            .iter()
            .map(|frame| self.map_frame_to_plan_with_profile(frame, profile))
            .collect()
    }

    pub fn map_frame_to_plan(&self, frame: &SymbolicFramePlan) -> CoreResult<RealizationPlan> {
        self.map_frame_to_plan_with_profile(frame, FarsiEncodeProfile::NeutralFormal)
    }

    fn map_frame_to_plan_with_profile(
        &self,
        frame: &SymbolicFramePlan,
        profile: FarsiEncodeProfile,
    ) -> CoreResult<RealizationPlan> {
        match frame.template_id.as_str() {
            "fa-basic-sov" => map_basic_sov_frame(frame, profile),
            "fa-time-location-sov" => map_time_location_sov_frame(frame, profile),
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

    pub fn decode_payload_from_plans(
        &self,
        plans: &[RealizationPlan],
        options: &FixedWidthPlanningOptions,
    ) -> CoreResult<Vec<u8>> {
        let schemas = self.frame_schemas();
        let frames = self.map_plans_to_frames(plans)?;
        let ordered_schemas = frames
            .iter()
            .map(|frame| schema_for_template(&frame.template_id, &schemas).cloned())
            .collect::<CoreResult<Vec<_>>>()?;
        decode_payload_from_symbolic_frames(&frames, &ordered_schemas, options)
    }
}

fn map_basic_sov_frame(
    frame: &SymbolicFramePlan,
    profile: FarsiEncodeProfile,
) -> CoreResult<RealizationPlan> {
    let subject_value = symbolic_value_for_slot(frame, "subject")?;
    let object_value = symbolic_value_for_slot(frame, "object")?;
    let adjective_value = symbolic_value_for_slot(frame, "adjective")?;
    let verb_value = symbolic_value_for_slot(frame, "verb")?;

    let subject_surface = select_surface(subject_forms(), subject_value)?;
    let object_lexeme = select_noun_lexeme(object_value)?;
    let adjective_lexeme = select_compatible_adjective_lexeme(object_lexeme, adjective_value)?;
    let verb_lexeme = select_compatible_verb_lexeme(object_lexeme, verb_value)?;
    let object_surface = style_noun_surface(profile, object_lexeme);
    let adjective_surface = style_adjective_surface(profile, adjective_lexeme);
    let verb_surface = style_verb_surface(profile, verb_lexeme);

    Ok(RealizationPlan {
        template_id: TemplateId::new("fa-basic-sov")?,
        assignments: vec![
            create_assignment("subject", subject_surface, None)?,
            create_assignment("object", object_surface, Some(object_lexeme.canonical))?,
            create_assignment(
                "adjective",
                adjective_surface,
                Some(adjective_lexeme.canonical),
            )?,
            create_assignment("verb", verb_surface, Some(verb_lexeme.canonical))?,
        ],
    })
}

fn map_time_location_sov_frame(
    frame: &SymbolicFramePlan,
    profile: FarsiEncodeProfile,
) -> CoreResult<RealizationPlan> {
    let subject_value = symbolic_value_for_slot(frame, "subject")?;
    let time_value = symbolic_value_for_slot(frame, "time")?;
    let location_value = symbolic_value_for_slot(frame, "location")?;
    let object_value = symbolic_value_for_slot(frame, "object")?;
    let verb_value = symbolic_value_for_slot(frame, "verb")?;

    let subject_surface = select_surface(subject_forms(), subject_value)?;
    let time_surface = select_surface(time_forms(), time_value)?;
    let location_surface = select_surface(location_forms(), location_value)?;
    let object_lexeme = select_noun_lexeme(object_value)?;
    let verb_lexeme = select_compatible_verb_lexeme(object_lexeme, verb_value)?;
    let object_surface = style_noun_surface(profile, object_lexeme);
    let verb_surface = style_verb_surface(profile, verb_lexeme);

    Ok(RealizationPlan {
        template_id: TemplateId::new("fa-time-location-sov")?,
        assignments: vec![
            create_assignment("subject", subject_surface, None)?,
            create_assignment("time", time_surface, None)?,
            create_assignment("location", location_surface, None)?,
            create_assignment("object", object_surface, Some(object_lexeme.canonical))?,
            create_assignment("verb", verb_surface, Some(verb_lexeme.canonical))?,
        ],
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
                "fa-basic-sov" => symbolic_value_for_basic_plan_slot(plan, field.slot.as_str())?,
                "fa-time-location-sov" => {
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
            let noun = noun_from_assignment(assignment)?;
            noun_index(noun)
        }
        "adjective" => {
            let object_assignment = assignment_by_slot(plan, "object")?;
            let adjective_assignment = assignment_by_slot(plan, "adjective")?;
            let noun = noun_from_assignment(object_assignment)?;
            let adjective = adjective_from_assignment(adjective_assignment)?;
            compatible_adjective_index(noun, adjective)
        }
        "verb" => {
            let object_assignment = assignment_by_slot(plan, "object")?;
            let verb_assignment = assignment_by_slot(plan, "verb")?;
            let noun = noun_from_assignment(object_assignment)?;
            let verb = verb_from_assignment(verb_assignment)?;
            compatible_verb_index(noun, verb)
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
            let noun = noun_from_assignment(assignment)?;
            noun_index(noun)
        }
        "verb" => {
            let object_assignment = assignment_by_slot(plan, "object")?;
            let verb_assignment = assignment_by_slot(plan, "verb")?;
            let noun = noun_from_assignment(object_assignment)?;
            let verb = verb_from_assignment(verb_assignment)?;
            compatible_verb_index(noun, verb)
        }
        _ => Err(CoreError::InvalidSymbolicPlan(format!(
            "unsupported slot '{slot}' for template '{}'",
            plan.template_id
        ))),
    }
}

fn symbolic_value_for_slot(frame: &SymbolicFramePlan, slot_name: &str) -> CoreResult<u32> {
    frame
        .values
        .iter()
        .find(|item| item.slot.as_str() == slot_name)
        .map(|item| item.value)
        .ok_or_else(|| {
            CoreError::InvalidSymbolicPlan(format!(
                "slot '{}' is missing in frame '{}'",
                slot_name, frame.template_id
            ))
        })
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
        .position(|candidate| *candidate == normalized)
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

fn noun_from_assignment(assignment: &SlotAssignment) -> CoreResult<&'static FarsiNounLexeme> {
    find_noun_lexeme(assignment_key(assignment)).ok_or_else(|| {
        CoreError::InvalidSymbolicPlan(format!(
            "unknown noun lexeme for slot '{}': {}",
            assignment.slot, assignment.surface
        ))
    })
}

fn verb_from_assignment(assignment: &SlotAssignment) -> CoreResult<&'static FarsiVerbLexeme> {
    find_verb_lexeme(assignment_key(assignment)).ok_or_else(|| {
        CoreError::InvalidSymbolicPlan(format!(
            "unknown verb lexeme for slot '{}': {}",
            assignment.slot, assignment.surface
        ))
    })
}

fn adjective_from_assignment(
    assignment: &SlotAssignment,
) -> CoreResult<&'static FarsiAdjectiveLexeme> {
    find_adjective_lexeme(assignment_key(assignment)).ok_or_else(|| {
        CoreError::InvalidSymbolicPlan(format!(
            "unknown adjective lexeme for slot '{}': {}",
            assignment.slot, assignment.surface
        ))
    })
}

fn noun_index(noun: &FarsiNounLexeme) -> CoreResult<u32> {
    let idx = FARSI_NOUN_LEXEMES
        .iter()
        .position(|candidate| candidate.canonical == noun.canonical)
        .ok_or_else(|| {
            CoreError::InvalidSymbolicPlan(format!(
                "noun lexeme '{}' is not part of inventory",
                noun.canonical
            ))
        })?;
    u32::try_from(idx).map_err(|_| {
        CoreError::InvalidSymbolicPlan(format!(
            "noun index {idx} is too large for symbolic value conversion"
        ))
    })
}

fn compatible_verb_index(noun: &FarsiNounLexeme, verb: &FarsiVerbLexeme) -> CoreResult<u32> {
    let compatible: Vec<&FarsiVerbLexeme> = FARSI_VERB_LEXEMES
        .iter()
        .filter(|candidate| is_verb_compatible(noun, candidate))
        .collect();

    let idx = compatible
        .iter()
        .position(|candidate| candidate.canonical == verb.canonical)
        .ok_or_else(|| {
            CoreError::InvalidSymbolicPlan(format!(
                "verb '{}' is not compatible with object '{}'",
                verb.canonical, noun.canonical
            ))
        })?;

    u32::try_from(idx).map_err(|_| {
        CoreError::InvalidSymbolicPlan(format!(
            "verb index {idx} is too large for symbolic value conversion"
        ))
    })
}

fn compatible_adjective_index(
    noun: &FarsiNounLexeme,
    adjective: &FarsiAdjectiveLexeme,
) -> CoreResult<u32> {
    let compatible: Vec<&FarsiAdjectiveLexeme> = FARSI_ADJECTIVE_LEXEMES
        .iter()
        .filter(|candidate| is_adjective_compatible(noun, candidate))
        .collect();

    let idx = compatible
        .iter()
        .position(|candidate| candidate.canonical == adjective.canonical)
        .ok_or_else(|| {
            CoreError::InvalidSymbolicPlan(format!(
                "adjective '{}' is not compatible with object '{}'",
                adjective.canonical, noun.canonical
            ))
        })?;

    u32::try_from(idx).map_err(|_| {
        CoreError::InvalidSymbolicPlan(format!(
            "adjective index {idx} is too large for symbolic value conversion"
        ))
    })
}

fn create_assignment(
    slot_name: &str,
    surface: String,
    lemma: Option<&str>,
) -> CoreResult<SlotAssignment> {
    Ok(SlotAssignment {
        slot: SlotId::new(slot_name)?,
        surface,
        lemma: lemma.map(ToString::to_string),
    })
}

fn select_surface(values: &[&str], encoded_value: u32) -> CoreResult<String> {
    if values.is_empty() {
        return Err(CoreError::InvalidTemplate(
            "surface inventory must not be empty".to_string(),
        ));
    }
    let idx = (encoded_value as usize) % values.len();
    Ok(values[idx].to_string())
}

fn select_noun_lexeme(encoded_value: u32) -> CoreResult<&'static FarsiNounLexeme> {
    let idx = (encoded_value as usize) % FARSI_NOUN_LEXEMES.len();
    FARSI_NOUN_LEXEMES
        .get(idx)
        .ok_or_else(|| CoreError::InvalidTemplate("noun lexicon is empty".to_string()))
}

fn select_compatible_verb_lexeme(
    noun: &FarsiNounLexeme,
    encoded_value: u32,
) -> CoreResult<&'static FarsiVerbLexeme> {
    let compatible: Vec<&FarsiVerbLexeme> = FARSI_VERB_LEXEMES
        .iter()
        .filter(|verb| is_verb_compatible(noun, verb))
        .collect();

    if compatible.is_empty() {
        return Err(CoreError::InvalidTemplate(format!(
            "no compatible verb found for object '{}'",
            noun.canonical
        )));
    }

    Ok(compatible[(encoded_value as usize) % compatible.len()])
}

fn select_compatible_adjective_lexeme(
    noun: &FarsiNounLexeme,
    encoded_value: u32,
) -> CoreResult<&'static FarsiAdjectiveLexeme> {
    let compatible: Vec<&FarsiAdjectiveLexeme> = FARSI_ADJECTIVE_LEXEMES
        .iter()
        .filter(|adjective| is_adjective_compatible(noun, adjective))
        .collect();

    if compatible.is_empty() {
        return Err(CoreError::InvalidTemplate(format!(
            "no compatible adjective found for object '{}'",
            noun.canonical
        )));
    }

    Ok(compatible[(encoded_value as usize) % compatible.len()])
}

fn style_noun_surface(profile: FarsiEncodeProfile, noun: &FarsiNounLexeme) -> String {
    let surface = match profile {
        FarsiEncodeProfile::NeutralFormal => noun.canonical,
        FarsiEncodeProfile::LiteraryClassicInspired => match noun.canonical {
            "نامه" => "مکتوب",
            "داستان" => "حکایت",
            "پیام" => "پیغام",
            "غذا" => "طعام",
            _ => noun.canonical,
        },
        FarsiEncodeProfile::SaadiInspiredLight => match noun.canonical {
            "نامه" => "مکتوب",
            "پیام" => "پیغام",
            _ => noun.canonical,
        },
    };
    surface.to_string()
}

fn style_verb_surface(profile: FarsiEncodeProfile, verb: &FarsiVerbLexeme) -> String {
    let surface = match profile {
        FarsiEncodeProfile::NeutralFormal => verb.canonical,
        FarsiEncodeProfile::LiteraryClassicInspired => match verb.canonical {
            "نوشت" => "نگاشت",
            "دید" => "نگریست",
            "خورد" => "چشید",
            "نوشید" => "سرکشید",
            _ => verb.canonical,
        },
        FarsiEncodeProfile::SaadiInspiredLight => match verb.canonical {
            "نوشت" => "نگاشت",
            "دید" => "نگریست",
            _ => verb.canonical,
        },
    };
    surface.to_string()
}

fn style_adjective_surface(
    profile: FarsiEncodeProfile,
    adjective: &FarsiAdjectiveLexeme,
) -> String {
    let surface = match profile {
        FarsiEncodeProfile::NeutralFormal => adjective.canonical,
        FarsiEncodeProfile::LiteraryClassicInspired => match adjective.canonical {
            "زیبا" => "خوش",
            "قدیمی" => "کهن",
            "تازه" => "نو",
            _ => adjective.canonical,
        },
        FarsiEncodeProfile::SaadiInspiredLight => match adjective.canonical {
            "زیبا" => "خوش",
            "تازه" => "نو",
            _ => adjective.canonical,
        },
    };
    surface.to_string()
}

fn is_verb_compatible(noun: &FarsiNounLexeme, verb: &FarsiVerbLexeme) -> bool {
    noun.semantic_tags
        .iter()
        .any(|tag| verb.accepted_object_tags.contains(tag))
}

fn is_adjective_compatible(noun: &FarsiNounLexeme, adjective: &FarsiAdjectiveLexeme) -> bool {
    noun.semantic_tags
        .iter()
        .any(|tag| adjective.accepted_noun_tags.contains(tag))
}

fn validate_assignment_surfaces(
    template: &RealizationTemplateDescriptor,
    assignments: &[SlotAssignment],
) -> CoreResult<()> {
    for assignment in assignments {
        let trimmed_surface = assignment.surface.trim();
        if trimmed_surface.is_empty() {
            return Err(CoreError::InvalidTemplate(format!(
                "slot '{}' has empty surface text",
                assignment.slot
            )));
        }

        if let Some(lemma) = &assignment.lemma {
            if lemma.trim().is_empty() {
                return Err(CoreError::InvalidTemplate(format!(
                    "slot '{}' has empty lemma",
                    assignment.slot
                )));
            }
        }

        if let Some(slot) = template
            .slots
            .iter()
            .find(|item| item.id == assignment.slot)
        {
            if matches!(slot.role, SlotRole::Verb) && !looks_like_farsi_or_translit(trimmed_surface)
            {
                return Err(CoreError::InvalidTemplate(format!(
                    "verb slot '{}' has invalid surface text",
                    assignment.slot
                )));
            }
        }
    }

    Ok(())
}

fn validate_lexical_compatibility(
    template: &RealizationTemplateDescriptor,
    assignments: &[SlotAssignment],
) -> CoreResult<()> {
    let object_assignment = find_assignment_by_role(template, assignments, SlotRole::DirectObject);
    let verb_assignment = find_assignment_by_role(template, assignments, SlotRole::Verb);
    let adjective_assignment = find_assignment_by_role(template, assignments, SlotRole::Adjective);

    if let Some(object_assignment) = object_assignment {
        let noun = find_noun_lexeme(assignment_key(object_assignment)).ok_or_else(|| {
            CoreError::InvalidTemplate(format!(
                "unknown object lexeme for slot '{}': {}",
                object_assignment.slot, object_assignment.surface
            ))
        })?;

        if let Some(verb_assignment) = verb_assignment {
            let verb = find_verb_lexeme(assignment_key(verb_assignment)).ok_or_else(|| {
                CoreError::InvalidTemplate(format!(
                    "unknown verb lexeme for slot '{}': {}",
                    verb_assignment.slot, verb_assignment.surface
                ))
            })?;

            if !noun
                .semantic_tags
                .iter()
                .any(|tag| verb.accepted_object_tags.contains(tag))
            {
                return Err(CoreError::InvalidTemplate(format!(
                    "verb '{}' is not compatible with object '{}'",
                    verb.canonical, noun.canonical
                )));
            }
        }

        if let Some(adjective_assignment) = adjective_assignment {
            let adjective = find_adjective_lexeme(assignment_key(adjective_assignment))
                .ok_or_else(|| {
                    CoreError::InvalidTemplate(format!(
                        "unknown adjective lexeme for slot '{}': {}",
                        adjective_assignment.slot, adjective_assignment.surface
                    ))
                })?;

            if !noun
                .semantic_tags
                .iter()
                .any(|tag| adjective.accepted_noun_tags.contains(tag))
            {
                return Err(CoreError::InvalidTemplate(format!(
                    "adjective '{}' is not compatible with object '{}'",
                    adjective.canonical, noun.canonical
                )));
            }
        }
    }

    Ok(())
}

fn find_assignment_by_role<'a>(
    template: &'a RealizationTemplateDescriptor,
    assignments: &'a [SlotAssignment],
    role: SlotRole,
) -> Option<&'a SlotAssignment> {
    template
        .slots
        .iter()
        .find(|slot| slot.role == role)
        .and_then(|slot| {
            assignments
                .iter()
                .find(|assignment| assignment.slot == slot.id)
        })
}

fn assignment_key(assignment: &SlotAssignment) -> &str {
    assignment
        .lemma
        .as_deref()
        .unwrap_or(&assignment.surface)
        .trim()
}

fn looks_like_farsi_or_translit(value: &str) -> bool {
    value.chars().any(|ch| ch.is_alphabetic())
}

fn normalize_farsi_spacing(input: &str) -> String {
    input
        .replace(" ،", "،")
        .replace(" .", ".")
        .replace(" !", "!")
        .replace(" ؟", "؟")
}

fn farsi_languages() -> Vec<LanguageDescriptor> {
    vec![LanguageDescriptor {
        tag: fa_tag(),
        display_name: "Persian".to_string(),
        direction: TextDirection::RightToLeft,
    }]
}

fn farsi_style_profiles() -> Vec<StyleProfileDescriptor> {
    vec![
        StyleProfileDescriptor {
            id: StyleProfileId::new("fa-neutral-formal").expect("valid style id"),
            language: fa_tag(),
            display_name: "Formal Persian (Neutral)".to_string(),
            register: WritingRegister::Formal,
            strength: StyleStrength::Light,
            inspiration: StyleInspiration::RegisterOnly,
        },
        StyleProfileDescriptor {
            id: StyleProfileId::new("fa-literary-classic-inspired").expect("valid style id"),
            language: fa_tag(),
            display_name: "Classical Persian Inspired".to_string(),
            register: WritingRegister::Literary,
            strength: StyleStrength::Medium,
            inspiration: StyleInspiration::EraInspired {
                era_label: "Classical Persian Prose".to_string(),
            },
        },
        StyleProfileDescriptor {
            id: StyleProfileId::new("fa-saadi-inspired-light").expect("valid style id"),
            language: fa_tag(),
            display_name: "Saadi-inspired (Light)".to_string(),
            register: WritingRegister::Literary,
            strength: StyleStrength::Light,
            inspiration: StyleInspiration::PublicDomainAuthorInspired {
                author_label: "Saadi".to_string(),
            },
        },
    ]
}

fn farsi_templates() -> Vec<RealizationTemplateDescriptor> {
    vec![
        RealizationTemplateDescriptor {
            id: TemplateId::new("fa-basic-sov").expect("valid template id"),
            language: fa_tag(),
            display_name: "Basic SOV".to_string(),
            slots: vec![
                slot("subject", SlotRole::Subject, true),
                slot("object", SlotRole::DirectObject, true),
                slot("adjective", SlotRole::Adjective, true),
                slot("verb", SlotRole::Verb, true),
            ],
            tokens: vec![
                TemplateToken::Slot(SlotId::new("subject").expect("valid slot id")),
                TemplateToken::Slot(SlotId::new("object").expect("valid slot id")),
                TemplateToken::Slot(SlotId::new("adjective").expect("valid slot id")),
                TemplateToken::Literal("را".to_string()),
                TemplateToken::Slot(SlotId::new("verb").expect("valid slot id")),
            ],
        },
        RealizationTemplateDescriptor {
            id: TemplateId::new("fa-time-location-sov").expect("valid template id"),
            language: fa_tag(),
            display_name: "Time + Location + SOV".to_string(),
            slots: vec![
                slot("subject", SlotRole::Subject, true),
                slot("time", SlotRole::Time, true),
                slot("location", SlotRole::Location, true),
                slot("object", SlotRole::DirectObject, true),
                slot("verb", SlotRole::Verb, true),
            ],
            tokens: vec![
                TemplateToken::Slot(SlotId::new("subject").expect("valid slot id")),
                TemplateToken::Slot(SlotId::new("time").expect("valid slot id")),
                TemplateToken::Literal("در".to_string()),
                TemplateToken::Slot(SlotId::new("location").expect("valid slot id")),
                TemplateToken::Slot(SlotId::new("object").expect("valid slot id")),
                TemplateToken::Literal("را".to_string()),
                TemplateToken::Slot(SlotId::new("verb").expect("valid slot id")),
            ],
        },
    ]
}

fn fa_tag() -> LanguageTag {
    LanguageTag::new("fa").expect("valid language tag")
}

fn slot(name: &str, role: SlotRole, required: bool) -> TemplateSlotDescriptor {
    TemplateSlotDescriptor {
        id: SlotId::new(name).expect("valid slot id"),
        role,
        required,
    }
}

fn farsi_symbolic_frame_schemas() -> Vec<SymbolicFrameSchema> {
    vec![
        SymbolicFrameSchema {
            template_id: TemplateId::new("fa-basic-sov").expect("valid template id"),
            fields: vec![
                symbolic_field("subject", 5),
                symbolic_field("object", 5),
                symbolic_field("adjective", 3),
                symbolic_field("verb", 5),
            ],
        },
        SymbolicFrameSchema {
            template_id: TemplateId::new("fa-time-location-sov").expect("valid template id"),
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

fn symbolic_field(slot_name: &str, bit_width: u8) -> SymbolicFieldSpec {
    SymbolicFieldSpec {
        slot: SlotId::new(slot_name).expect("valid slot id"),
        bit_width,
    }
}

fn find_noun_lexeme(value: &str) -> Option<&'static FarsiNounLexeme> {
    let normalized = value.trim();
    FARSI_NOUN_LEXEMES
        .iter()
        .find(|entry| entry.accepted_forms.contains(&normalized))
}

fn find_verb_lexeme(value: &str) -> Option<&'static FarsiVerbLexeme> {
    let normalized = value.trim();
    FARSI_VERB_LEXEMES
        .iter()
        .find(|entry| entry.accepted_forms.contains(&normalized))
}

fn find_adjective_lexeme(value: &str) -> Option<&'static FarsiAdjectiveLexeme> {
    let normalized = value.trim();
    FARSI_ADJECTIVE_LEXEMES
        .iter()
        .find(|entry| entry.accepted_forms.contains(&normalized))
}

const FARSI_NOUN_LEXEMES: &[FarsiNounLexeme] = &[
    FarsiNounLexeme {
        canonical: "کتاب",
        accepted_forms: &["کتاب", "ketab"],
        semantic_tags: &["core", "document", "readable", "physical-object"],
    },
    FarsiNounLexeme {
        canonical: "نامه",
        accepted_forms: &["نامه", "nameh", "مکتوب"],
        semantic_tags: &["core", "document", "message", "physical-object"],
    },
    FarsiNounLexeme {
        canonical: "چای",
        accepted_forms: &["چای", "chay", "tea"],
        semantic_tags: &["core", "drink", "food"],
    },
    FarsiNounLexeme {
        canonical: "غذا",
        accepted_forms: &["غذا", "ghaza", "food", "طعام"],
        semantic_tags: &["core", "food"],
    },
    FarsiNounLexeme {
        canonical: "گل",
        accepted_forms: &["گل", "gol", "flower"],
        semantic_tags: &["core", "plant", "gift", "decorative"],
    },
    FarsiNounLexeme {
        canonical: "عکس",
        accepted_forms: &["عکس", "aks"],
        semantic_tags: &["core", "image", "document", "physical-object"],
    },
    FarsiNounLexeme {
        canonical: "داستان",
        accepted_forms: &["داستان", "حکایت"],
        semantic_tags: &["core"],
    },
    FarsiNounLexeme {
        canonical: "دفتر",
        accepted_forms: &["دفتر"],
        semantic_tags: &["core"],
    },
    FarsiNounLexeme {
        canonical: "قلم",
        accepted_forms: &["قلم"],
        semantic_tags: &["core"],
    },
    FarsiNounLexeme {
        canonical: "کاغذ",
        accepted_forms: &["کاغذ"],
        semantic_tags: &["core"],
    },
    FarsiNounLexeme {
        canonical: "مقاله",
        accepted_forms: &["مقاله"],
        semantic_tags: &["core"],
    },
    FarsiNounLexeme {
        canonical: "گزارش",
        accepted_forms: &["گزارش"],
        semantic_tags: &["core"],
    },
    FarsiNounLexeme {
        canonical: "طرح",
        accepted_forms: &["طرح"],
        semantic_tags: &["core"],
    },
    FarsiNounLexeme {
        canonical: "برنامه",
        accepted_forms: &["برنامه"],
        semantic_tags: &["core"],
    },
    FarsiNounLexeme {
        canonical: "پرونده",
        accepted_forms: &["پرونده"],
        semantic_tags: &["core"],
    },
    FarsiNounLexeme {
        canonical: "سند",
        accepted_forms: &["سند"],
        semantic_tags: &["core"],
    },
    FarsiNounLexeme {
        canonical: "پیام",
        accepted_forms: &["پیام", "پیغام"],
        semantic_tags: &["core"],
    },
    FarsiNounLexeme {
        canonical: "یادداشت",
        accepted_forms: &["یادداشت"],
        semantic_tags: &["core"],
    },
    FarsiNounLexeme {
        canonical: "شعر",
        accepted_forms: &["شعر"],
        semantic_tags: &["core"],
    },
    FarsiNounLexeme {
        canonical: "نقشه",
        accepted_forms: &["نقشه"],
        semantic_tags: &["core"],
    },
    FarsiNounLexeme {
        canonical: "دستگاه",
        accepted_forms: &["دستگاه"],
        semantic_tags: &["core"],
    },
    FarsiNounLexeme {
        canonical: "ابزار",
        accepted_forms: &["ابزار"],
        semantic_tags: &["core"],
    },
    FarsiNounLexeme {
        canonical: "بسته",
        accepted_forms: &["بسته"],
        semantic_tags: &["core"],
    },
    FarsiNounLexeme {
        canonical: "هدیه",
        accepted_forms: &["هدیه"],
        semantic_tags: &["core"],
    },
    FarsiNounLexeme {
        canonical: "سیب",
        accepted_forms: &["سیب"],
        semantic_tags: &["core"],
    },
    FarsiNounLexeme {
        canonical: "نان",
        accepted_forms: &["نان"],
        semantic_tags: &["core"],
    },
    FarsiNounLexeme {
        canonical: "سوپ",
        accepted_forms: &["سوپ"],
        semantic_tags: &["core"],
    },
    FarsiNounLexeme {
        canonical: "قهوه",
        accepted_forms: &["قهوه"],
        semantic_tags: &["core"],
    },
    FarsiNounLexeme {
        canonical: "میوه",
        accepted_forms: &["میوه"],
        semantic_tags: &["core"],
    },
    FarsiNounLexeme {
        canonical: "خوراک",
        accepted_forms: &["خوراک"],
        semantic_tags: &["core"],
    },
    FarsiNounLexeme {
        canonical: "فیلم",
        accepted_forms: &["فیلم"],
        semantic_tags: &["core"],
    },
    FarsiNounLexeme {
        canonical: "مجله",
        accepted_forms: &["مجله"],
        semantic_tags: &["core"],
    },
];

const FARSI_VERB_LEXEMES: &[FarsiVerbLexeme] = &[
    FarsiVerbLexeme {
        canonical: "خرید",
        accepted_forms: &["خرید", "kharid", "bought"],
        accepted_object_tags: &[
            "core",
            "document",
            "food",
            "gift",
            "decorative",
            "physical-object",
        ],
    },
    FarsiVerbLexeme {
        canonical: "نوشت",
        accepted_forms: &["نوشت", "nevesht", "wrote", "نگاشت"],
        accepted_object_tags: &["core", "document", "message"],
    },
    FarsiVerbLexeme {
        canonical: "دید",
        accepted_forms: &["دید", "did", "saw", "نگریست"],
        accepted_object_tags: &[
            "core",
            "document",
            "image",
            "food",
            "gift",
            "decorative",
            "physical-object",
            "plant",
        ],
    },
    FarsiVerbLexeme {
        canonical: "خورد",
        accepted_forms: &["خورد", "khord", "ate", "چشید"],
        accepted_object_tags: &["core", "food"],
    },
    FarsiVerbLexeme {
        canonical: "نوشید",
        accepted_forms: &["نوشید", "noushid", "drank", "سرکشید"],
        accepted_object_tags: &["core", "drink"],
    },
    FarsiVerbLexeme {
        canonical: "خواند",
        accepted_forms: &["خواند"],
        accepted_object_tags: &["core"],
    },
    FarsiVerbLexeme {
        canonical: "برد",
        accepted_forms: &["برد"],
        accepted_object_tags: &["core"],
    },
    FarsiVerbLexeme {
        canonical: "آورد",
        accepted_forms: &["آورد"],
        accepted_object_tags: &["core"],
    },
    FarsiVerbLexeme {
        canonical: "ساخت",
        accepted_forms: &["ساخت"],
        accepted_object_tags: &["core"],
    },
    FarsiVerbLexeme {
        canonical: "یافت",
        accepted_forms: &["یافت"],
        accepted_object_tags: &["core"],
    },
    FarsiVerbLexeme {
        canonical: "گرفت",
        accepted_forms: &["گرفت"],
        accepted_object_tags: &["core"],
    },
    FarsiVerbLexeme {
        canonical: "گذاشت",
        accepted_forms: &["گذاشت"],
        accepted_object_tags: &["core"],
    },
    FarsiVerbLexeme {
        canonical: "گفت",
        accepted_forms: &["گفت"],
        accepted_object_tags: &["core"],
    },
    FarsiVerbLexeme {
        canonical: "شنید",
        accepted_forms: &["شنید"],
        accepted_object_tags: &["core"],
    },
    FarsiVerbLexeme {
        canonical: "داد",
        accepted_forms: &["داد"],
        accepted_object_tags: &["core"],
    },
    FarsiVerbLexeme {
        canonical: "کشید",
        accepted_forms: &["کشید"],
        accepted_object_tags: &["core"],
    },
    FarsiVerbLexeme {
        canonical: "چید",
        accepted_forms: &["چید"],
        accepted_object_tags: &["core"],
    },
    FarsiVerbLexeme {
        canonical: "پخت",
        accepted_forms: &["پخت"],
        accepted_object_tags: &["core"],
    },
    FarsiVerbLexeme {
        canonical: "بست",
        accepted_forms: &["بست"],
        accepted_object_tags: &["core"],
    },
    FarsiVerbLexeme {
        canonical: "گشود",
        accepted_forms: &["گشود"],
        accepted_object_tags: &["core"],
    },
    FarsiVerbLexeme {
        canonical: "فرستاد",
        accepted_forms: &["فرستاد"],
        accepted_object_tags: &["core"],
    },
    FarsiVerbLexeme {
        canonical: "پرداخت",
        accepted_forms: &["پرداخت"],
        accepted_object_tags: &["core"],
    },
    FarsiVerbLexeme {
        canonical: "شمرد",
        accepted_forms: &["شمرد"],
        accepted_object_tags: &["core"],
    },
    FarsiVerbLexeme {
        canonical: "سنجید",
        accepted_forms: &["سنجید"],
        accepted_object_tags: &["core"],
    },
    FarsiVerbLexeme {
        canonical: "دوخت",
        accepted_forms: &["دوخت"],
        accepted_object_tags: &["core"],
    },
    FarsiVerbLexeme {
        canonical: "شست",
        accepted_forms: &["شست"],
        accepted_object_tags: &["core"],
    },
    FarsiVerbLexeme {
        canonical: "ریخت",
        accepted_forms: &["ریخت"],
        accepted_object_tags: &["core"],
    },
    FarsiVerbLexeme {
        canonical: "چرخاند",
        accepted_forms: &["چرخاند"],
        accepted_object_tags: &["core"],
    },
    FarsiVerbLexeme {
        canonical: "افزود",
        accepted_forms: &["افزود"],
        accepted_object_tags: &["core"],
    },
    FarsiVerbLexeme {
        canonical: "کاشت",
        accepted_forms: &["کاشت"],
        accepted_object_tags: &["core"],
    },
    FarsiVerbLexeme {
        canonical: "برداشت",
        accepted_forms: &["برداشت"],
        accepted_object_tags: &["core"],
    },
    FarsiVerbLexeme {
        canonical: "آموخت",
        accepted_forms: &["آموخت"],
        accepted_object_tags: &["core"],
    },
];

const FARSI_ADJECTIVE_LEXEMES: &[FarsiAdjectiveLexeme] = &[
    FarsiAdjectiveLexeme {
        canonical: "زیبا",
        accepted_forms: &["زیبا", "ziba", "خوش"],
        accepted_noun_tags: &[
            "core",
            "document",
            "image",
            "gift",
            "decorative",
            "physical-object",
            "plant",
        ],
    },
    FarsiAdjectiveLexeme {
        canonical: "قدیمی",
        accepted_forms: &["قدیمی", "ghadimi", "کهن"],
        accepted_noun_tags: &["core", "document", "image", "physical-object"],
    },
    FarsiAdjectiveLexeme {
        canonical: "تازه",
        accepted_forms: &["تازه", "taze", "نو"],
        accepted_noun_tags: &["core", "food", "drink", "plant"],
    },
    FarsiAdjectiveLexeme {
        canonical: "گرم",
        accepted_forms: &["گرم", "garm"],
        accepted_noun_tags: &["core", "food", "drink"],
    },
    FarsiAdjectiveLexeme {
        canonical: "روشن",
        accepted_forms: &["روشن"],
        accepted_noun_tags: &["core"],
    },
    FarsiAdjectiveLexeme {
        canonical: "نرم",
        accepted_forms: &["نرم"],
        accepted_noun_tags: &["core"],
    },
    FarsiAdjectiveLexeme {
        canonical: "ساده",
        accepted_forms: &["ساده"],
        accepted_noun_tags: &["core"],
    },
    FarsiAdjectiveLexeme {
        canonical: "دقیق",
        accepted_forms: &["دقیق"],
        accepted_noun_tags: &["core"],
    },
];

fn subject_forms() -> &'static [&'static str] {
    &[
        "مرد",
        "زن",
        "دانشجو",
        "نویسنده",
        "پژوهشگر",
        "معلم",
        "دوست",
        "هنرمند",
        "تحلیلگر",
        "ویراستار",
        "طراح",
        "مدیر",
        "مهندس",
        "پزشک",
        "وکیل",
        "پرستار",
        "کشاورز",
        "راننده",
        "فروشنده",
        "ورزشکار",
        "بازیگر",
        "موسیقیدان",
        "عکاس",
        "آشپز",
        "دریانورد",
        "کوهنورد",
        "مربی",
        "کارمند",
        "حسابدار",
        "سیاستمدار",
        "خبرنگار",
        "ناشر",
    ]
}

fn time_forms() -> &'static [&'static str] {
    &["امروز", "دیروز", "صبح", "عصر", "شب", "سحر", "اکنون", "فردا"]
}

fn location_forms() -> &'static [&'static str] {
    &[
        "خانه",
        "کتابخانه",
        "بازار",
        "دانشگاه",
        "باغ",
        "پارک",
        "مدرسه",
        "آشپزخانه",
    ]
}

#[cfg(test)]
mod tests {
    use linguasteg_core::{
        BitRange, FixedWidthBitPlanner, FixedWidthPlanningOptions, LanguageRealizer,
        LanguageRegistry, RealizationPlan, SlotAssignment, SlotId, StyleProfileRegistry,
        SymbolicFramePlan, SymbolicPayloadPlan, SymbolicPayloadPlanner, SymbolicSlotValue,
        TemplateId, TemplateRegistry, decode_payload_from_symbolic_frames,
    };

    use super::{
        FARSI_ADJECTIVE_LEXEMES, FARSI_NOUN_LEXEMES, FARSI_VERB_LEXEMES,
        FarsiPrototypeConstraintChecker, FarsiPrototypeLanguagePack, FarsiPrototypeLexicon,
        FarsiPrototypeRealizer, FarsiPrototypeSymbolicMapper, location_forms, subject_forms,
        time_forms,
    };
    use linguasteg_core::GrammarConstraintChecker;

    #[test]
    fn farsi_pack_exposes_language_templates_and_style_profiles() {
        let pack = FarsiPrototypeLanguagePack::default();

        let fa = pack
            .all_languages()
            .iter()
            .find(|language| language.tag.as_str() == "fa")
            .expect("fa language should exist");
        assert_eq!(fa.display_name, "Persian");

        let fa_templates = pack.templates_for_language(&fa.tag);
        assert!(!fa_templates.is_empty());

        let fa_profiles = pack.style_profiles_for_language(&fa.tag);
        assert!(fa_profiles.len() >= 2);
    }

    #[test]
    fn constraint_checker_accepts_valid_farsi_plan() {
        let pack = FarsiPrototypeLanguagePack::default();
        let template = pack
            .template(&TemplateId::new("fa-time-location-sov").expect("valid template"))
            .expect("template should exist");

        let plan = RealizationPlan {
            template_id: TemplateId::new("fa-time-location-sov").expect("valid template"),
            assignments: vec![
                assign("subject", "مرد"),
                assign("time", "امروز"),
                assign("location", "بازار"),
                assign("object", "کتاب"),
                assign("verb", "خرید"),
            ],
        };

        FarsiPrototypeConstraintChecker
            .validate_plan(template, &plan)
            .expect("plan should validate");
    }

    #[test]
    fn constraint_checker_rejects_empty_surface() {
        let pack = FarsiPrototypeLanguagePack::default();
        let template = pack
            .template(&TemplateId::new("fa-basic-sov").expect("valid template"))
            .expect("template should exist");

        let plan = RealizationPlan {
            template_id: TemplateId::new("fa-basic-sov").expect("valid template"),
            assignments: vec![
                assign("subject", "زن"),
                assign("object", " "),
                assign("adjective", "زیبا"),
                assign("verb", "دید"),
            ],
        };

        let error = FarsiPrototypeConstraintChecker
            .validate_plan(template, &plan)
            .expect_err("plan should fail");
        assert!(error.to_string().contains("empty surface"));
    }

    #[test]
    fn realizer_renders_farsi_sentence_from_plan() {
        let pack = FarsiPrototypeLanguagePack::default();
        let template = pack
            .template(&TemplateId::new("fa-time-location-sov").expect("valid template"))
            .expect("template should exist");

        let plan = RealizationPlan {
            template_id: TemplateId::new("fa-time-location-sov").expect("valid template"),
            assignments: vec![
                assign("subject", "دانشجو"),
                assign("time", "امروز"),
                assign("location", "کتابخانه"),
                assign("object", "نامه"),
                assign("verb", "نوشت"),
            ],
        };

        let sentence = FarsiPrototypeRealizer
            .render(template, &plan)
            .expect("render should work");
        assert_eq!(sentence, "دانشجو امروز در کتابخانه نامه را نوشت");
    }

    #[test]
    fn constraint_checker_rejects_unknown_verb_lexeme() {
        let pack = FarsiPrototypeLanguagePack::default();
        let template = pack
            .template(&TemplateId::new("fa-time-location-sov").expect("valid template"))
            .expect("template should exist");

        let plan = RealizationPlan {
            template_id: TemplateId::new("fa-time-location-sov").expect("valid template"),
            assignments: vec![
                assign("subject", "دانشجو"),
                assign("time", "امروز"),
                assign("location", "کتابخانه"),
                assign("object", "کتاب"),
                assign("verb", "پرید"),
            ],
        };

        let error = FarsiPrototypeConstraintChecker
            .validate_plan(template, &plan)
            .expect_err("plan should fail");
        assert!(error.to_string().contains("unknown verb lexeme"));
    }

    #[test]
    fn constraint_checker_rejects_unknown_adjective_lexeme() {
        let pack = FarsiPrototypeLanguagePack::default();
        let template = pack
            .template(&TemplateId::new("fa-basic-sov").expect("valid template"))
            .expect("template should exist");

        let plan = RealizationPlan {
            template_id: TemplateId::new("fa-basic-sov").expect("valid template"),
            assignments: vec![
                assign("subject", "زن"),
                assign("object", "کتاب"),
                assign("adjective", "براق"),
                assign("verb", "دید"),
            ],
        };

        let error = FarsiPrototypeConstraintChecker
            .validate_plan(template, &plan)
            .expect_err("plan should fail");
        assert!(error.to_string().contains("unknown adjective lexeme"));
    }

    #[test]
    fn lexicon_recognizes_known_forms() {
        assert!(FarsiPrototypeLexicon::is_known_object_noun("کتاب"));
        assert!(FarsiPrototypeLexicon::is_known_verb("نوشید"));
        assert!(FarsiPrototypeLexicon::is_known_adjective("گرم"));
        assert!(!FarsiPrototypeLexicon::is_known_object_noun("ابر"));
    }

    #[test]
    fn symbolic_inventories_match_bit_width_capacity() {
        assert_eq!(subject_forms().len(), 32);
        assert_eq!(FARSI_NOUN_LEXEMES.len(), 32);
        assert_eq!(FARSI_VERB_LEXEMES.len(), 32);
        assert_eq!(FARSI_ADJECTIVE_LEXEMES.len(), 8);
        assert_eq!(time_forms().len(), 8);
        assert_eq!(location_forms().len(), 8);
    }

    #[test]
    fn symbolic_mapper_exposes_two_frame_schemas() {
        let mapper = FarsiPrototypeSymbolicMapper;
        let schemas = mapper.frame_schemas();

        assert_eq!(schemas.len(), 2);
        assert_eq!(schemas[0].template_id.as_str(), "fa-basic-sov");
        assert_eq!(schemas[1].template_id.as_str(), "fa-time-location-sov");
    }

    #[test]
    fn symbolic_mapper_maps_basic_frame_to_valid_plan() {
        let mapper = FarsiPrototypeSymbolicMapper;
        let pack = FarsiPrototypeLanguagePack::default();

        let frame = symbolic_frame(
            "fa-basic-sov",
            &[
                ("subject", 5, 3),
                ("object", 5, 1),
                ("adjective", 3, 2),
                ("verb", 5, 4),
            ],
        );

        let plan = mapper
            .map_frame_to_plan(&frame)
            .expect("mapping should succeed");

        let template = pack
            .template(&TemplateId::new("fa-basic-sov").expect("valid template"))
            .expect("template should exist");

        FarsiPrototypeConstraintChecker
            .validate_plan(template, &plan)
            .expect("mapped plan should validate");
    }

    #[test]
    fn symbolic_mapper_maps_payload_plan_to_renderable_plans() {
        let mapper = FarsiPrototypeSymbolicMapper;
        let pack = FarsiPrototypeLanguagePack::default();
        let planner = FixedWidthBitPlanner {
            options: FixedWidthPlanningOptions {
                prepend_u16_be_length: false,
            },
        };

        let payload_plan = planner
            .plan_payload(&[0b1010_0110], &mapper.frame_schemas())
            .expect("planning should succeed");
        let plans = mapper
            .map_payload_to_plans(&payload_plan)
            .expect("mapping should succeed");

        assert_eq!(plans.len(), payload_plan.frames.len());

        for plan in &plans {
            let template = pack
                .template(&plan.template_id)
                .expect("template should exist");
            FarsiPrototypeConstraintChecker
                .validate_plan(template, plan)
                .expect("mapped plan should validate");

            let rendered = FarsiPrototypeRealizer
                .render(template, plan)
                .expect("render should work");
            assert!(!rendered.trim().is_empty());
        }
    }

    #[test]
    fn symbolic_mapper_maps_plans_back_to_frames_with_canonical_values() {
        let mapper = FarsiPrototypeSymbolicMapper;
        let original_frames = vec![
            symbolic_frame(
                "fa-basic-sov",
                &[
                    ("subject", 5, 31),
                    ("object", 5, 30),
                    ("adjective", 3, 7),
                    ("verb", 5, 29),
                ],
            ),
            symbolic_frame(
                "fa-basic-sov",
                &[
                    ("subject", 5, 24),
                    ("object", 5, 17),
                    ("adjective", 3, 6),
                    ("verb", 5, 22),
                ],
            ),
            symbolic_frame(
                "fa-basic-sov",
                &[
                    ("subject", 5, 12),
                    ("object", 5, 8),
                    ("adjective", 3, 5),
                    ("verb", 5, 16),
                ],
            ),
            symbolic_frame(
                "fa-basic-sov",
                &[
                    ("subject", 5, 3),
                    ("object", 5, 2),
                    ("adjective", 3, 4),
                    ("verb", 5, 11),
                ],
            ),
        ];
        let payload_plan = SymbolicPayloadPlan {
            original_len_bytes: 0,
            encoded_len_bytes: 0,
            length_prefix_bytes: 0,
            padding_bits: 0,
            frames: original_frames.clone(),
        };

        let plans = mapper
            .map_payload_to_plans(&payload_plan)
            .expect("mapping frames to plans should succeed");
        let recovered_frames = mapper
            .map_plans_to_frames(&plans)
            .expect("mapping plans back to frames should succeed");

        assert_eq!(recovered_frames.len(), original_frames.len());
        for (expected, actual) in original_frames.iter().zip(recovered_frames.iter()) {
            assert_eq!(actual.template_id, expected.template_id);
            let expected_values: Vec<(String, u8, u32)> = expected
                .values
                .iter()
                .map(|value| (value.slot.to_string(), value.bit_width, value.value))
                .collect();
            let actual_values: Vec<(String, u8, u32)> = actual
                .values
                .iter()
                .map(|value| (value.slot.to_string(), value.bit_width, value.value))
                .collect();
            assert_eq!(actual_values, expected_values);
        }
    }

    #[test]
    fn symbolic_mapper_decodes_payload_from_canonical_plans() {
        let mapper = FarsiPrototypeSymbolicMapper;
        let original_frames = vec![
            symbolic_frame(
                "fa-basic-sov",
                &[
                    ("subject", 5, 31),
                    ("object", 5, 30),
                    ("adjective", 3, 7),
                    ("verb", 5, 29),
                ],
            ),
            symbolic_frame(
                "fa-basic-sov",
                &[
                    ("subject", 5, 24),
                    ("object", 5, 17),
                    ("adjective", 3, 6),
                    ("verb", 5, 22),
                ],
            ),
            symbolic_frame(
                "fa-basic-sov",
                &[
                    ("subject", 5, 12),
                    ("object", 5, 8),
                    ("adjective", 3, 5),
                    ("verb", 5, 16),
                ],
            ),
            symbolic_frame(
                "fa-basic-sov",
                &[
                    ("subject", 5, 3),
                    ("object", 5, 2),
                    ("adjective", 3, 4),
                    ("verb", 5, 11),
                ],
            ),
        ];
        let payload_plan = SymbolicPayloadPlan {
            original_len_bytes: 0,
            encoded_len_bytes: 0,
            length_prefix_bytes: 0,
            padding_bits: 0,
            frames: original_frames,
        };
        let options = FixedWidthPlanningOptions {
            prepend_u16_be_length: false,
        };

        let plans = mapper
            .map_payload_to_plans(&payload_plan)
            .expect("mapping frames to plans should succeed");
        let recovered_frames = mapper
            .map_plans_to_frames(&plans)
            .expect("mapping plans back to frames should succeed");
        let schemas = mapper.frame_schemas();
        let ordered_schemas = recovered_frames
            .iter()
            .map(|frame| {
                schemas
                    .iter()
                    .find(|schema| schema.template_id == frame.template_id)
                    .cloned()
                    .expect("schema should exist for recovered frame")
            })
            .collect::<Vec<_>>();

        let payload_from_plans = mapper
            .decode_payload_from_plans(&plans, &options)
            .expect("payload decode from plans should succeed");
        let payload_from_frames =
            decode_payload_from_symbolic_frames(&recovered_frames, &ordered_schemas, &options)
                .expect("payload decode from recovered frames should succeed");

        assert_eq!(payload_from_plans, payload_from_frames);
        assert_eq!(payload_from_plans.len(), 9);
    }

    #[test]
    fn symbolic_mapper_rejects_unknown_surface_during_reverse_mapping() {
        let mapper = FarsiPrototypeSymbolicMapper;
        let plan = RealizationPlan {
            template_id: TemplateId::new("fa-basic-sov").expect("valid template"),
            assignments: vec![
                assign("subject", "ناشناخته"),
                assign("object", "کتاب"),
                assign("adjective", "قدیمی"),
                assign("verb", "نوشت"),
            ],
        };

        let error = mapper
            .map_plans_to_frames(&[plan])
            .expect_err("reverse mapping should fail");

        assert!(error.to_string().contains("unknown surface"));
    }

    fn assign(slot: &str, surface: &str) -> SlotAssignment {
        SlotAssignment {
            slot: SlotId::new(slot).expect("valid slot"),
            surface: surface.to_string(),
            lemma: None,
        }
    }

    fn symbolic_frame(template: &str, fields: &[(&str, u8, u32)]) -> SymbolicFramePlan {
        SymbolicFramePlan {
            template_id: TemplateId::new(template).expect("valid template"),
            source: BitRange {
                start_bit: 0,
                consumed_bits: fields.iter().map(|(_, width, _)| usize::from(*width)).sum(),
            },
            values: fields
                .iter()
                .map(|(slot, bit_width, value)| SymbolicSlotValue {
                    slot: SlotId::new(*slot).expect("valid slot"),
                    bit_width: *bit_width,
                    value: *value,
                })
                .collect(),
        }
    }
}
