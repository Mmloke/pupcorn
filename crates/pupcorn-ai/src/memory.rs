use std::collections::HashMap;

/// Researcher profile built from decisions over time.
#[derive(Debug, Clone, Default)]
pub struct ResearcherProfile {
    /// Preferred tools -> usage count.
    pub tool_usage: HashMap<String, u32>,
    /// Vulnerability kinds focused on.
    pub focus_kinds: HashMap<String, u32>,
    /// Patterns marked false positive -> negative weight.
    pub false_positive_patterns: HashMap<String, f64>,
}

impl ResearcherProfile {
    pub fn record_decision(&mut self, tool: &str, kind: &str, is_false_positive: bool) {
        *self.tool_usage.entry(tool.to_string()).or_insert(0) += 1;
        *self.focus_kinds.entry(kind.to_string()).or_insert(0) += 1;
        if is_false_positive {
            *self
                .false_positive_patterns
                .entry(kind.to_string())
                .or_insert(0.0) -= 1.0;
        }
    }

    /// Weight adjustment for a suggestion pattern (negative if historically FP).
    pub fn weight_for(&self, pattern: &str) -> f64 {
        self.false_positive_patterns.get(pattern).copied().unwrap_or(0.0)
    }
}

/// Feedback weighting helper: apply deltas to a base score.
pub fn apply_feedback(base: f64, deltas: &[f64]) -> f64 {
    deltas.iter().fold(base, |acc, d| acc + d)
}