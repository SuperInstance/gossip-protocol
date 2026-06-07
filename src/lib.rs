//! # gossip-protocol
//!
//! Gossip protocol simulation: rumor spreading, anti-entropy, scuttlebutt, membership.
//!
//! ## Modules
//! - `rumor` — Rumor mongering with TTL and fanout
//! - `anti_entropy` — Anti-entropy synchronization
//! - `membership` — Suspicion-based failure detection
//! - `fanout` — Fanout calculation and peer selection
//! - `convergence` — Convergence metrics and simulation

pub mod rumor;
pub mod anti_entropy;
pub mod membership;
pub mod fanout;
pub mod convergence;

pub use rumor::{Rumor, RumorStore, RumorId};
pub use anti_entropy::{AntiEntropy, Digest};
pub use membership::{MembershipList, MemberState, Member};
pub use fanout::{select_fanout_peers, calculate_fanout};
pub use convergence::{SimulatedNetwork, ConvergenceMetrics};
