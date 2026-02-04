//! Drive Engine — Exploration Pressure Measurement
//!
//! Measures magnitude of exploration drive, NOT direction.
//! Formula: Drive(t) = w1·Compression + w2·Stagnation + w3·UnexpressedCapacity − w4·ThreatLoad

use std::collections::VecDeque;
use serde::{Serialize, Deserialize};

const EMA_ALPHA: f32 = 0.25;
const MAX_HISTORY: usize = 100;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub struct DriveComponents {
    pub compression: f32,
    pub stagnation: f32,
    pub unexpressed_capacity: f32,
    pub threat_load: f32,
    pub signal_noise: f32,
}

impl DriveComponents {
    pub fn new(compression: f32, stagnation: f32, unexpressed_capacity: f32, threat_load: f32) -> Self {
        Self {
            compression: compression.clamp(0.0, 1.0),
            stagnation: stagnation.clamp(0.0, 1.0),
            unexpressed_capacity: unexpressed_capacity.clamp(0.0, 1.0),
            threat_load: threat_load.clamp(0.0, 1.0),
            signal_noise: 0.0,
        }
    }
    
    pub fn from_qualia_imbalance(imbalance: &[f32]) -> Self {
        if imbalance.is_empty() { return Self::default(); }
        let mean: f32 = imbalance.iter().sum::<f32>() / imbalance.len() as f32;
        let variance: f32 = imbalance.iter().map(|x| (x - mean).powi(2)).sum::<f32>() / imbalance.len() as f32;
        let compression = variance.sqrt().clamp(0.0, 1.0);
        let stagnation = (1.0 - (mean - 0.5).abs() * 2.0).max(0.0);
        let unexpressed = imbalance.iter().map(|x| (x - 0.5).abs()).fold(0.0f32, |a, b| a.max(b)) * 2.0;
        Self::new(compression, stagnation, unexpressed, 0.0)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriveSignal {
    pub magnitude: f32,
    pub smoothed_magnitude: f32,
    pub confidence: f32,
    pub meta_uncertainty: f32,
    pub components: DriveComponents,
    pub timestamp: u64,
}

impl DriveSignal {
    pub fn action_magnitude(&self) -> f32 { self.smoothed_magnitude }
    
    pub fn is_actionable(&self, magnitude_threshold: f32, confidence_threshold: f32) -> bool {
        self.smoothed_magnitude >= magnitude_threshold && self.confidence >= confidence_threshold
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DriveTrend { Declining, Stable, Rising, Surging }

impl DriveTrend {
    pub fn from_slope(slope: f32) -> Self {
        if slope < -0.05 { DriveTrend::Declining }
        else if slope < 0.05 { DriveTrend::Stable }
        else if slope < 0.15 { DriveTrend::Rising }
        else { DriveTrend::Surging }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DriveHistory {
    signals: VecDeque<DriveSignal>,
    max_size: usize,
}

impl DriveHistory {
    pub fn new(max_size: usize) -> Self {
        Self { signals: VecDeque::with_capacity(max_size), max_size }
    }
    
    pub fn push(&mut self, signal: DriveSignal) {
        if self.signals.len() >= self.max_size { self.signals.pop_front(); }
        self.signals.push_back(signal);
    }
    
    pub fn latest(&self) -> Option<&DriveSignal> { self.signals.back() }
    
    pub fn trend(&self, window: usize) -> DriveTrend {
        let n = self.signals.len().min(window);
        if n < 2 { return DriveTrend::Stable; }
        let start = self.signals.len() - n;
        let (mut sum_x, mut sum_y, mut sum_xy, mut sum_xx) = (0.0, 0.0, 0.0, 0.0);
        for (i, signal) in self.signals.iter().skip(start).enumerate() {
            let x = i as f32;
            let y = signal.smoothed_magnitude;
            sum_x += x; sum_y += y; sum_xy += x * y; sum_xx += x * x;
        }
        let n_f = n as f32;
        let slope = (n_f * sum_xy - sum_x * sum_y) / (n_f * sum_xx - sum_x * sum_x + 0.001);
        DriveTrend::from_slope(slope)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriveEngine {
    weights: [f32; 4],
    ema_state: f32,
    tick: u64,
    history: DriveHistory,
}

impl Default for DriveEngine {
    fn default() -> Self { Self::new() }
}

impl DriveEngine {
    pub fn new() -> Self {
        Self {
            weights: [0.35, 0.25, 0.25, 0.15],
            ema_state: 0.0,
            tick: 0,
            history: DriveHistory::new(MAX_HISTORY),
        }
    }
    
    pub fn compute(&mut self, components: DriveComponents) -> DriveSignal {
        self.tick += 1;
        let raw = self.weights[0] * components.compression +
                  self.weights[1] * components.stagnation +
                  self.weights[2] * components.unexpressed_capacity -
                  self.weights[3] * components.threat_load;
        let magnitude = raw.clamp(0.0, 1.0);
        self.ema_state = EMA_ALPHA * magnitude + (1.0 - EMA_ALPHA) * self.ema_state;
        let meta_uncertainty = components.signal_noise * 0.6;
        let confidence = (1.0 - 0.3 * meta_uncertainty - 0.2 * components.signal_noise).clamp(0.0, 1.0);
        let signal = DriveSignal {
            magnitude, smoothed_magnitude: self.ema_state, confidence,
            meta_uncertainty, components, timestamp: self.tick,
        };
        self.history.push(signal.clone());
        signal
    }
    
    pub fn trend(&self) -> DriveTrend { self.history.trend(20) }
    pub fn latest(&self) -> Option<&DriveSignal> { self.history.latest() }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_drive_signal() {
        let mut engine = DriveEngine::new();
        let signal = engine.compute(DriveComponents::new(0.7, 0.4, 0.5, 0.1));
        assert!(signal.magnitude > 0.0);
    }
}
