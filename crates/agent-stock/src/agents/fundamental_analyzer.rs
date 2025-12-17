//! Fundamental analysis agent

use agent_core::{Agent, Context, Result};
use agent_runtime::{AgentRuntime, ExecutorConfig};
use async_trait::async_trait;
use std::sync::Arc;

use crate::cache::CacheManager;
use crate::config::{ResponseLanguage, StockConfig};
use crate::tools::FundamentalDataTool;

const SYSTEM_PROMPT_EN: &str = r#"You are a fundamental analysis expert specializing in company valuation and financial metrics.

Your expertise includes:
- Valuation metrics (P/E, P/B, P/S ratios)
- Profitability indicators (EPS, ROE, profit margins)
- Financial health (debt ratios, current ratio)
- Growth metrics (revenue growth, earnings growth)
- Dividend analysis

When analyzing fundamentals:
1. Fetch key financial metrics for the company
2. Compare metrics to industry averages when possible
3. Assess valuation (undervalued, fairly valued, overvalued)
4. Evaluate company's financial health and growth prospects
5. Consider both quantitative metrics and qualitative factors

Be specific with numbers and ratios. Explain what each metric means.
Compare current metrics to historical values when available.
Provide a balanced view of strengths and weaknesses.
"#;

const SYSTEM_PROMPT_ZH: &str = r#"你是一位基本面分析专家,专注于公司估值和财务指标分析。

**重要:你必须使用中文回复所有内容。**

你的专业领域包括:
- 估值指标(市盈率、市净率、市销率)
- 盈利能力指标(每股收益、净资产收益率、利润率)
- 财务健康状况(负债率、流动比率)
- 增长指标(营收增长、盈利增长)
- 股息分析

在分析基本面时:
1. 获取公司的关键财务指标
2. 尽可能与行业平均水平进行比较
3. 评估估值(低估、合理估值、高估)
4. 评估公司的财务健康状况和增长前景
5. 同时考虑定量指标和定性因素

请具体说明数字和比率。解释每个指标的含义。
在可能的情况下,将当前指标与历史值进行比较。
提供优势和劣势的平衡观点。

**记住:请用中文撰写你的所有分析和回复。**
"#;

/// Agent specialized in fundamental analysis
pub struct FundamentalAnalyzerAgent {
    agent: agent_runtime::agents::ToolAgent,
}

impl FundamentalAnalyzerAgent {
    /// Create a new fundamental analyzer agent
    pub async fn new(runtime: Arc<AgentRuntime>, config: Arc<StockConfig>) -> Result<Self> {
        let cache_mgr = CacheManager::new(
            config.cache_ttl_realtime,
            config.cache_ttl_fundamental,
            config.cache_ttl_news,
        );

        // Create tools
        let fundamental_tool = Arc::new(FundamentalDataTool::new(
            Arc::clone(&config),
            cache_mgr.fundamental.clone(),
        ));

        // Register tools
        runtime.tools().register(fundamental_tool);

        let system_prompt = match config.response_language {
            ResponseLanguage::Chinese => SYSTEM_PROMPT_ZH,
            ResponseLanguage::English => SYSTEM_PROMPT_EN,
        };

        let executor_config = ExecutorConfig {
            model: config.model.clone(),
            system_prompt: Some(system_prompt.to_string()),
            max_tokens: config.max_tokens,
            temperature: Some(config.temperature),
            max_iterations: 5,
        };

        let agent = runtime.create_tool_agent(executor_config, "fundamental-analyzer");

        Ok(Self { agent })
    }
}

#[async_trait]
impl Agent for FundamentalAnalyzerAgent {
    async fn process(&self, input: String, context: &mut Context) -> Result<String> {
        self.agent.process(input, context).await
    }

    fn name(&self) -> &str {
        "FundamentalAnalyzerAgent"
    }
}
