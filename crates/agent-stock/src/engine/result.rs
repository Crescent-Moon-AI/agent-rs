//! Analysis result types

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AnalysisType {
    Technical,
    Fundamental,
    News,
    Earnings,
    Macro,
    Geopolitical,
    Comprehensive,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisResult {
    pub symbol: String,
    pub analysis_type: AnalysisType,
    pub content: String,
    pub data: HashMap<String, serde_json::Value>,
    pub timestamp: DateTime<Utc>,
    pub data_freshness: DataFreshness,
    pub confidence: Option<f64>,
    pub warnings: Vec<String>,
    pub sources: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DataFreshness {
    RealTime,
    Recent,
    Stale,
    Partial,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComparisonResult {
    pub symbols: Vec<String>,
    pub analyses: HashMap<String, AnalysisResult>,
    pub summary: String,
    pub metrics: ComparisonMetrics,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ComparisonMetrics {
    pub performance: HashMap<String, PerformanceMetric>,
    pub valuation: HashMap<String, ValuationMetric>,
    pub risk: HashMap<String, RiskMetric>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PerformanceMetric {
    pub return_1d: Option<f64>,
    pub return_1w: Option<f64>,
    pub return_1m: Option<f64>,
    pub return_ytd: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ValuationMetric {
    pub pe_ratio: Option<f64>,
    pub pb_ratio: Option<f64>,
    pub market_cap: Option<f64>,
    pub dividend_yield: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RiskMetric {
    pub beta: Option<f64>,
    pub week_52_high: Option<f64>,
    pub week_52_low: Option<f64>,
    pub avg_volume: Option<f64>,
}

impl AnalysisResult {
    pub fn new(symbol: impl Into<String>, analysis_type: AnalysisType, content: impl Into<String>) -> Self {
        Self {
            symbol: symbol.into(),
            analysis_type,
            content: content.into(),
            data: HashMap::new(),
            timestamp: Utc::now(),
            data_freshness: DataFreshness::Recent,
            confidence: None,
            warnings: Vec::new(),
            sources: Vec::new(),
        }
    }
    
    pub fn with_data(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        self.data.insert(key.into(), value);
        self
    }
    
    pub fn with_freshness(mut self, freshness: DataFreshness) -> Self {
        self.data_freshness = freshness;
        self
    }
    
    pub fn with_confidence(mut self, confidence: f64) -> Self {
        self.confidence = Some(confidence.clamp(0.0, 1.0));
        self
    }
    
    pub fn add_warning(&mut self, warning: impl Into<String>) {
        self.warnings.push(warning.into());
    }
    
    pub fn add_source(mut self, source: impl Into<String>) -> Self {
        let source = source.into();
        if !self.sources.contains(&source) {
            self.sources.push(source);
        }
        self
    }
    
    pub fn is_fresh(&self) -> bool {
        matches!(self.data_freshness, DataFreshness::RealTime | DataFreshness::Recent)
    }
    
    pub fn summary(&self) -> String {
        let freshness_indicator = match self.data_freshness {
            DataFreshness::RealTime => "🟢",
            DataFreshness::Recent => "🟡",
            DataFreshness::Stale => "🟠",
            DataFreshness::Partial => "⚠️",
        };
        
        format!(
            "{} {} Analysis - {} ({})",
            freshness_indicator,
            self.symbol,
            format!("{:?}", self.analysis_type),
            self.timestamp.format("%Y-%m-%d %H:%M UTC")
        )
    }
}

impl ComparisonResult {
    pub fn new(symbols: Vec<String>) -> Self {
        Self {
            symbols,
            analyses: HashMap::new(),
            summary: String::new(),
            metrics: ComparisonMetrics::default(),
            timestamp: Utc::now(),
        }
    }
    
    pub fn add_analysis(&mut self, symbol: String, analysis: AnalysisResult) {
        self.analyses.insert(symbol, analysis);
    }
    
    pub fn with_summary(mut self, summary: impl Into<String>) -> Self {
        self.summary = summary.into();
        self
    }
    
    pub fn is_complete(&self) -> bool {
        self.symbols.iter().all(|s| self.analyses.contains_key(s))
    }
    
    pub fn success_rate(&self) -> f64 {
        if self.symbols.is_empty() {
            return 0.0;
        }
        self.analyses.len() as f64 / self.symbols.len() as f64
    }
}
