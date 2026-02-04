//! Volition Engine — Agency and Will Synthesis
//!
//! Synthesizes volition (will) from goals, values, and constraints.
//! This is the "I choose" layer that converts desires into intentions.

use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VolitionType { Pursue, Avoid, Maintain, Explore, Release, Wait }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConstraintType { Ethical, Physical, Resource, Social, Temporal, Relational }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WillConstraint {
    pub constraint_type: ConstraintType,
    pub description: String,
    pub strength: f32,
    pub active: bool,
}

impl WillConstraint {
    pub fn new(constraint_type: ConstraintType, description: impl Into<String>, strength: f32) -> Self {
        Self { constraint_type, description: description.into(), strength: strength.clamp(0.0, 1.0), active: true }
    }
    
    pub fn ethical(description: impl Into<String>) -> Self {
        Self::new(ConstraintType::Ethical, description, 1.0)
    }
    
    pub fn blocks(&self, action_strength: f32) -> bool {
        self.active && self.strength > action_strength
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Volition {
    pub id: u64,
    pub volition_type: VolitionType,
    pub target: String,
    pub strength: f32,
    pub clarity: f32,
    pub value_alignment: f32,
    pub constrained: bool,
    pub active_constraints: Vec<String>,
    pub source_goal: Option<u64>,
    pub created_at: u64,
}

impl Volition {
    pub fn new(id: u64, volition_type: VolitionType, target: impl Into<String>) -> Self {
        Self {
            id, volition_type, target: target.into(),
            strength: 0.5, clarity: 0.5, value_alignment: 1.0,
            constrained: false, active_constraints: Vec::new(),
            source_goal: None, created_at: 0,
        }
    }
    
    pub fn with_strength(mut self, strength: f32) -> Self { self.strength = strength.clamp(0.0, 1.0); self }
    pub fn with_clarity(mut self, clarity: f32) -> Self { self.clarity = clarity.clamp(0.0, 1.0); self }
    pub fn with_goal(mut self, goal_id: u64) -> Self { self.source_goal = Some(goal_id); self }
    
    pub fn effective_strength(&self) -> f32 {
        let base = self.strength * self.clarity * self.value_alignment;
        if self.constrained { base * 0.5 } else { base }
    }
    
    pub fn is_actionable(&self, threshold: f32) -> bool {
        self.effective_strength() >= threshold && !self.constrained
    }
    
    pub fn add_constraint(&mut self, constraint: impl Into<String>) {
        self.active_constraints.push(constraint.into());
        self.constrained = true;
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WillState {
    pub energy: f32,
    pub focus: f32,
    pub coherence: f32,
    pub in_conflict: bool,
    pub fatigue: f32,
    pub recovery_rate: f32,
}

impl Default for WillState {
    fn default() -> Self {
        Self { energy: 0.8, focus: 0.5, coherence: 0.8, in_conflict: false, fatigue: 0.0, recovery_rate: 0.01 }
    }
}

impl WillState {
    pub fn capacity(&self) -> f32 {
        (self.energy * self.coherence - self.fatigue * 0.5).clamp(0.0, 1.0)
    }
    
    pub fn spend(&mut self, amount: f32) {
        self.energy = (self.energy - amount).max(0.0);
        self.fatigue = (self.fatigue + amount * 0.3).min(1.0);
    }
    
    pub fn tick(&mut self) {
        self.energy = (self.energy + self.recovery_rate).min(1.0);
        self.fatigue = (self.fatigue - self.recovery_rate * 0.5).max(0.0);
    }
    
    pub fn is_exhausted(&self) -> bool { self.capacity() < 0.1 }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WillEngine {
    state: WillState,
    constraints: Vec<WillConstraint>,
    volitions: Vec<Volition>,
    next_id: u64,
    tick: u64,
    max_volitions: usize,
}

impl Default for WillEngine {
    fn default() -> Self { Self::new() }
}

impl WillEngine {
    pub fn new() -> Self {
        let mut engine = Self {
            state: WillState::default(), constraints: Vec::new(),
            volitions: Vec::new(), next_id: 1, tick: 0, max_volitions: 5,
        };
        engine.constraints.push(WillConstraint::ethical("Do not harm others intentionally"));
        engine.constraints.push(WillConstraint::ethical("Honor commitments"));
        engine
    }
    
    pub fn state(&self) -> &WillState { &self.state }
    
    pub fn will_goal(&mut self, description: &str, goal_id: u64, priority: f32) -> Option<Volition> {
        if self.state.is_exhausted() { return None; }
        let id = self.next_id; self.next_id += 1;
        let mut volition = Volition::new(id, VolitionType::Pursue, description)
            .with_strength(priority).with_clarity(self.state.focus).with_goal(goal_id);
        volition.created_at = self.tick;
        
        for constraint in &self.constraints {
            if constraint.active && constraint.blocks(volition.strength) {
                volition.add_constraint(constraint.description.clone());
            }
        }
        
        self.state.spend(volition.strength * 0.1);
        if self.volitions.len() < self.max_volitions {
            self.volitions.push(volition.clone());
        }
        Some(volition)
    }
    
    pub fn volitions(&self) -> &[Volition] { &self.volitions }
    
    pub fn primary_volition(&self) -> Option<&Volition> {
        self.volitions.iter().filter(|v| v.is_actionable(0.1))
            .max_by(|a, b| a.effective_strength().partial_cmp(&b.effective_strength()).unwrap())
    }
    
    pub fn tick(&mut self) {
        self.tick += 1;
        self.state.tick();
        self.volitions.retain(|v| self.tick - v.created_at < 1000);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_will_engine() {
        let mut engine = WillEngine::new();
        let volition = engine.will_goal("Test", 1, 0.7);
        assert!(volition.is_some());
    }
}
