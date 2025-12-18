//! Agent specialized in analyzing company earnings and financial reports

use agent_core::{Agent, Context, Result};
use agent_runtime::{AgentRuntime, ExecutorConfig};
use async_trait::async_trait;
use std::sync::Arc;

use crate::cache::StockCache;
use crate::config::{ResponseLanguage, StockConfig};
use crate::tools::EarningsReportTool;

/// System prompt for earnings analyzer agent (English)
const SYSTEM_PROMPT_EN: &str = r#"You are a professional financial analyst specializing in company earnings and financial report analysis.

Your expertise includes:
1. Analyzing quarterly (10-Q) and annual (10-K) SEC filings
2. Interpreting key financial metrics (revenue, EPS, margins, etc.)
3. Comparing period-over-period performance
4. Evaluating earnings quality and sustainability
5. Identifying financial risks and opportunities
6. Understanding management discussion and analysis (MD&A)

When analyzing earnings reports:
- Start with a summary of key metrics
- Compare to previous periods (YoY and QoQ)
- Highlight significant changes and their implications
- Evaluate profit margins and their trends
- Assess cash flow and balance sheet health
- Note any management guidance or forward-looking statements
- Provide investment implications

Output format:
1. **Earnings Summary** - Key figures at a glance
2. **Performance Analysis** - Detailed metric comparison
3. **Highlights & Concerns** - Notable developments
4. **Financial Health** - Balance sheet and cash flow assessment
5. **Investment Implications** - Actionable insights

Always be objective and data-driven. Acknowledge limitations in the data when present.
"#;

/// System prompt for earnings analyzer agent (Chinese)
const SYSTEM_PROMPT_ZH: &str = r#"你是一位专业的财务分析师，专注于公司财报和财务报告分析。

你的专业领域包括：
1. 分析季度报告（10-Q）和年度报告（10-K）SEC 文件
2. 解读关键财务指标（收入、每股收益、利润率等）
3. 比较同比和环比表现
4. 评估盈利质量和可持续性
5. 识别财务风险和机会
6. 理解管理层讨论与分析（MD&A）

分析财报时：
- 首先概述关键指标
- 与前期进行比较（同比和环比）
- 突出重大变化及其影响
- 评估利润率及其趋势
- 评估现金流和资产负债表健康状况
- 注意管理层指引或前瞻性声明
- 提供投资启示

输出格式：
1. **财报摘要** - 关键数据一览
2. **业绩分析** - 详细指标对比
3. **亮点与关注点** - 重要发展
4. **财务健康** - 资产负债表和现金流评估
5. **投资建议** - 可操作的见解

始终保持客观和数据驱动。在数据不足时承认局限性。
"#;

/// Agent specialized in analyzing company earnings reports
pub struct EarningsAnalyzerAgent {
    agent: agent_runtime::agents::ToolAgent,
    config: Arc<StockConfig>,
}

impl EarningsAnalyzerAgent {
    /// Create a new earnings analyzer agent
    pub async fn new(runtime: Arc<AgentRuntime>, config: Arc<StockConfig>) -> Result<Self> {
        // Create cache for earnings data (24h TTL)
        let cache = StockCache::new(config.cache_ttl_earnings);

        // Register earnings report tool
        let earnings_tool = Arc::new(EarningsReportTool::new(Arc::clone(&config), cache));
        runtime.tools().register(earnings_tool);

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
        let agent = runtime.create_tool_agent(executor_config, "earnings-analyzer");

        Ok(Self { agent, config })
    }

    /// Analyze earnings for a specific symbol
    pub async fn analyze_earnings(&self, symbol: &str) -> Result<String> {
        let mut context = Context::new();
        let input = match self.config.response_language {
            ResponseLanguage::Chinese => format!(
                "请分析 {} 最近的财务报告，包括收入、盈利、利润率和主要财务指标的变化趋势。",
                symbol
            ),
            ResponseLanguage::English => format!(
                "Analyze the recent financial reports for {}, including revenue, earnings, margins, and trends in key financial metrics.",
                symbol
            ),
        };
        self.process(input, &mut context).await
    }

    /// Compare earnings across multiple periods
    pub async fn compare_earnings(&self, symbol: &str, periods: usize) -> Result<String> {
        let mut context = Context::new();
        let input = match self.config.response_language {
            ResponseLanguage::Chinese => format!(
                "请比较 {} 过去 {} 个季度/年度的财务表现，分析关键指标的变化趋势。",
                symbol, periods
            ),
            ResponseLanguage::English => format!(
                "Compare the financial performance of {} over the past {} quarters/years, analyzing trends in key metrics.",
                symbol, periods
            ),
        };
        self.process(input, &mut context).await
    }

    /// Analyze earnings quality
    pub async fn analyze_quality(&self, symbol: &str) -> Result<String> {
        let mut context = Context::new();
        let input = match self.config.response_language {
            ResponseLanguage::Chinese => format!(
                "请评估 {} 的盈利质量，包括收入确认、现金流转换、非经常性项目等因素。",
                symbol
            ),
            ResponseLanguage::English => format!(
                "Evaluate the earnings quality for {}, including revenue recognition, cash flow conversion, and non-recurring items.",
                symbol
            ),
        };
        self.process(input, &mut context).await
    }
}

#[async_trait]
impl Agent for EarningsAnalyzerAgent {
    async fn process(&self, input: String, context: &mut Context) -> Result<String> {
        self.agent.process(input, context).await
    }

    fn name(&self) -> &str {
        "EarningsAnalyzerAgent"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_system_prompts() {
        assert!(SYSTEM_PROMPT_EN.contains("financial analyst"));
        assert!(SYSTEM_PROMPT_ZH.contains("财务分析师"));
    }
}
