use linguasteg_core::{
    CoreError, CoreResult, GrammarConstraintChecker, LanguageDescriptor, LanguageRealizer,
    LanguageRegistry, LanguageTag, RealizationPlan, RealizationTemplateDescriptor, SlotAssignment,
    SlotId, SlotRole, StyleInspiration, StyleProfileDescriptor, StyleProfileId,
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

        validate_assignment_surfaces(template, &plan.assignments)
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

#[cfg(test)]
mod tests {
    use linguasteg_core::{
        LanguageRealizer, LanguageRegistry, RealizationPlan, SlotAssignment, SlotId,
        StyleProfileRegistry, TemplateId, TemplateRegistry,
    };

    use super::{FarsiPrototypeConstraintChecker, FarsiPrototypeLanguagePack, FarsiPrototypeRealizer};
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

    fn assign(slot: &str, surface: &str) -> SlotAssignment {
        SlotAssignment {
            slot: SlotId::new(slot).expect("valid slot"),
            surface: surface.to_string(),
            lemma: None,
        }
    }
}
