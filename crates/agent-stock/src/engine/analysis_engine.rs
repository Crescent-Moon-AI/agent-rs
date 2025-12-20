//! Stock Analysis Engine - delegates to existing StockAnalysisAgent

use crate::agents::StockAnalysisAgent;
use crate::config::StockConfig;
use crate::error::Result;
use crate::router::SmartRouter;
use agent_runtime::AgentRuntime;
use std::sync::Arc;

use super::context::AnalysisContext;
use super::result::{AnalysisResult, AnalysisType, ComparisonResult};

/// Stock Analysis Engine - wrapper around StockAnalysisAgent
pub struct StockAnalysisEngine {
    agent: StockAnalysisAgent,
    router: SmartRouter,
}

impl StockAnalysisEngine {
    pub async fn new(runtime: Arc<AgentRuntime>, config: Arc<StockConfig>) -> Result<Self> {
        let agent = StockAnalysisAgent::new(runtime, config).await?;
        let router = SmartRouter::new();
        
        Ok(Self { agent, router })
    }
    
    pub async fn analyze_stock(
        &self,
        symbol: &str,
        _ctx: &mut AnalysisContext,
    ) -> Result<AnalysisResult> {
        let content = self.agent.analyze(symbol).await?;
        Ok(AnalysisResult::new(symbol, AnalysisType::Comprehensive, content))
    }
    
    pub async fn analyze_technical(
        &self,
        symbol: &str,
        _ctx: &mut AnalysisContext,
    ) -> Result<AnalysisResult> {
        let content = self.agent.analyze_technical(symbol).await?;
        Ok(AnalysisResult::new(symbol, AnalysisType::Technical, content))
    }
    
    pub async fn analyze_fundamental(
        &self,
        symbol: &str,
        _ctx: &mut AnalysisContext,
    ) -> Result<AnalysisResult> {
        let content = self.agent.analyze_fundamental(symbol).await?;
        Ok(AnalysisResult::new(symbol, AnalysisType::Fundamental, content))
    }
    
    pub async fn analyze_news(
        &self,
        symbol: &str,
        _ctx: &mut AnalysisContext,
    ) -> Result<AnalysisResult> {
        let content = self.agent.analyze_news(symbol).await?;
        Ok(AnalysisResult::new(symbol, AnalysisType::News, content))
    }
    
    pub async fn analyze_earnings(
        &self,
        symbol: &str,
        _ctx: &mut AnalysisContext,
    ) -> Result<AnalysisResult> {
        let content = self.agent.analyze_earnings(symbol).await?;
        Ok(AnalysisResult::new(symbol, AnalysisType::Earnings, content))
    }
    
    pub async fn analyze_macro(&self, _ctx: &mut AnalysisContext) -> Result<AnalysisResult> {
        let content = self.agent.analyze_macro().await?;
        Ok(AnalysisResult::new("MARKET", AnalysisType::Macro, content))
    }
    
    pub async fn compare_stocks(
        &self,
        symbols: &[String],
        _ctx: &mut AnalysisContext,
    ) -> Result<ComparisonResult> {
        let content = self.agent.compare_stocks(symbols).await?;
        let mut result = ComparisonResult::new(symbols.to_vec());
        result = result.with_summary(content);
        Ok(result)
    }
    
    pub fn router(&self) -> &SmartRouter {
        &self.router
    }
}
