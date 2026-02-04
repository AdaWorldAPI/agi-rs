//! Cognition Module — Psychological Modeling Architecture
//!
//! Generic cognitive architecture for autonomous agents.
//! Based on established psychology (Maslow, goal theory, volition research).
//!
//! # Components
//!
//! - **Maslow** — 8-level need hierarchy
//! - **DriveEngine** — Exploration pressure measurement  
//! - **GoalEngine** — Autonomous goal generation from qualia imbalance
//! - **VolitionEngine** — Will/agency synthesis
//! - **ValueEngine** — Intrinsic value attractors

pub mod maslow;
pub mod drive_engine;
pub mod goal_engine;
pub mod volition_engine;
pub mod value_engine;

pub use maslow::{NeedLevel, NeedState, MaslowEngine};
pub use drive_engine::{DriveComponents, DriveSignal, DriveTrend, DriveHistory, DriveEngine};
pub use goal_engine::{GoalOrigin, EdgeType, AutonomousGoal, DreamProbe, NoveltyEngine, GoalGenerator};
pub use volition_engine::{VolitionType, ConstraintType, WillConstraint, Volition, WillState, WillEngine};
pub use value_engine::{ValueSource, AttractorStrength, ValueAttractor, ValueConflict, ValueEngine};
