use crate::{CoreError, CoreResult, RealizationTemplateDescriptor, SlotId, TemplateToken};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SlotAssignment {
    pub slot: SlotId,
    pub surface: String,
    pub lemma: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RealizationPlan {
    pub template_id: crate::TemplateId,
    pub assignments: Vec<SlotAssignment>,
}

pub trait GrammarConstraintChecker: Send + Sync {
    fn validate_plan(
        &self,
        template: &RealizationTemplateDescriptor,
        plan: &RealizationPlan,
    ) -> CoreResult<()>;
}

pub trait LanguageRealizer: Send + Sync {
    fn render(
        &self,
        template: &RealizationTemplateDescriptor,
        plan: &RealizationPlan,
    ) -> CoreResult<String>;
}

pub fn validate_template_descriptor(template: &RealizationTemplateDescriptor) -> CoreResult<()> {
    for token in &template.tokens {
        if let TemplateToken::Slot(slot_id) = token {
            if template.slots.iter().all(|slot| &slot.id != slot_id) {
                return Err(CoreError::InvalidTemplate(format!(
                    "token references undefined slot '{}'",
                    slot_id
                )));
            }
        }
    }

    Ok(())
}

pub fn validate_realization_plan(
    template: &RealizationTemplateDescriptor,
    plan: &RealizationPlan,
) -> CoreResult<()> {
    if plan.template_id != template.id {
        return Err(CoreError::UnsupportedTemplate(plan.template_id.to_string()));
    }

    validate_template_descriptor(template)?;

    let mut seen_slots: Vec<&SlotId> = Vec::new();
    for assignment in &plan.assignments {
        if template.slots.iter().all(|slot| slot.id != assignment.slot) {
            return Err(CoreError::UnknownTemplateSlot(assignment.slot.to_string()));
        }

        if seen_slots.contains(&&assignment.slot) {
            return Err(CoreError::DuplicateSlotAssignment(
                assignment.slot.to_string(),
            ));
        }

        seen_slots.push(&assignment.slot);
    }

    for slot in &template.slots {
        if slot.required && plan.assignments.iter().all(|item| item.slot != slot.id) {
            return Err(CoreError::MissingRequiredSlot(slot.id.to_string()));
        }
    }

    Ok(())
}

pub fn render_realization_plan(
    template: &RealizationTemplateDescriptor,
    plan: &RealizationPlan,
) -> CoreResult<String> {
    validate_realization_plan(template, plan)?;

    let mut rendered_tokens = Vec::with_capacity(template.tokens.len());
    for token in &template.tokens {
        match token {
            TemplateToken::Literal(value) => rendered_tokens.push(value.clone()),
            TemplateToken::Slot(slot_id) => {
                let assignment = plan
                    .assignments
                    .iter()
                    .find(|assignment| &assignment.slot == slot_id)
                    .ok_or_else(|| CoreError::MissingRequiredSlot(slot_id.to_string()))?;
                rendered_tokens.push(assignment.surface.clone());
            }
        }
    }

    Ok(rendered_tokens.join(" "))
}

#[cfg(test)]
mod tests {
    use crate::{
        LanguageTag, RealizationTemplateDescriptor, SlotId, SlotRole, TemplateId,
        TemplateSlotDescriptor, TemplateToken,
    };

    use super::{
        RealizationPlan, SlotAssignment, render_realization_plan, validate_realization_plan,
        validate_template_descriptor,
    };

    fn sample_template() -> RealizationTemplateDescriptor {
        RealizationTemplateDescriptor {
            id: TemplateId::new("fa-basic").expect("valid template id"),
            language: LanguageTag::new("fa").expect("valid language"),
            display_name: "Basic Persian Template".to_string(),
            slots: vec![
                TemplateSlotDescriptor {
                    id: SlotId::new("subject").expect("valid slot"),
                    role: SlotRole::Subject,
                    required: true,
                },
                TemplateSlotDescriptor {
                    id: SlotId::new("object").expect("valid slot"),
                    role: SlotRole::DirectObject,
                    required: true,
                },
                TemplateSlotDescriptor {
                    id: SlotId::new("verb").expect("valid slot"),
                    role: SlotRole::Verb,
                    required: true,
                },
            ],
            tokens: vec![
                TemplateToken::Slot(SlotId::new("subject").expect("valid slot")),
                TemplateToken::Slot(SlotId::new("object").expect("valid slot")),
                TemplateToken::Literal("ra".to_string()),
                TemplateToken::Slot(SlotId::new("verb").expect("valid slot")),
            ],
        }
    }

    #[test]
    fn template_validation_rejects_undefined_slot_reference() {
        let mut template = sample_template();
        template.tokens.push(TemplateToken::Slot(
            SlotId::new("missing").expect("valid slot"),
        ));

        let error = validate_template_descriptor(&template).expect_err("template should fail");
        assert!(error.to_string().contains("undefined slot"));
    }

    #[test]
    fn realization_plan_validation_rejects_missing_required_slot() {
        let template = sample_template();
        let plan = RealizationPlan {
            template_id: TemplateId::new("fa-basic").expect("valid template id"),
            assignments: vec![
                SlotAssignment {
                    slot: SlotId::new("subject").expect("valid slot"),
                    surface: "mard".to_string(),
                    lemma: None,
                },
                SlotAssignment {
                    slot: SlotId::new("verb").expect("valid slot"),
                    surface: "kharid".to_string(),
                    lemma: None,
                },
            ],
        };

        let error = validate_realization_plan(&template, &plan).expect_err("plan should fail");
        assert!(error.to_string().contains("missing required slot"));
    }

    #[test]
    fn render_realization_plan_renders_template_tokens_in_order() {
        let template = sample_template();
        let plan = RealizationPlan {
            template_id: TemplateId::new("fa-basic").expect("valid template id"),
            assignments: vec![
                SlotAssignment {
                    slot: SlotId::new("subject").expect("valid slot"),
                    surface: "mard".to_string(),
                    lemma: Some("mard".to_string()),
                },
                SlotAssignment {
                    slot: SlotId::new("object").expect("valid slot"),
                    surface: "ketab".to_string(),
                    lemma: Some("ketab".to_string()),
                },
                SlotAssignment {
                    slot: SlotId::new("verb").expect("valid slot"),
                    surface: "kharid".to_string(),
                    lemma: Some("kharid".to_string()),
                },
            ],
        };

        let rendered = render_realization_plan(&template, &plan).expect("render should work");
        assert_eq!(rendered, "mard ketab ra kharid");
    }
}
