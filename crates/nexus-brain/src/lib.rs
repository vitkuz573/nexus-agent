pub mod scaffold;
pub mod thought;
pub mod verify;
pub mod memory;
pub mod diff;
pub mod graph;
pub mod hypothesis;
pub mod risk;
pub mod search;
pub mod architect;

pub use scaffold::CognitiveScaffold;
pub use thought::{ThoughtChain, ThoughtNode, ThoughtType};
pub use verify::CodeVerifier;
pub use memory::MemoryPalace;
pub use diff::SemanticDiff;
pub use graph::CodeGraph;
pub use hypothesis::HypothesisEngine;
pub use risk::RiskAnalyzer;
pub use search::NeuralSearch;
pub use architect::AutoArchitect;

#[cfg(test)]
mod thought_tests;
#[cfg(test)]
mod scaffold_tests;
#[cfg(test)]
mod verify_tests;
#[cfg(test)]
mod memory_tests;
#[cfg(test)]
mod diff_tests;
#[cfg(test)]
mod graph_tests;
#[cfg(test)]
mod hypothesis_tests;
#[cfg(test)]
mod risk_tests;
#[cfg(test)]
mod search_tests;
#[cfg(test)]
mod architect_tests;
