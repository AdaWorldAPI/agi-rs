//! Goal Engine — Autonomous Goal Generation
//!
//! Goals emerge from qualia imbalance, not external commands.
//! Uses diagonal lack detection across qualia dimensions.

use serde::{Serialize, Deserialize};
use super::maslow::{NeedLevel, MaslowEngine};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GoalOrigin {
    QualiaImbalance, Stagnation, Curiosity, DreamProbe, NeedFrustration, ExternalSuggestion,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EdgeType { Requires, Conflicts, Supports, SubgoalOf }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutonomousGoal {
    pub id: u64,
    pub description: String,
    pub origin: GoalOrigin,
    pub need_level: NeedLevel,
    pub target_qualia: Vec<(usize, f32)>,
    pub priority: f32,
    pub novelty: f32,
    pub urgency: f32,
    pub active: bool,
    pub created_at: u64,
    pub progress: f32,
}

impl AutonomousGoal {
    pub fn new(id: u64, description: impl Into<String>, origin: GoalOrigin) -> Self {
        Self {
            id, description: description.into(), origin,
            need_level: NeedLevel::Cognitive, target_qualia: Vec::new(),
            priority: 0.5, novelty: 0.5, urgency: 0.0,
            active: true, created_at: 0, progress: 0.0,
        }
    }
    
    pub fn with_target(mut self, dim: usize, value: f32) -> Self {
        self.target_qualia.push((dim, value.clamp(0.0, 1.0))); self
    }
    
    pub fn with_priority(mut self, priority: f32) -> Self {
        self.priority = priority.clamp(0.0, 1.0); self
    }
    
    pub fn with_novelty(mut self, novelty: f32) -> Self {
        self.novelty = novelty.clamp(0.0, 1.0); self
    }
    
    pub fn effective_priority(&self) -> f32 {
        (self.priority + self.novelty * 0.2 + self.urgency * 0.3).clamp(0.0, 1.0)
    }
    
    pub fn is_achieved(&self, current: &[f32], threshold: f32) -> bool {
        for (dim, target) in &self.target_qualia {
            if *dim < current.len() && (current[*dim] - target).abs() > threshold {
                return false;
            }
        }
        true
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DreamProbe {
    pub exploration_target: String,
    pub probe_dimensions: Vec<usize>,
    pub intensity: f32,
    pub active: bool,
    pub observations: Vec<(usize, f32)>,
}

impl DreamProbe {
    pub fn new(target: impl Into<String>) -> Self {
        Self {
            exploration_target: target.into(),
            probe_dimensions: Vec::new(),
            intensity: 0.3, active: true, observations: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NoveltyEngine {
    seen_states: Vec<(u64, u32)>,
    max_states: usize,
    decay: f32,
}

impl NoveltyEngine {
    pub fn new(max_states: usize) -> Self {
        Self { seen_states: Vec::new(), max_states, decay: 0.99 }
    }
    
    fn hash_state(qualia: &[f32]) -> u64 {
        let mut hash = 0u64;
        for (i, &q) in qualia.iter().enumerate() {
            let quantized = (q * 10.0) as u64;
            hash ^= quantized.wrapping_mul(0x517cc1b727220a95u64.wrapping_add(i as u64));
        }
        hash
    }
    
    pub fn novelty(&self, qualia: &[f32]) -> f32 {
        let hash = Self::hash_state(qualia);
        if let Some((_, count)) = self.seen_states.iter().find(|(h, _)| *h == hash) {
            (1.0 - (*count as f32).ln() / 10.0).clamp(0.0, 1.0)
        } else { 1.0 }
    }
    
    pub fn observe(&mut self, qualia: &[f32]) {
        let hash = Self::hash_state(qualia);
        if let Some((_, count)) = self.seen_states.iter_mut().find(|(h, _)| *h == hash) {
            *count = count.saturating_add(1);
        } else {
            if self.seen_states.len() >= self.max_states {
                if let Some(min_idx) = self.seen_states.iter().enumerate()
                    .min_by_key(|(_, (_, c))| c).map(|(i, _)| i) {
                    self.seen_states.remove(min_idx);
                }
            }
            self.seen_states.push((hash, 1));
        }
    }
    
    pub fn decay_all(&mut self) {
        for (_, count) in &mut self.seen_states {
            *count = (*count as f32 * self.decay) as u32;
        }
        self.seen_states.retain(|(_, c)| *c > 0);
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoalGenerator {
    goals: Vec<AutonomousGoal>,
    novelty_engine: NoveltyEngine,
    next_id: u64,
    diagonal_threshold: f32,
    max_goals: usize,
    tick: u64,
}

impl Default for GoalGenerator {
    fn default() -> Self { Self::new() }
}

impl GoalGenerator {
    pub fn new() -> Self {
        Self {
            goals: Vec::new(),
            novelty_engine: NoveltyEngine::new(1000),
            next_id: 1, diagonal_threshold: 0.3, max_goals: 10, tick: 0,
        }
    }
    
    pub fn generate(&mut self, qualia: &[f32], maslow: &MaslowEngine) -> Vec<AutonomousGoal> {
        self.tick += 1;
        self.novelty_engine.observe(qualia);
        let mut new_goals = Vec::new();
        
        // Diagonal lack detection
        let lacks = self.detect_diagonal_lack(qualia);
        for (dim, lack) in lacks {
            if lack > self.diagonal_threshold {
                let goal = self.create_goal_from_lack(dim, lack, qualia);
                if maslow.can_pursue(goal.need_level) && self.goals.len() < self.max_goals {
                    self.goals.push(goal.clone());
                    new_goals.push(goal);
                }
            }
        }
        new_goals
    }
    
    fn detect_diagonal_lack(&self, qualia: &[f32]) -> Vec<(usize, f32)> {
        if qualia.len() < 2 { return Vec::new(); }
        let mean: f32 = qualia.iter().sum::<f32>() / qualia.len() as f32;
        qualia.iter().enumerate()
            .map(|(i, &q)| (i, (q - mean).abs()))
            .filter(|(_, lack)| *lack > 0.1)
            .collect()
    }
    
    fn create_goal_from_lack(&mut self, dim: usize, lack: f32, current: &[f32]) -> AutonomousGoal {
        let id = self.next_id; self.next_id += 1;
        let mean: f32 = current.iter().sum::<f32>() / current.len() as f32;
        let current_val = current.get(dim).copied().unwrap_or(0.5);
        let target = (current_val + mean) / 2.0;
        let novelty = self.novelty_engine.novelty(current);
        AutonomousGoal::new(id, format!("Balance dimension {}", dim), GoalOrigin::QualiaImbalance)
            .with_target(dim, target).with_priority(lack).with_novelty(novelty)
    }
    
    pub fn goals(&self) -> &[AutonomousGoal] { &self.goals }
    
    pub fn top_goal(&self) -> Option<&AutonomousGoal> {
        self.goals.iter().filter(|g| g.active)
            .max_by(|a, b| a.effective_priority().partial_cmp(&b.effective_priority()).unwrap())
    }
    
    pub fn decay(&mut self) { self.novelty_engine.decay_all(); }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_goal_generation() {
        let mut generator = GoalGenerator::new();
        let maslow = MaslowEngine::new();
        let qualia = vec![0.9, 0.1, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5];
        generator.generate(&qualia, &maslow);
        // Should generate goals from imbalance
    }
}
