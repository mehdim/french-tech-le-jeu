//! Veridix the product: detection quality, drift, tech debt, inference cost.
//!
//! Universal AI-text detection is *not* a solvable problem here: accuracy
//! gains shrink near a ceiling, benchmarks drift as new foundation models
//! ship, and false positives on human text keep customers nervous.

use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ProductState {
    /// Task ids shipped at least once.
    pub shipped: BTreeSet<String>,
    /// Breadth/polish of the product, 0..100.
    pub feature_score: f64,
    /// Model accuracy on the reference benchmark before drift, in %.
    pub model_accuracy: f64,
    /// Benchmark degradation accumulated since the last retraining.
    pub drift: f64,
    /// Robustness to drift and adversarial paraphrasing (0..0.8).
    pub robustness: f64,
    /// Base false-positive rate on human-written text, in %.
    pub fpr: f64,
    pub tech_debt: f64,
    /// € per 1,000 analyses (before temporary modifiers).
    pub inference_cost_per_k: f64,
    /// Multiplier on per-customer usage.
    pub usage_mult: f64,
    /// Supported languages (1 = French + English).
    pub languages: u32,
    /// Reliability hit from recent incidents (decays).
    pub incident_penalty: f64,
    pub launched: bool,
    /// Relative retention bonuses per segment (0.1 = -10 % churn).
    pub retention: [f64; 3],
    /// Relative conversion bonuses per segment.
    pub conversion: [f64; 3],
    /// Extra monthly hosting from architecture decisions.
    pub hosting_extra: f64,
    /// Incident probability reduction from monitoring, audits, SRE work.
    #[serde(default)]
    pub resilience: f64,
    /// Accumulated organic usage growth (capped).
    #[serde(default)]
    pub usage_creep: f64,
}

impl ProductState {
    pub fn new(accuracy: f64, fpr: f64, debt: f64, inference_cost: f64) -> Self {
        Self {
            shipped: BTreeSet::new(),
            feature_score: 0.0,
            model_accuracy: accuracy,
            drift: 0.0,
            robustness: 0.0,
            fpr,
            tech_debt: debt,
            inference_cost_per_k: inference_cost,
            usage_mult: 1.0,
            languages: 1,
            incident_penalty: 0.0,
            launched: false,
            retention: [0.0; 3],
            conversion: [0.0; 3],
            hosting_extra: 0.0,
            resilience: 0.0,
            usage_creep: 0.0,
        }
    }

    /// Accuracy measured on today's traffic (model accuracy minus drift).
    pub fn effective_accuracy(&self) -> f64 {
        (self.model_accuracy - self.drift).clamp(50.0, 99.0)
    }

    /// False positives grow as the benchmark drifts.
    pub fn effective_fpr(&self) -> f64 {
        (self.fpr + self.drift * 0.15).clamp(0.3, 40.0)
    }

    /// Detection score 0..100: accuracy above coin-flip minus a penalty for
    /// flagging humans (the thing customers hate most).
    pub fn detection_score(&self) -> f64 {
        ((self.effective_accuracy() - 50.0) * 3.0 - self.effective_fpr() * 1.2).clamp(0.0, 100.0)
    }

    pub fn reliability(&self) -> f64 {
        (100.0 - self.tech_debt * 0.6 - self.incident_penalty).clamp(0.0, 100.0)
    }

    /// Market-facing product quality, 0..100.
    /// quality = 35 % features + 45 % detection + 20 % reliability
    pub fn quality(&self) -> f64 {
        (0.35 * self.feature_score.min(100.0)
            + 0.45 * self.detection_score()
            + 0.20 * self.reliability())
        .clamp(0.0, 100.0)
    }

    /// Apply an accuracy gain with diminishing returns: the closer to the
    /// ceiling, the smaller the improvement.
    pub fn gain_accuracy(&mut self, amount: f64, ceiling: f64) {
        let gap = (ceiling - self.model_accuracy).max(0.0);
        let scale = (gap / (ceiling - 53.0)).clamp(0.0, 1.0).sqrt();
        self.model_accuracy = (self.model_accuracy + amount * scale.max(0.05)).min(ceiling);
    }

    pub fn has(&self, task: &str) -> bool {
        self.shipped.contains(task)
    }
}
