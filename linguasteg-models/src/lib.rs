pub mod en;
pub mod fa;
pub mod gateway;

use en::parse_english_prototype_text;
use linguasteg_core::{
    CoreError, CoreResult, ModelAdapter, ModelCapability, RealizationPlan, SlotAssignment, SlotId,
    TemplateId, TextExtractor,
};

pub use en::{
    EnglishPrototypeConstraintChecker, EnglishPrototypeLanguagePack, EnglishPrototypeRealizer,
    EnglishPrototypeSymbolicMapper,
};
pub use fa::{
    FarsiPrototypeConstraintChecker, FarsiPrototypeLanguagePack, FarsiPrototypeLexicon,
    FarsiPrototypeRealizer, FarsiPrototypeSymbolicMapper,
};
pub use gateway::{InMemoryGatewayRegistry, StubGateway};

#[derive(Debug, Default)]
pub struct StubModelAdapter;

impl ModelAdapter for StubModelAdapter {
    fn id(&self) -> &str {
        "stub"
    }

    fn supports(&self, _capability: ModelCapability) -> bool {
        false
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct FarsiPrototypeTextExtractor;

impl TextExtractor for FarsiPrototypeTextExtractor {
    fn extract_plans(&self, stego_text: &str) -> CoreResult<Vec<RealizationPlan>> {
        let text = select_farsi_text_body(stego_text);
        let mut plans = Vec::new();

        for raw_sentence in split_sentences(text) {
            let sentence = raw_sentence.trim();
            if sentence.is_empty() || !sentence.contains(" را ") {
                continue;
            }
            plans.push(parse_farsi_sentence_to_plan(sentence)?);
        }

        if plans.is_empty() {
            return Err(CoreError::InvalidTemplate(
                "farsi text extractor could not detect canonical prototype sentences".to_string(),
            ));
        }

        Ok(plans)
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct EnglishPrototypeTextExtractor;

impl TextExtractor for EnglishPrototypeTextExtractor {
    fn extract_plans(&self, stego_text: &str) -> CoreResult<Vec<RealizationPlan>> {
        parse_english_prototype_text(stego_text)
    }
}

fn select_farsi_text_body(input: &str) -> &str {
    let marker = "final prototype text:";
    if let Some(index) = input.find(marker) {
        let start = index + marker.len();
        return input[start..].trim();
    }

    input.trim()
}

fn split_sentences(input: &str) -> impl Iterator<Item = &str> {
    input.split(['.', '۔', '\n'])
}

fn parse_farsi_sentence_to_plan(sentence: &str) -> CoreResult<RealizationPlan> {
    let tokens = sentence.split_whitespace().collect::<Vec<_>>();
    let ra_positions = tokens
        .iter()
        .enumerate()
        .filter_map(|(index, token)| (*token == "را").then_some(index))
        .collect::<Vec<_>>();

    if ra_positions.len() != 1 {
        return Err(CoreError::InvalidTemplate(format!(
            "farsi sentence must contain exactly one 'را' marker: {sentence}"
        )));
    }

    let ra_index = ra_positions[0];
    if tokens.len() == 5 && ra_index == 3 {
        return Ok(RealizationPlan {
            template_id: TemplateId::new("fa-basic-sov")?,
            assignments: vec![
                assignment("subject", tokens[0])?,
                assignment("object", tokens[1])?,
                assignment("adjective", tokens[2])?,
                assignment("verb", tokens[4])?,
            ],
        });
    }

    if tokens.len() == 7 && ra_index == 5 && tokens[2] == "در" {
        return Ok(RealizationPlan {
            template_id: TemplateId::new("fa-time-location-sov")?,
            assignments: vec![
                assignment("subject", tokens[0])?,
                assignment("time", tokens[1])?,
                assignment("location", tokens[3])?,
                assignment("object", tokens[4])?,
                assignment("verb", tokens[6])?,
            ],
        });
    }

    Err(CoreError::InvalidTemplate(format!(
        "unsupported canonical farsi sentence shape: {sentence}"
    )))
}

fn assignment(slot: &str, surface: &str) -> CoreResult<SlotAssignment> {
    Ok(SlotAssignment {
        slot: SlotId::new(slot)?,
        surface: surface.to_string(),
        lemma: None,
    })
}

#[cfg(test)]
mod tests {
    use super::{EnglishPrototypeTextExtractor, FarsiPrototypeTextExtractor};
    use linguasteg_core::TextExtractor;

    #[test]
    fn farsi_extractor_parses_canonical_sentences() {
        let extractor = FarsiPrototypeTextExtractor;
        let text = "مرد کتاب زیبا را نوشت. دانشجو امروز در خانه نامه را خرید.";

        let plans = extractor
            .extract_plans(text)
            .expect("extractor should parse canonical sentences");

        assert_eq!(plans.len(), 2);
        assert_eq!(plans[0].template_id.as_str(), "fa-basic-sov");
        assert_eq!(plans[1].template_id.as_str(), "fa-time-location-sov");
    }

    #[test]
    fn farsi_extractor_reads_final_text_section_from_report() {
        let extractor = FarsiPrototypeTextExtractor;
        let report = "Farsi prototype encode\nframes: 2\n\nfinal prototype text:\nمرد کتاب زیبا را نوشت. دانشجو امروز در خانه نامه را خرید.\ngateway response: stub:encode";

        let plans = extractor
            .extract_plans(report)
            .expect("extractor should parse report final text section");

        assert_eq!(plans.len(), 2);
    }

    #[test]
    fn english_extractor_parses_canonical_sentences() {
        let extractor = EnglishPrototypeTextExtractor;
        let text = "the manager labels clear report. the architect in winter at the office, records manual.";

        let plans = extractor
            .extract_plans(text)
            .expect("extractor should parse canonical sentences");

        assert_eq!(plans.len(), 2);
        assert_eq!(plans[0].template_id.as_str(), "en-basic-svo");
        assert_eq!(plans[1].template_id.as_str(), "en-time-location-svo");
    }

    #[test]
    fn english_extractor_reads_final_text_section_from_report() {
        let extractor = EnglishPrototypeTextExtractor;
        let report = "English prototype encode\nframes: 2\n\nfinal prototype text:\nthe manager labels clear report. the architect in winter at the office, records manual.\ngateway response: stub:encode";

        let plans = extractor
            .extract_plans(report)
            .expect("extractor should parse report final text section");

        assert_eq!(plans.len(), 2);
    }
}
