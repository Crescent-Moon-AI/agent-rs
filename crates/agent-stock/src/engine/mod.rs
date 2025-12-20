//! Stock Analysis Engine
//!
//! Core coordination layer for multi-agent stock analysis

pub mod analysis_engine;
pub mod context;
pub mod result;

pub use analysis_engine::StockAnalysisEngine;
pub use context::AnalysisContext;
pub use result::{AnalysisResult, AnalysisType, ComparisonResult};
