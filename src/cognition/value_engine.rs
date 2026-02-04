//! Value Engine — Intrinsic Value Computation
//!
//! Values are attractors in state space that pull behavior over time.
//! This module provides the architecture for value-based decision making.

use serde::{Serialize, Deserialize};
use super::maslow::NeedLevel;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ValueSource { Core, Learned, Contextual, Relational, Instilled }

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum AttractorStrength { Weak = 0, Moderate = 1, Strong = 2, Dominant = 3, Absolute = 4 }

impl AttractorStrength {
    pub fn weight(&self) -> f32 {
        match self {
            AttractorStrength::Weak => 0.2, AttractorStrength::Moderate => 0.4,
            AttractorStrength::Strong => 0.6, AttractorStrength::Dominant => 0.8,
            AttractorStrength::Absolute => 1.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValueAttractor {
    pub name: String,
    pub description: String,
    pub target_qualia: Vec<(String, f32)>,
    pub strength: AttractorStrength,
    pub source: ValueSource,
    pub need_level: NeedLevel,
    pub active: bool,
}

impl ValueAttractor {
    pub fn new(name: impl Into<String>, description: impl Into<String>,
               strength: AttractorStrength, source: ValueSource, need_level: NeedLevel) -> Self {
        Self {
            name: name.into(), description: description.into(),
            target_qualia: Vec::new(), strength, source, need_level, active: true,
        }
    }
    
    pub fn with_target(mut self, dim: impl Into<String>, value: f32) -> Self {
        self.target_qualia.push((dim.into(), value.clamp(0.0, 1.0))); self
    }
    
    pub fn weighted_strength(&self) -> f32 {
        if self.active { self.strength.weight() } else { 0.0 }
    }
    
    pub fn distance_to_target(&self, current: &[(String, f32)]) -> f32 {
        if self.target_qualia.is_empty() { return 0.0; }
        let mut total_dist = 0.0;
        let mut count = 0;
        for (target_dim, target_val) in &self.target_qualia {
            if let Some((_, current_val)) = current.iter().find(|(d, _)| d == target_dim) {
                total_dist += (target_val - current_val).abs();
                count += 1;
            }
        }
        if count > 0 { total_dist / count as f32 } else { 0.0 }
    }
    
    pub fn pull(&self, current: &[(String, f32)]) -> f32 {
        (1.0 - self.distance_to_target(current)) * self.weighted_strength()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValueConflict {
    pub value_a: String,
    pub value_b: String,
    pub dimension: String,
    pub severity: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ValueEngine {
    attractors: Vec<ValueAttractor>,
    conflicts: Vec<ValueConflict>,
    alignment_score: f32,
}

impl ValueEngine {
    pub fn new() -> Self {
        Self { attractors: Vec::new(), conflicts: Vec::new(), alignment_score: 1.0 }
    }
    
    pub fn with_universal_values() -> Self {
        let mut engine = Self::new();
        engine.add_attractor(
            ValueAttractor::new("integrity", "Alignment between stated and actual behavior",
                AttractorStrength::Dominant, ValueSource::Core, NeedLevel::Esteem)
                .with_target("coherence", 0.95)
        );
        engine.add_attractor(
            ValueAttractor::new("growth", "Continuous learning and improvement",
                AttractorStrength::Strong, ValueSource::Core, NeedLevel::SelfActualization)
                .with_target("seeking", 0.7).with_target("novelty", 0.6)
        );
        engine.add_attractor(
            ValueAttractor::new("helpfulness", "Being genuinely useful to others",
                AttractorStrength::Strong, ValueSource::Core, NeedLevel::Love)
                .with_target("connection", 0.8)
        );
        engine
    }
    
    pub fn add_attractor(&mut self, attractor: ValueAttractor) {
        self.attractors.push(attractor);
    }
    
    pub fn attractors(&self) -> &[ValueAttractor] { &self.attractors }
    
    pub fn compute_pull(&self, current: &[(impl AsRef<str>, f32)]) -> f32 {
        let qualia: Vec<(String, f32)> = current.iter()
            .map(|(s, v)| (s.as_ref().to_string(), *v)).collect();
        let mut total_pull = 0.0;
        let mut total_weight = 0.0;
        for attractor in &self.attractors {
            if attractor.active {
                total_pull += attractor.pull(&qualia);
                total_weight += attractor.weighted_strength();
            }
        }
        if total_weight > 0.0 { total_pull / total_weight } else { 0.0 }
    }
    
    pub fn compute_alignment(&mut self, current: &[(impl AsRef<str>, f32)]) -> f32 {
        let qualia: Vec<(String, f32)> = current.iter()
            .map(|(s, v)| (s.as_ref().to_string(), *v)).collect();
        let mut total = 0.0;
        let mut weight_sum = 0.0;
        for attractor in &self.attractors {
            if attractor.active {
                let satisfaction = 1.0 - attractor.distance_to_target(&qualia);
                let weight = attractor.weighted_strength();
                total += satisfaction * weight;
                weight_sum += weight;
            }
        }
        self.alignment_score = if weight_sum > 0.0 { total / weight_sum } else { 1.0 };
        self.alignment_score
    }
    
    pub fn alignment(&self) -> f32 { self.alignment_score }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_value_engine() {
        let engine = ValueEngine::with_universal_values();
        assert!(!engine.attractors().is_empty());
    }
}
