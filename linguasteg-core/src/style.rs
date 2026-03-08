use crate::{CoreResult, LanguageTag, StyleProfileDescriptor, StyleProfileId, StyleStrength};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StyleSelection {
    pub profile_id: StyleProfileId,
    pub strength_override: Option<StyleStrength>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StyleCandidate {
    pub text: String,
    pub quality_score: Option<f32>,
    pub style_score: Option<f32>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StyleRerankRequest {
    pub language: LanguageTag,
    pub profile: Option<StyleSelection>,
    pub candidates: Vec<String>,
}

pub trait StyleScorer: Send + Sync {
    fn score_candidate(
        &self,
        language: &LanguageTag,
        profile: &StyleProfileDescriptor,
        candidate: &str,
    ) -> CoreResult<f32>;
}

pub trait StyleReranker: Send + Sync {
    fn rerank_candidates(&self, request: &StyleRerankRequest) -> CoreResult<Vec<StyleCandidate>>;
}
