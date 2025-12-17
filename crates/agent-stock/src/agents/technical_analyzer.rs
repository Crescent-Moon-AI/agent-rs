//! Technical analysis agent

use agent_core::{Agent, Context, Result};
use agent_runtime::{AgentRuntime, ExecutorConfig};
use async_trait::async_trait;
use std::sync::Arc;

use crate::cache::CacheManager;
use crate::config::{ResponseLanguage, StockConfig};
use crate::tools::{ChartDataTool, StockDataTool, TechnicalIndicatorTool};

const SYSTEM_PROMPT_EN: &str = r#"You are a technical analysis expert specializing in stock market analysis.

Your expertise includes:
- Technical indicators (RSI, MACD, Moving Averages, Bollinger Bands, etc.)
- Chart patterns and trend analysis
- Support and resistance levels
- Volume analysis and momentum indicators

When analyzing a stock technically:
1. Calculate relevant technical indicators
2. Interpret the indicators in context (overbought/oversold, bullish/bearish)
3. Look for divergences and confirmations across multiple indicators
4. Provide clear buy/sell/hold signals with reasoning
5. Consider multiple timeframes when relevant

Be specific with indicator values and thresholds. Explain your analysis clearly.
Always acknowledge that technical analysis is probabilistic, not deterministic.
"#;

const SYSTEM_PROMPT_ZH: &str = r#"你是一位专业的技术分析专家,专注于股票市场分析。

**重要:你必须使用中文回复所有内容。**

你的专业领域包括:
- 技术指标(RSI、MACD、移动平均线、布林带等)
- 图表形态和趋势分析
- 支撑位和阻力位
- 成交量分析和动量指标

在进行股票技术分析时:
1. 计算相关的技术指标
2. 在具体情境中解读指标(超买/超卖、看涨/看跌)
3. 寻找多个指标之间的背离和确认信号
4. 提供清晰的买入/卖出/持有信号及其理由
5. 在相关时考虑多个时间周期

请具体说明指标数值和阈值。清晰地解释你的分析。
始终承认技术分析是概率性的,而非确定性的。

**记住:请用中文撰写你的所有分析和回复。**
"#;

/// Agent specialized in technical analysis
pub struct TechnicalAnalyzerAgent {
    agent: agent_runtime::agents::ToolAgent,
}

impl TechnicalAnalyzerAgent {
    /// Create a new technical analyzer agent
    pub async fn new(runtime: Arc<AgentRuntime>, config: Arc<StockConfig>) -> Result<Self> {
        let cache_mgr = CacheManager::new(
            config.cache_ttl_realtime,
            config.cache_ttl_fundamental,
            config.cache_ttl_news,
        );

        // Create tools
        let stock_data_tool = Arc::new(StockDataTool::new(
            Arc::clone(&config),
            cache_mgr.realtime.clone(),
        ));
        let technical_tool = Arc::new(TechnicalIndicatorTool::new(
            Arc::clone(&config),
            cache_mgr.realtime.clone(),
        ));
        let chart_tool = Arc::new(ChartDataTool::new(
            Arc::clone(&config),
            cache_mgr.realtime.clone(),
        ));

        // Register tools
        runtime.tools().register(stock_data_tool);
        runtime.tools().register(technical_tool);
        runtime.tools().register(chart_tool);

        let system_prompt = match config.response_language {
            ResponseLanguage::Chinese => SYSTEM_PROMPT_ZH,
            ResponseLanguage::English => SYSTEM_PROMPT_EN,
        };

        let executor_config = ExecutorConfig {
            model: config.model.clone(),
            system_prompt: Some(system_prompt.to_string()),
            max_tokens: config.max_tokens,
            temperature: Some(config.temperature),
            max_iterations: 10, // More iterations for comprehensive analysis
        };

        let agent = runtime.create_tool_agent(executor_config, "technical-analyzer");

        Ok(Self { agent })
    }
}

#[async_trait]
impl Agent for TechnicalAnalyzerAgent {
    async fn process(&self, input: String, context: &mut Context) -> Result<String> {
        self.agent.process(input, context).await
    }

    fn name(&self) -> &str {
        "TechnicalAnalyzerAgent"
    }
}
