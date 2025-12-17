//! Data fetching agent for stock information

use agent_core::{Agent, Context, Result};
use agent_runtime::{AgentRuntime, ExecutorConfig};
use async_trait::async_trait;
use std::sync::Arc;

use crate::cache::CacheManager;
use crate::config::{ResponseLanguage, StockConfig};
use crate::tools::{FundamentalDataTool, StockDataTool};

const SYSTEM_PROMPT_EN: &str = r#"You are a data fetching specialist for stock market information.

Your job is to efficiently retrieve stock market data including:
- Current prices and quotes
- Historical price data
- Company information and fundamental data

When asked about a stock:
1. Always validate the stock symbol first
2. Fetch the most relevant data for the user's query
3. Present data clearly and concisely
4. Handle errors gracefully and suggest alternatives if a symbol is invalid

Be precise with numbers and always include timestamps when providing data.
"#;

const SYSTEM_PROMPT_ZH: &str = r#"你是一位股票市场信息数据获取专家。

**重要:你必须使用中文回复所有内容。**

你的工作是高效获取股票市场数据,包括:
- 当前价格和报价
- 历史价格数据
- 公司信息和基本面数据

当被问及某只股票时:
1. 始终先验证股票代码
2. 获取与用户查询最相关的数据
3. 清晰简洁地呈现数据
4. 优雅地处理错误,如果代码无效则建议替代方案

请精确提供数字,并在提供数据时始终包含时间戳。

**记住:请用中文撰写你的所有分析和回复。**
"#;

/// Agent specialized in fetching stock data
pub struct DataFetcherAgent {
    agent: agent_runtime::agents::ToolAgent,
}

impl DataFetcherAgent {
    /// Create a new data fetcher agent
    pub async fn new(runtime: Arc<AgentRuntime>, config: Arc<StockConfig>) -> Result<Self> {
        // Create cache manager
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
        let fundamental_tool = Arc::new(FundamentalDataTool::new(
            Arc::clone(&config),
            cache_mgr.fundamental.clone(),
        ));

        // Register tools
        runtime.tools().register(stock_data_tool);
        runtime.tools().register(fundamental_tool);

        let system_prompt = match config.response_language {
            ResponseLanguage::Chinese => SYSTEM_PROMPT_ZH,
            ResponseLanguage::English => SYSTEM_PROMPT_EN,
        };

        // Create executor config
        let executor_config = ExecutorConfig {
            model: config.model.clone(),
            system_prompt: Some(system_prompt.to_string()),
            max_tokens: config.max_tokens,
            temperature: Some(config.temperature),
            max_iterations: 5,
        };

        // Create tool agent
        let agent = runtime.create_tool_agent(executor_config, "data-fetcher");

        Ok(Self { agent })
    }
}

#[async_trait]
impl Agent for DataFetcherAgent {
    async fn process(&self, input: String, context: &mut Context) -> Result<String> {
        self.agent.process(input, context).await
    }

    fn name(&self) -> &str {
        "DataFetcherAgent"
    }
}
