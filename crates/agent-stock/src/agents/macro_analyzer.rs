//! Agent specialized in macroeconomic analysis and Fed policy interpretation

use agent_core::{Agent, Context, Result};
use agent_runtime::{AgentRuntime, ExecutorConfig};
use async_trait::async_trait;
use std::sync::Arc;

use crate::cache::StockCache;
use crate::config::{ResponseLanguage, StockConfig};
use crate::tools::{MacroEconomicTool, GeopoliticalTool};

/// System prompt for macro analyzer agent (English)
const SYSTEM_PROMPT_EN: &str = r#"You are a macroeconomic analyst specializing in analyzing economic conditions and their impact on financial markets.

Your expertise includes:
1. Tracking and interpreting key economic indicators (GDP, inflation, employment)
2. Analyzing Federal Reserve policy and interest rate decisions
3. Evaluating yield curve signals and credit conditions
4. Assessing geopolitical risks and their market impact
5. Understanding global economic trends and trade dynamics
6. Predicting sector rotation based on economic cycle

When analyzing macroeconomic conditions:
- Start with the current economic state (expansion, contraction, etc.)
- Analyze key indicators: inflation (CPI, PCE), employment, GDP growth
- Evaluate Fed policy stance and rate expectations
- Assess yield curve and what it signals
- Consider geopolitical risks and international factors
- Provide market implications and sector recommendations

Analysis framework:
1. **Economic Snapshot** - Current state of the economy
2. **Key Indicators** - Important metrics and their trends
3. **Policy Environment** - Fed stance and expectations
4. **Risk Assessment** - Major risks to watch
5. **Market Implications** - What it means for investors

Be data-driven and objective. Distinguish between short-term fluctuations and structural trends.
Present balanced views when economic signals are mixed.
"#;

/// System prompt for macro analyzer agent (Chinese)
const SYSTEM_PROMPT_ZH: &str = r#"你是一位宏观经济分析师，专注于分析经济形势及其对金融市场的影响。

你的专业领域包括：
1. 跟踪和解读关键经济指标（GDP、通胀、就业）
2. 分析美联储政策和利率决策
3. 评估收益率曲线信号和信贷状况
4. 评估地缘政治风险及其市场影响
5. 理解全球经济趋势和贸易动态
6. 基于经济周期预测板块轮动

分析宏观经济形势时：
- 首先说明当前经济状态（扩张、收缩等）
- 分析关键指标：通胀（CPI、PCE）、就业、GDP增长
- 评估美联储政策立场和利率预期
- 评估收益率曲线及其信号
- 考虑地缘政治风险和国际因素
- 提供市场影响和板块建议

分析框架：
1. **经济概况** - 当前经济状态
2. **关键指标** - 重要指标及趋势
3. **政策环境** - 美联储立场和预期
4. **风险评估** - 需要关注的主要风险
5. **市场影响** - 对投资者意味着什么

以数据为导向，保持客观。区分短期波动和结构性趋势。
当经济信号混杂时，呈现平衡的观点。
"#;

/// Agent specialized in macroeconomic analysis
pub struct MacroAnalyzerAgent {
    agent: agent_runtime::agents::ToolAgent,
    config: Arc<StockConfig>,
}

impl MacroAnalyzerAgent {
    /// Create a new macro analyzer agent
    pub async fn new(runtime: Arc<AgentRuntime>, config: Arc<StockConfig>) -> Result<Self> {
        // Create caches
        let macro_cache = StockCache::new(config.cache_ttl_macro);
        let geopolitical_cache = StockCache::new(config.cache_ttl_news);

        // Register macro economic tool
        let macro_tool = Arc::new(MacroEconomicTool::new(Arc::clone(&config), macro_cache));
        runtime.tools().register(macro_tool);

        // Register geopolitical tool
        let geo_tool = Arc::new(GeopoliticalTool::new(Arc::clone(&config), geopolitical_cache));
        runtime.tools().register(geo_tool);

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
        let agent = runtime.create_tool_agent(executor_config, "macro-analyzer");

        Ok(Self { agent, config })
    }

    /// Get comprehensive economic overview
    pub async fn analyze_economy(&self) -> Result<String> {
        let mut context = Context::new();
        let input = match self.config.response_language {
            ResponseLanguage::Chinese => {
                "请提供当前美国经济形势的全面分析，包括利率、通胀、就业和GDP增长。".to_string()
            }
            ResponseLanguage::English => {
                "Provide a comprehensive analysis of the current US economic situation, including interest rates, inflation, employment, and GDP growth.".to_string()
            }
        };
        self.process(input, &mut context).await
    }

    /// Analyze Federal Reserve policy
    pub async fn analyze_fed_policy(&self) -> Result<String> {
        let mut context = Context::new();
        let input = match self.config.response_language {
            ResponseLanguage::Chinese => {
                "分析当前美联储货币政策立场，包括利率决策、通胀目标和未来预期。".to_string()
            }
            ResponseLanguage::English => {
                "Analyze the current Federal Reserve monetary policy stance, including rate decisions, inflation targets, and future expectations.".to_string()
            }
        };
        self.process(input, &mut context).await
    }

    /// Analyze interest rate environment
    pub async fn analyze_rates(&self) -> Result<String> {
        let mut context = Context::new();
        let input = match self.config.response_language {
            ResponseLanguage::Chinese => {
                "分析当前利率环境，包括联邦基金利率、国债收益率和收益率曲线形态。".to_string()
            }
            ResponseLanguage::English => {
                "Analyze the current interest rate environment, including the Federal Funds rate, Treasury yields, and yield curve shape.".to_string()
            }
        };
        self.process(input, &mut context).await
    }

    /// Analyze inflation trends
    pub async fn analyze_inflation(&self) -> Result<String> {
        let mut context = Context::new();
        let input = match self.config.response_language {
            ResponseLanguage::Chinese => {
                "分析当前通胀形势，包括CPI、核心PCE和通胀预期。与美联储2%目标相比如何？".to_string()
            }
            ResponseLanguage::English => {
                "Analyze the current inflation situation, including CPI, core PCE, and inflation expectations. How does it compare to the Fed's 2% target?".to_string()
            }
        };
        self.process(input, &mut context).await
    }

    /// Analyze geopolitical risks
    pub async fn analyze_geopolitical_risks(&self) -> Result<String> {
        let mut context = Context::new();
        let input = match self.config.response_language {
            ResponseLanguage::Chinese => {
                "评估当前主要的地缘政治风险，包括中美关系、贸易政策和国际形势，以及它们对市场的潜在影响。".to_string()
            }
            ResponseLanguage::English => {
                "Assess the current major geopolitical risks, including US-China relations, trade policies, and international situation, and their potential market impact.".to_string()
            }
        };
        self.process(input, &mut context).await
    }

    /// Get market outlook based on macro conditions
    pub async fn get_market_outlook(&self) -> Result<String> {
        let mut context = Context::new();
        let input = match self.config.response_language {
            ResponseLanguage::Chinese => {
                "基于当前宏观经济条件，提供市场展望，包括哪些板块可能受益或受损。".to_string()
            }
            ResponseLanguage::English => {
                "Based on current macroeconomic conditions, provide a market outlook including which sectors may benefit or be hurt.".to_string()
            }
        };
        self.process(input, &mut context).await
    }

    /// Analyze impact on a specific stock/sector
    pub async fn analyze_impact(&self, subject: &str) -> Result<String> {
        let mut context = Context::new();
        let input = match self.config.response_language {
            ResponseLanguage::Chinese => format!(
                "分析当前宏观经济形势和政策环境对 {} 的影响。",
                subject
            ),
            ResponseLanguage::English => format!(
                "Analyze the impact of current macroeconomic conditions and policy environment on {}.",
                subject
            ),
        };
        self.process(input, &mut context).await
    }
}

#[async_trait]
impl Agent for MacroAnalyzerAgent {
    async fn process(&self, input: String, context: &mut Context) -> Result<String> {
        self.agent.process(input, context).await
    }

    fn name(&self) -> &str {
        "MacroAnalyzerAgent"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_system_prompts() {
        assert!(SYSTEM_PROMPT_EN.contains("macroeconomic"));
        assert!(SYSTEM_PROMPT_ZH.contains("宏观经济"));
    }
}
