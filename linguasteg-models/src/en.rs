use linguasteg_core::{
    CoreError, CoreResult, GrammarConstraintChecker, LanguageDescriptor, LanguageRealizer,
    LanguageRegistry, LanguageTag, RealizationPlan, RealizationTemplateDescriptor, SlotId,
    SlotRole, StyleInspiration, StyleProfileDescriptor, StyleProfileId, StyleProfileRegistry,
    StyleStrength, SymbolicFieldSpec, SymbolicFramePlan, SymbolicFrameSchema, SymbolicPayloadPlan,
    TemplateId, TemplateRegistry, TemplateSlotDescriptor, TemplateToken,
    TextDirection, WritingRegister, render_realization_plan, validate_realization_plan,
};

#[derive(Debug, Clone)]
pub struct EnglishPrototypeLanguagePack {
    languages: Vec<LanguageDescriptor>,
    style_profiles: Vec<StyleProfileDescriptor>,
    templates: Vec<RealizationTemplateDescriptor>,
}

impl Default for EnglishPrototypeLanguagePack {
    fn default() -> Self {
        Self {
            languages: english_languages(),
            style_profiles: english_style_profiles(),
            templates: english_templates(),
        }
    }
}

impl LanguageRegistry for EnglishPrototypeLanguagePack {
    fn all_languages(&self) -> &[LanguageDescriptor] {
        &self.languages
    }
}

impl StyleProfileRegistry for EnglishPrototypeLanguagePack {
    fn all_style_profiles(&self) -> &[StyleProfileDescriptor] {
        &self.style_profiles
    }
}

impl TemplateRegistry for EnglishPrototypeLanguagePack {
    fn all_templates(&self) -> &[RealizationTemplateDescriptor] {
        &self.templates
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct EnglishPrototypeConstraintChecker;

impl GrammarConstraintChecker for EnglishPrototypeConstraintChecker {
    fn validate_plan(
        &self,
        template: &RealizationTemplateDescriptor,
        plan: &RealizationPlan,
    ) -> CoreResult<()> {
        validate_realization_plan(template, plan)?;

        if template.language.as_str() != "en" {
            return Err(CoreError::UnsupportedLanguage(
                template.language.to_string(),
            ));
        }

        for assignment in &plan.assignments {
            if assignment.surface.trim().is_empty() {
                return Err(CoreError::InvalidTemplate(format!(
                    "slot '{}' cannot be empty in english realization plan",
                    assignment.slot
                )));
            }
        }

        Ok(())
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct EnglishPrototypeRealizer;

impl LanguageRealizer for EnglishPrototypeRealizer {
    fn render(
        &self,
        template: &RealizationTemplateDescriptor,
        plan: &RealizationPlan,
    ) -> CoreResult<String> {
        EnglishPrototypeConstraintChecker.validate_plan(template, plan)?;
        let rendered = render_realization_plan(template, plan)?;
        Ok(normalize_english_spacing(&rendered))
    }
}

fn normalize_english_spacing(input: &str) -> String {
    let collapsed = input.split_whitespace().collect::<Vec<_>>().join(" ");
    collapsed.replace(" ,", ",")
}

#[derive(Debug, Default, Clone, Copy)]
pub struct EnglishPrototypeSymbolicMapper;

impl EnglishPrototypeSymbolicMapper {
    pub fn frame_schemas(&self) -> Vec<SymbolicFrameSchema> {
        english_symbolic_frame_schemas()
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
            "en-basic-svo" => map_basic_svo_frame(frame),
            "en-time-location-svo" => map_time_location_svo_frame(frame),
            _ => Err(CoreError::UnsupportedTemplate(
                frame.template_id.to_string(),
            )),
        }
    }
}

fn map_basic_svo_frame(frame: &SymbolicFramePlan) -> CoreResult<RealizationPlan> {
    let subject = select_form(subject_forms(), symbolic_value_for_slot(frame, "subject")?);
    let object = select_form(object_forms(), symbolic_value_for_slot(frame, "object")?);
    let adjective = select_form(
        adjective_forms(),
        symbolic_value_for_slot(frame, "adjective")?,
    );
    let verb = select_form(verb_forms(), symbolic_value_for_slot(frame, "verb")?);

    Ok(RealizationPlan {
        template_id: TemplateId::new("en-basic-svo")?,
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
        template_id: TemplateId::new("en-time-location-svo")?,
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

fn english_languages() -> Vec<LanguageDescriptor> {
    vec![LanguageDescriptor {
        tag: LanguageTag::new("en").expect("valid language tag"),
        display_name: "English".to_string(),
        direction: TextDirection::LeftToRight,
    }]
}

fn english_style_profiles() -> Vec<StyleProfileDescriptor> {
    vec![StyleProfileDescriptor {
        id: StyleProfileId::new("en-neutral-prototype").expect("valid style profile id"),
        language: LanguageTag::new("en").expect("valid language tag"),
        display_name: "Neutral English Prototype".to_string(),
        register: WritingRegister::Neutral,
        strength: StyleStrength::Light,
        inspiration: StyleInspiration::RegisterOnly,
    }]
}

fn english_templates() -> Vec<RealizationTemplateDescriptor> {
    vec![
        RealizationTemplateDescriptor {
            id: TemplateId::new("en-basic-svo").expect("valid template id"),
            language: LanguageTag::new("en").expect("valid language tag"),
            display_name: "English Basic SVO".to_string(),
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
            id: TemplateId::new("en-time-location-svo").expect("valid template id"),
            language: LanguageTag::new("en").expect("valid language tag"),
            display_name: "English Time Location SVO".to_string(),
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

fn english_symbolic_frame_schemas() -> Vec<SymbolicFrameSchema> {
    vec![
        SymbolicFrameSchema {
            template_id: TemplateId::new("en-basic-svo").expect("valid template id"),
            fields: vec![
                symbolic_field("subject", 5),
                symbolic_field("object", 5),
                symbolic_field("adjective", 3),
                symbolic_field("verb", 5),
            ],
        },
        SymbolicFrameSchema {
            template_id: TemplateId::new("en-time-location-svo").expect("valid template id"),
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
    ]
}

fn object_forms() -> &'static [&'static str] {
    &[
        "book", "letter", "photo", "tea", "food", "flower", "note", "report",
    ]
}

fn adjective_forms() -> &'static [&'static str] {
    &[
        "old", "new", "quiet", "bright", "warm", "fresh", "small", "clear",
    ]
}

fn verb_forms() -> &'static [&'static str] {
    &[
        "buys", "writes", "reads", "sees", "keeps", "builds", "finds", "moves",
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
        GrammarConstraintChecker, LanguageRealizer, LanguageTag, RealizationPlan, SlotAssignment,
        SlotId, StyleProfileRegistry, TemplateId, TemplateRegistry,
    };

    use super::{
        EnglishPrototypeConstraintChecker, EnglishPrototypeLanguagePack, EnglishPrototypeRealizer,
        EnglishPrototypeSymbolicMapper,
    };

    #[test]
    fn english_pack_exposes_templates_and_style_profiles() {
        let pack = EnglishPrototypeLanguagePack::default();
        let en = LanguageTag::new("en").expect("valid language");

        assert_eq!(pack.templates_for_language(&en).len(), 2);
        assert_eq!(pack.style_profiles_for_language(&en).len(), 1);
    }

    #[test]
    fn english_mapper_maps_frame_to_plan() {
        let mapper = EnglishPrototypeSymbolicMapper;
        let schemas = mapper.frame_schemas();
        let frame = linguasteg_core::SymbolicFramePlan {
            template_id: TemplateId::new("en-basic-svo").expect("valid template"),
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
        assert_eq!(plan.template_id.as_str(), "en-basic-svo");
        assert_eq!(plan.assignments.len(), schemas[0].fields.len());
    }

    #[test]
    fn english_checker_rejects_empty_surface() {
        let pack = EnglishPrototypeLanguagePack::default();
        let checker = EnglishPrototypeConstraintChecker;
        let template = pack
            .template(&TemplateId::new("en-basic-svo").expect("valid template"))
            .expect("template should exist");
        let plan = RealizationPlan {
            template_id: TemplateId::new("en-basic-svo").expect("valid template"),
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
    fn english_realizer_renders_sentence() {
        let pack = EnglishPrototypeLanguagePack::default();
        let realizer = EnglishPrototypeRealizer;
        let template = pack
            .template(&TemplateId::new("en-basic-svo").expect("valid template"))
            .expect("template should exist");
        let plan = RealizationPlan {
            template_id: TemplateId::new("en-basic-svo").expect("valid template"),
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
}
