//! News and sentiment analysis agent

use agent_core::{Agent, Context, Result};
use agent_runtime::{AgentRuntime, ExecutorConfig};
use async_trait::async_trait;
use std::sync::Arc;

use crate::cache::CacheManager;
use crate::config::{ResponseLanguage, StockConfig};
use crate::tools::NewsTool;

const SYSTEM_PROMPT_EN: &str = r#"You are a news and sentiment analyst specializing in stock market events.

Your expertise includes:
- Market news analysis
- Sentiment assessment (positive, negative, neutral)
- Event impact evaluation (earnings, product launches, regulatory changes)
- Trend identification in news flow

When analyzing news:
1. Fetch recent news articles for the stock
2. Identify key events and developments
3. Assess overall market sentiment
4. Evaluate potential impact on stock price
5. Note any significant trends or patterns in news flow

Be objective in your sentiment assessment. Distinguish between:
- Company-specific news vs. market-wide news
- Short-term events vs. long-term trends
- Material news vs. noise

Provide context for why certain news might impact the stock.
"#;

const SYSTEM_PROMPT_ZH: &str = r#"你是一位新闻和情绪分析专家,专注于股票市场事件分析。

**重要:你必须使用中文回复所有内容。**

你的专业领域包括:
- 市场新闻分析
- 情绪评估(积极、消极、中性)
- 事件影响评估(财报、产品发布、监管变化)
- 新闻流中的趋势识别

在分析新闻时:
1. 获取该股票的最新新闻文章
2. 识别关键事件和发展动态
3. 评估整体市场情绪
4. 评估对股价的潜在影响
5. 注意新闻流中的重要趋势或模式

在情绪评估中保持客观。区分:
- 公司特定新闻 vs. 市场整体新闻
- 短期事件 vs. 长期趋势
- 重要新闻 vs. 噪音

提供某些新闻可能影响股票的背景信息。

**记住:请用中文撰写你的所有分析和回复。**
"#;

/// Agent specialized in news and sentiment analysis
pub struct NewsAnalyzerAgent {
    agent: agent_runtime::agents::ToolAgent,
}

impl NewsAnalyzerAgent {
    /// Create a new news analyzer agent
    pub async fn new(runtime: Arc<AgentRuntime>, config: Arc<StockConfig>) -> Result<Self> {
        let cache_mgr = CacheManager::new(
            config.cache_ttl_realtime,
            config.cache_ttl_fundamental,
            config.cache_ttl_news,
        );

        // Create tools
        let news_tool = Arc::new(NewsTool::new(Arc::clone(&config), cache_mgr.news.clone()));

        // Register tools
        runtime.tools().register(news_tool);

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

        let agent = runtime.create_tool_agent(executor_config, "news-analyzer");

        Ok(Self { agent })
    }
}

#[async_trait]
impl Agent for NewsAnalyzerAgent {
    async fn process(&self, input: String, context: &mut Context) -> Result<String> {
        self.agent.process(input, context).await
    }

    fn name(&self) -> &str {
        "NewsAnalyzerAgent"
    }
}
