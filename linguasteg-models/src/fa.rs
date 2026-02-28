use linguasteg_core::{
    CoreError, CoreResult, GrammarConstraintChecker, LanguageDescriptor, LanguageRealizer,
    LanguageRegistry, LanguageTag, RealizationPlan, RealizationTemplateDescriptor, SlotAssignment,
    SlotId, SlotRole, StyleInspiration, StyleProfileDescriptor, StyleProfileId,
    SymbolicFieldSpec, SymbolicFramePlan, SymbolicFrameSchema, SymbolicPayloadPlan,
    StyleProfileRegistry, StyleStrength, TemplateId, TemplateRegistry, TemplateSlotDescriptor,
    TemplateToken, TextDirection, WritingRegister, render_realization_plan, validate_realization_plan,
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
            return Err(CoreError::UnsupportedLanguage(template.language.to_string()));
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

impl FarsiPrototypeSymbolicMapper {
    pub fn frame_schemas(&self) -> Vec<SymbolicFrameSchema> {
        farsi_symbolic_frame_schemas()
    }

    pub fn map_payload_to_plans(
        &self,
        payload_plan: &SymbolicPayloadPlan,
    ) -> CoreResult<Vec<RealizationPlan>> {
        payload_plan
            .frames
            .iter()
            .map(|frame| self.map_frame_to_plan(frame))
            .collect()
    }

    pub fn map_frame_to_plan(&self, frame: &SymbolicFramePlan) -> CoreResult<RealizationPlan> {
        match frame.template_id.as_str() {
            "fa-basic-sov" => map_basic_sov_frame(frame),
            "fa-time-location-sov" => map_time_location_sov_frame(frame),
            _ => Err(CoreError::UnsupportedTemplate(frame.template_id.to_string())),
        }
    }
}

fn map_basic_sov_frame(frame: &SymbolicFramePlan) -> CoreResult<RealizationPlan> {
    let subject_value = symbolic_value_for_slot(frame, "subject")?;
    let object_value = symbolic_value_for_slot(frame, "object")?;
    let adjective_value = symbolic_value_for_slot(frame, "adjective")?;
    let verb_value = symbolic_value_for_slot(frame, "verb")?;

    let subject_surface = select_surface(subject_forms(), subject_value)?;
    let object_lexeme = select_noun_lexeme(object_value)?;
    let adjective_lexeme = select_compatible_adjective_lexeme(object_lexeme, adjective_value)?;
    let verb_lexeme = select_compatible_verb_lexeme(object_lexeme, verb_value)?;

    Ok(RealizationPlan {
        template_id: TemplateId::new("fa-basic-sov")?,
        assignments: vec![
            create_assignment("subject", subject_surface, None)?,
            create_assignment("object", object_lexeme.canonical.to_string(), Some(object_lexeme.canonical))?,
            create_assignment(
                "adjective",
                adjective_lexeme.canonical.to_string(),
                Some(adjective_lexeme.canonical),
            )?,
            create_assignment("verb", verb_lexeme.canonical.to_string(), Some(verb_lexeme.canonical))?,
        ],
    })
}

fn map_time_location_sov_frame(frame: &SymbolicFramePlan) -> CoreResult<RealizationPlan> {
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

    Ok(RealizationPlan {
        template_id: TemplateId::new("fa-time-location-sov")?,
        assignments: vec![
            create_assignment("subject", subject_surface, None)?,
            create_assignment("time", time_surface, None)?,
            create_assignment("location", location_surface, None)?,
            create_assignment("object", object_lexeme.canonical.to_string(), Some(object_lexeme.canonical))?,
            create_assignment("verb", verb_lexeme.canonical.to_string(), Some(verb_lexeme.canonical))?,
        ],
    })
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

        if let Some(slot) = template.slots.iter().find(|item| item.id == assignment.slot) {
            if matches!(slot.role, SlotRole::Verb) && !looks_like_farsi_or_translit(trimmed_surface) {
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
            let adjective =
                find_adjective_lexeme(assignment_key(adjective_assignment)).ok_or_else(|| {
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
        .and_then(|slot| assignments.iter().find(|assignment| assignment.slot == slot.id))
}

fn assignment_key(assignment: &SlotAssignment) -> &str {
    assignment.lemma.as_deref().unwrap_or(&assignment.surface).trim()
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
            id: StyleProfileId::new("fa-literary-classic-inspired")
                .expect("valid style id"),
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
        semantic_tags: &["document", "readable", "physical-object"],
    },
    FarsiNounLexeme {
        canonical: "نامه",
        accepted_forms: &["نامه", "nameh"],
        semantic_tags: &["document", "message", "physical-object"],
    },
    FarsiNounLexeme {
        canonical: "چای",
        accepted_forms: &["چای", "chay", "tea"],
        semantic_tags: &["drink", "food"],
    },
    FarsiNounLexeme {
        canonical: "غذا",
        accepted_forms: &["غذا", "ghaza", "food"],
        semantic_tags: &["food"],
    },
    FarsiNounLexeme {
        canonical: "گل",
        accepted_forms: &["گل", "gol", "flower"],
        semantic_tags: &["plant", "gift", "decorative"],
    },
    FarsiNounLexeme {
        canonical: "عکس",
        accepted_forms: &["عکس", "aks"],
        semantic_tags: &["image", "document", "physical-object"],
    },
];

const FARSI_VERB_LEXEMES: &[FarsiVerbLexeme] = &[
    FarsiVerbLexeme {
        canonical: "خرید",
        accepted_forms: &["خرید", "kharid", "bought"],
        accepted_object_tags: &["document", "food", "gift", "decorative", "physical-object"],
    },
    FarsiVerbLexeme {
        canonical: "نوشت",
        accepted_forms: &["نوشت", "nevesht", "wrote"],
        accepted_object_tags: &["document", "message"],
    },
    FarsiVerbLexeme {
        canonical: "دید",
        accepted_forms: &["دید", "did", "saw"],
        accepted_object_tags: &[
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
        accepted_forms: &["خورد", "khord", "ate"],
        accepted_object_tags: &["food"],
    },
    FarsiVerbLexeme {
        canonical: "نوشید",
        accepted_forms: &["نوشید", "noushid", "drank"],
        accepted_object_tags: &["drink"],
    },
];

const FARSI_ADJECTIVE_LEXEMES: &[FarsiAdjectiveLexeme] = &[
    FarsiAdjectiveLexeme {
        canonical: "زیبا",
        accepted_forms: &["زیبا", "ziba"],
        accepted_noun_tags: &[
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
        accepted_forms: &["قدیمی", "ghadimi"],
        accepted_noun_tags: &["document", "image", "physical-object"],
    },
    FarsiAdjectiveLexeme {
        canonical: "تازه",
        accepted_forms: &["تازه", "taze"],
        accepted_noun_tags: &["food", "drink", "plant"],
    },
    FarsiAdjectiveLexeme {
        canonical: "گرم",
        accepted_forms: &["گرم", "garm"],
        accepted_noun_tags: &["food", "drink"],
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
        SymbolicFramePlan, SymbolicPayloadPlanner, SymbolicSlotValue, TemplateId, TemplateRegistry,
    };

    use super::{
        FarsiPrototypeConstraintChecker, FarsiPrototypeLanguagePack, FarsiPrototypeLexicon,
        FarsiPrototypeRealizer, FarsiPrototypeSymbolicMapper,
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
    fn constraint_checker_rejects_incompatible_object_verb_combination() {
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
                assign("verb", "نوشید"),
            ],
        };

        let error = FarsiPrototypeConstraintChecker
            .validate_plan(template, &plan)
            .expect_err("plan should fail");
        assert!(error.to_string().contains("not compatible"));
    }

    #[test]
    fn constraint_checker_rejects_incompatible_object_adjective_combination() {
        let pack = FarsiPrototypeLanguagePack::default();
        let template = pack
            .template(&TemplateId::new("fa-basic-sov").expect("valid template"))
            .expect("template should exist");

        let plan = RealizationPlan {
            template_id: TemplateId::new("fa-basic-sov").expect("valid template"),
            assignments: vec![
                assign("subject", "زن"),
                assign("object", "کتاب"),
                assign("adjective", "گرم"),
                assign("verb", "دید"),
            ],
        };

        let error = FarsiPrototypeConstraintChecker
            .validate_plan(template, &plan)
            .expect_err("plan should fail");
        assert!(error.to_string().contains("adjective"));
    }

    #[test]
    fn lexicon_recognizes_known_forms() {
        assert!(FarsiPrototypeLexicon::is_known_object_noun("کتاب"));
        assert!(FarsiPrototypeLexicon::is_known_verb("نوشید"));
        assert!(FarsiPrototypeLexicon::is_known_adjective("گرم"));
        assert!(!FarsiPrototypeLexicon::is_known_object_noun("ابر"));
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
