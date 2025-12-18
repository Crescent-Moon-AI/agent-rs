//! User message templates for stock analysis agents

use agent_prompt::{JinjaTemplate, Result};

// ============================================================================
// Earnings Analyzer User Messages
// ============================================================================

/// Create the analyze earnings user message template
pub fn analyze_earnings_prompt() -> Result<JinjaTemplate> {
    JinjaTemplate::bilingual(
        "stock.user.analyze_earnings",
        "Analyze the recent financial reports for {{ symbol }}, including revenue, earnings, margins, and trends in key financial metrics.",
        "请分析 {{ symbol }} 最近的财务报告，包括收入、盈利、利润率和主要财务指标的变化趋势。",
    )
}

/// Create the compare earnings user message template
pub fn compare_earnings_prompt() -> Result<JinjaTemplate> {
    JinjaTemplate::bilingual(
        "stock.user.compare_earnings",
        "Compare the financial performance of {{ symbol }} over the past {{ periods }} quarters/years, analyzing trends in key metrics.",
        "请比较 {{ symbol }} 过去 {{ periods }} 个季度/年度的财务表现，分析关键指标的变化趋势。",
    )
}

/// Create the analyze earnings quality user message template
pub fn analyze_quality_prompt() -> Result<JinjaTemplate> {
    JinjaTemplate::bilingual(
        "stock.user.analyze_quality",
        "Evaluate the earnings quality for {{ symbol }}, including revenue recognition, cash flow conversion, and non-recurring items.",
        "请评估 {{ symbol }} 的盈利质量，包括收入确认、现金流转换、非经常性项目等因素。",
    )
}

// ============================================================================
// Macro Analyzer User Messages
// ============================================================================

/// Create the analyze economy user message template
pub fn analyze_economy_prompt() -> Result<JinjaTemplate> {
    JinjaTemplate::bilingual(
        "stock.user.analyze_economy",
        "Provide a comprehensive analysis of the current US economic situation, including interest rates, inflation, employment, and GDP growth.",
        "请提供当前美国经济形势的全面分析，包括利率、通胀、就业和GDP增长。",
    )
}

/// Create the analyze Fed policy user message template
pub fn analyze_fed_policy_prompt() -> Result<JinjaTemplate> {
    JinjaTemplate::bilingual(
        "stock.user.analyze_fed_policy",
        "Analyze the current Federal Reserve monetary policy stance, including rate decisions, inflation targets, and future expectations.",
        "分析当前美联储货币政策立场，包括利率决策、通胀目标和未来预期。",
    )
}

/// Create the analyze rates user message template
pub fn analyze_rates_prompt() -> Result<JinjaTemplate> {
    JinjaTemplate::bilingual(
        "stock.user.analyze_rates",
        "Analyze the current interest rate environment, including the Federal Funds rate, Treasury yields, and yield curve shape.",
        "分析当前利率环境，包括联邦基金利率、国债收益率和收益率曲线形态。",
    )
}

/// Create the analyze inflation user message template
pub fn analyze_inflation_prompt() -> Result<JinjaTemplate> {
    JinjaTemplate::bilingual(
        "stock.user.analyze_inflation",
        "Analyze the current inflation situation, including CPI, core PCE, and inflation expectations. How does it compare to the Fed's 2% target?",
        "分析当前通胀形势，包括CPI、核心PCE和通胀预期。与美联储2%目标相比如何？",
    )
}

/// Create the analyze geopolitical risks user message template
pub fn analyze_geopolitical_risks_prompt() -> Result<JinjaTemplate> {
    JinjaTemplate::bilingual(
        "stock.user.analyze_geopolitical_risks",
        "Assess the current major geopolitical risks, including US-China relations, trade policies, and international situation, and their potential market impact.",
        "评估当前主要的地缘政治风险，包括中美关系、贸易政策和国际形势，以及它们对市场的潜在影响。",
    )
}

/// Create the get market outlook user message template
pub fn get_market_outlook_prompt() -> Result<JinjaTemplate> {
    JinjaTemplate::bilingual(
        "stock.user.get_market_outlook",
        "Based on current macroeconomic conditions, provide a market outlook including which sectors may benefit or be hurt.",
        "基于当前宏观经济条件，提供市场展望，包括哪些板块可能受益或受损。",
    )
}

/// Create the analyze impact user message template
pub fn analyze_impact_prompt() -> Result<JinjaTemplate> {
    JinjaTemplate::bilingual(
        "stock.user.analyze_impact",
        "Analyze the impact of current macroeconomic conditions and policy environment on {{ subject }}.",
        "分析当前宏观经济形势和政策环境对 {{ subject }} 的影响。",
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use agent_prompt::{Language, PromptTemplate};
    use serde_json::json;

    #[test]
    fn test_all_user_prompts_created() {
        // Earnings prompts
        assert!(analyze_earnings_prompt().is_ok());
        assert!(compare_earnings_prompt().is_ok());
        assert!(analyze_quality_prompt().is_ok());

        // Macro prompts
        assert!(analyze_economy_prompt().is_ok());
        assert!(analyze_fed_policy_prompt().is_ok());
        assert!(analyze_rates_prompt().is_ok());
        assert!(analyze_inflation_prompt().is_ok());
        assert!(analyze_geopolitical_risks_prompt().is_ok());
        assert!(get_market_outlook_prompt().is_ok());
        assert!(analyze_impact_prompt().is_ok());
    }

    #[test]
    fn test_earnings_prompt_render() {
        let template = analyze_earnings_prompt().unwrap();

        let en = template
            .render(&Language::English, &json!({ "symbol": "AAPL" }))
            .unwrap();
        assert!(en.contains("AAPL"));
        assert!(en.contains("financial reports"));

        let zh = template
            .render(&Language::Chinese, &json!({ "symbol": "AAPL" }))
            .unwrap();
        assert!(zh.contains("AAPL"));
        assert!(zh.contains("财务报告"));
    }

    #[test]
    fn test_compare_earnings_render() {
        let template = compare_earnings_prompt().unwrap();

        let en = template
            .render(&Language::English, &json!({ "symbol": "MSFT", "periods": 4 }))
            .unwrap();
        assert!(en.contains("MSFT"));
        assert!(en.contains("4"));

        let zh = template
            .render(&Language::Chinese, &json!({ "symbol": "MSFT", "periods": 4 }))
            .unwrap();
        assert!(zh.contains("MSFT"));
        assert!(zh.contains("4"));
    }

    #[test]
    fn test_analyze_impact_render() {
        let template = analyze_impact_prompt().unwrap();

        let en = template
            .render(&Language::English, &json!({ "subject": "technology sector" }))
            .unwrap();
        assert!(en.contains("technology sector"));

        let zh = template
            .render(&Language::Chinese, &json!({ "subject": "科技板块" }))
            .unwrap();
        assert!(zh.contains("科技板块"));
    }
}
