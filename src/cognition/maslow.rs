//! Maslow Hierarchy — Need Level Psychology
//!
//! Standard 8-level Maslow hierarchy for modeling agent needs.
//! This is textbook psychology, not personality-specific.

use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum NeedLevel {
    Physiological = 0,
    Safety = 1,
    Love = 2,
    Esteem = 3,
    Cognitive = 4,
    Aesthetic = 5,
    SelfActualization = 6,
    Transcendence = 7,
}

impl NeedLevel {
    pub fn all() -> &'static [NeedLevel] {
        &[
            NeedLevel::Physiological, NeedLevel::Safety, NeedLevel::Love,
            NeedLevel::Esteem, NeedLevel::Cognitive, NeedLevel::Aesthetic,
            NeedLevel::SelfActualization, NeedLevel::Transcendence,
        ]
    }
    
    pub fn name(&self) -> &'static str {
        match self {
            NeedLevel::Physiological => "Physiological",
            NeedLevel::Safety => "Safety",
            NeedLevel::Love => "Love/Belonging",
            NeedLevel::Esteem => "Esteem",
            NeedLevel::Cognitive => "Cognitive",
            NeedLevel::Aesthetic => "Aesthetic",
            NeedLevel::SelfActualization => "Self-Actualization",
            NeedLevel::Transcendence => "Transcendence",
        }
    }
    
    pub fn is_deficiency(&self) -> bool { (*self as u8) < 4 }
    pub fn is_growth(&self) -> bool { (*self as u8) >= 4 }
}

impl Default for NeedLevel {
    fn default() -> Self { NeedLevel::Physiological }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub struct NeedState {
    pub satisfaction: f32,
    pub urgency: f32,
    pub time_since_satisfied: u64,
    pub active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaslowEngine {
    states: [NeedState; 8],
    satisfaction_threshold: f32,
    decay_rate: f32,
    tick: u64,
}

impl Default for MaslowEngine {
    fn default() -> Self { Self::new() }
}

impl MaslowEngine {
    pub fn new() -> Self {
        Self {
            states: [NeedState::default(); 8],
            satisfaction_threshold: 0.6,
            decay_rate: 0.001,
            tick: 0,
        }
    }
    
    pub fn satisfy(&mut self, level: NeedLevel, amount: f32) {
        let state = &mut self.states[level as usize];
        state.satisfaction = (state.satisfaction + amount).clamp(0.0, 1.0);
    }
    
    pub fn current_focus(&self) -> NeedLevel {
        for level in NeedLevel::all() {
            if self.states[*level as usize].satisfaction < self.satisfaction_threshold {
                return *level;
            }
        }
        NeedLevel::Transcendence
    }
    
    pub fn can_pursue(&self, level: NeedLevel) -> bool {
        for lower in NeedLevel::all() {
            if *lower >= level { break; }
            if self.states[*lower as usize].satisfaction < self.satisfaction_threshold {
                return false;
            }
        }
        true
    }
    
    pub fn wellbeing(&self) -> f32 {
        let weights = [1.0, 0.9, 0.8, 0.7, 0.6, 0.5, 0.4, 0.3];
        let mut total = 0.0;
        let mut weight_sum = 0.0;
        for (i, state) in self.states.iter().enumerate() {
            total += state.satisfaction * weights[i];
            weight_sum += weights[i];
        }
        total / weight_sum
    }
    
    pub fn tick(&mut self) {
        self.tick += 1;
        for state in &mut self.states {
            state.satisfaction = (state.satisfaction - self.decay_rate).max(0.0);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_maslow_focus() {
        let mut engine = MaslowEngine::new();
        assert_eq!(engine.current_focus(), NeedLevel::Physiological);
        engine.satisfy(NeedLevel::Physiological, 0.8);
        assert_eq!(engine.current_focus(), NeedLevel::Safety);
    }
}
