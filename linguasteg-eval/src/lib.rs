#[derive(Debug, Clone, Default, PartialEq)]
pub struct EvaluationSummary {
    pub semantic_similarity: f32,
    pub payload_accuracy: f32,
    pub detectability_score: f32,
}
