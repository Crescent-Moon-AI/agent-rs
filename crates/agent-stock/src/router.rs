//! Smart router for directing queries to appropriate agents
//!
//! This module provides intelligent routing based on query intent analysis,
//! supporting both rule-based and keyword-based routing strategies.

use std::collections::HashSet;

/// Intent types that can be detected from user queries
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum QueryIntent {
    /// Get current price or quote data
    PriceQuery,
    /// Technical analysis (RSI, MACD, etc.)
    TechnicalAnalysis,
    /// Fundamental analysis (P/E, market cap, etc.)
    FundamentalAnalysis,
    /// News and sentiment analysis
    NewsAnalysis,
    /// Earnings and financial reports
    EarningsAnalysis,
    /// Macroeconomic analysis
    MacroAnalysis,
    /// Geopolitical analysis
    GeopoliticalAnalysis,
    /// Comprehensive analysis (multiple agents)
    ComprehensiveAnalysis,
    /// Stock comparison
    Comparison,
    /// General query or unknown intent
    General,
}

impl QueryIntent {
    /// Get the corresponding agent name for this intent
    pub fn agent_name(&self) -> &'static str {
        match self {
            Self::PriceQuery => "data-fetcher",
            Self::TechnicalAnalysis => "technical-analyzer",
            Self::FundamentalAnalysis => "fundamental-analyzer",
            Self::NewsAnalysis => "news-analyzer",
            Self::EarningsAnalysis => "earnings-analyzer",
            Self::MacroAnalysis | Self::GeopoliticalAnalysis => "macro-analyzer",
            Self::ComprehensiveAnalysis | Self::Comparison | Self::General => "technical-analyzer",
        }
    }

    /// Check if this intent requires multiple agents
    pub fn requires_multiple_agents(&self) -> bool {
        matches!(self, Self::ComprehensiveAnalysis | Self::Comparison)
    }

    /// Get all agents needed for comprehensive analysis
    pub fn comprehensive_agents() -> Vec<&'static str> {
        vec![
            "data-fetcher",
            "technical-analyzer",
            "fundamental-analyzer",
            "news-analyzer",
            "earnings-analyzer",
            "macro-analyzer",
        ]
    }
}

/// Keywords for intent classification (English)
mod keywords_en {
    pub const PRICE: &[&str] = &[
        "price",
        "quote",
        "current",
        "latest",
        "stock price",
        "how much",
        "cost",
    ];

    pub const TECHNICAL: &[&str] = &[
        "technical",
        "rsi",
        "macd",
        "moving average",
        "sma",
        "ema",
        "bollinger",
        "indicator",
        "chart",
        "pattern",
        "trend",
        "support",
        "resistance",
        "momentum",
        "volume",
        "atr",
        "stochastic",
    ];

    pub const FUNDAMENTAL: &[&str] = &[
        "fundamental",
        "p/e",
        "pe ratio",
        "valuation",
        "market cap",
        "dividend",
        "eps",
        "earnings per share",
        "book value",
        "revenue",
        "profit",
        "margin",
        "debt",
        "assets",
    ];

    pub const NEWS: &[&str] = &[
        "news",
        "sentiment",
        "event",
        "headline",
        "announcement",
        "recent",
        "latest news",
    ];

    pub const EARNINGS: &[&str] = &[
        "earnings",
        "10-k",
        "10-q",
        "quarterly",
        "annual report",
        "financial report",
        "sec filing",
        "income statement",
        "balance sheet",
        "cash flow",
    ];

    pub const MACRO: &[&str] = &[
        "fed",
        "federal reserve",
        "interest rate",
        "inflation",
        "gdp",
        "unemployment",
        "macro",
        "economy",
        "economic",
        "cpi",
        "pce",
        "yield curve",
    ];

    pub const GEOPOLITICAL: &[&str] = &[
        "geopolitical",
        "trade war",
        "sanctions",
        "tariff",
        "political",
        "war",
        "conflict",
        "international",
    ];

    pub const COMPREHENSIVE: &[&str] = &[
        "comprehensive",
        "full analysis",
        "complete",
        "overall",
        "all",
        "detailed",
        "in-depth",
        "thorough",
    ];

    pub const COMPARISON: &[&str] = &["compare", "comparison", "versus", "vs", "better", "which"];
}

/// Keywords for intent classification (Chinese)
mod keywords_zh {
    pub const PRICE: &[&str] = &["价格", "股价", "报价", "多少钱", "现价", "最新价"];

    pub const TECHNICAL: &[&str] = &[
        "技术分析",
        "技术指标",
        "均线",
        "移动平均",
        "布林带",
        "趋势",
        "支撑",
        "阻力",
        "动量",
        "成交量",
    ];

    pub const FUNDAMENTAL: &[&str] = &[
        "基本面",
        "市盈率",
        "估值",
        "市值",
        "股息",
        "每股收益",
        "营收",
        "利润",
        "利润率",
        "负债",
    ];

    pub const NEWS: &[&str] = &["新闻", "消息", "情绪", "舆情", "公告", "最新消息"];

    pub const EARNINGS: &[&str] = &[
        "财报", "季报", "年报", "财务报告", "盈利", "业绩", "财务报表", "收益报告",
    ];

    pub const MACRO: &[&str] = &[
        "美联储",
        "利率",
        "通胀",
        "经济",
        "宏观",
        "失业率",
        "加息",
        "降息",
        "货币政策",
    ];

    pub const GEOPOLITICAL: &[&str] = &[
        "地缘政治",
        "贸易战",
        "制裁",
        "关税",
        "国际形势",
        "冲突",
        "战争",
    ];

    pub const COMPREHENSIVE: &[&str] = &[
        "综合分析",
        "全面分析",
        "详细分析",
        "完整",
        "深入分析",
        "全方位",
    ];

    pub const COMPARISON: &[&str] = &["比较", "对比", "哪个好", "哪只"];
}

/// Smart router for query intent classification
#[derive(Debug, Clone)]
pub struct SmartRouter {
    /// Enable debug logging
    debug: bool,
}

impl Default for SmartRouter {
    fn default() -> Self {
        Self::new()
    }
}

impl SmartRouter {
    /// Create a new smart router
    pub fn new() -> Self {
        Self { debug: false }
    }

    /// Enable debug mode
    pub fn with_debug(mut self, debug: bool) -> Self {
        self.debug = debug;
        self
    }

    /// Classify the intent of a query
    pub fn classify(&self, query: &str) -> QueryIntent {
        let query_lower = query.to_lowercase();
        let intents = self.detect_all_intents(&query_lower);

        if self.debug {
            tracing::debug!("Detected intents for query: {:?}", intents);
        }

        // Priority-based intent selection
        if intents.contains(&QueryIntent::ComprehensiveAnalysis) || intents.len() > 2 {
            return QueryIntent::ComprehensiveAnalysis;
        }

        if intents.contains(&QueryIntent::Comparison) {
            return QueryIntent::Comparison;
        }

        // Return the first detected intent, or General if none
        intents.into_iter().next().unwrap_or(QueryIntent::General)
    }

    /// Detect all matching intents from a query
    fn detect_all_intents(&self, query: &str) -> HashSet<QueryIntent> {
        let mut intents = HashSet::new();

        // Check English keywords
        if Self::matches_any(query, keywords_en::PRICE) {
            intents.insert(QueryIntent::PriceQuery);
        }
        if Self::matches_any(query, keywords_en::TECHNICAL) {
            intents.insert(QueryIntent::TechnicalAnalysis);
        }
        if Self::matches_any(query, keywords_en::FUNDAMENTAL) {
            intents.insert(QueryIntent::FundamentalAnalysis);
        }
        if Self::matches_any(query, keywords_en::NEWS) {
            intents.insert(QueryIntent::NewsAnalysis);
        }
        if Self::matches_any(query, keywords_en::EARNINGS) {
            intents.insert(QueryIntent::EarningsAnalysis);
        }
        if Self::matches_any(query, keywords_en::MACRO) {
            intents.insert(QueryIntent::MacroAnalysis);
        }
        if Self::matches_any(query, keywords_en::GEOPOLITICAL) {
            intents.insert(QueryIntent::GeopoliticalAnalysis);
        }
        if Self::matches_any(query, keywords_en::COMPREHENSIVE) {
            intents.insert(QueryIntent::ComprehensiveAnalysis);
        }
        if Self::matches_any(query, keywords_en::COMPARISON) {
            intents.insert(QueryIntent::Comparison);
        }

        // Check Chinese keywords
        if Self::matches_any(query, keywords_zh::PRICE) {
            intents.insert(QueryIntent::PriceQuery);
        }
        if Self::matches_any(query, keywords_zh::TECHNICAL) {
            intents.insert(QueryIntent::TechnicalAnalysis);
        }
        if Self::matches_any(query, keywords_zh::FUNDAMENTAL) {
            intents.insert(QueryIntent::FundamentalAnalysis);
        }
        if Self::matches_any(query, keywords_zh::NEWS) {
            intents.insert(QueryIntent::NewsAnalysis);
        }
        if Self::matches_any(query, keywords_zh::EARNINGS) {
            intents.insert(QueryIntent::EarningsAnalysis);
        }
        if Self::matches_any(query, keywords_zh::MACRO) {
            intents.insert(QueryIntent::MacroAnalysis);
        }
        if Self::matches_any(query, keywords_zh::GEOPOLITICAL) {
            intents.insert(QueryIntent::GeopoliticalAnalysis);
        }
        if Self::matches_any(query, keywords_zh::COMPREHENSIVE) {
            intents.insert(QueryIntent::ComprehensiveAnalysis);
        }
        if Self::matches_any(query, keywords_zh::COMPARISON) {
            intents.insert(QueryIntent::Comparison);
        }

        intents
    }

    /// Check if query contains any of the keywords
    fn matches_any(query: &str, keywords: &[&str]) -> bool {
        keywords.iter().any(|kw| query.contains(kw))
    }

    /// Get agents to invoke based on intent
    pub fn get_agents(&self, intent: QueryIntent) -> Vec<&'static str> {
        if intent.requires_multiple_agents() {
            QueryIntent::comprehensive_agents()
        } else {
            vec![intent.agent_name()]
        }
    }

    /// Extract stock symbols from a query
    pub fn extract_symbols(&self, query: &str) -> Vec<String> {
        let mut symbols = Vec::new();

        // Common patterns for stock symbols
        // 1. Explicit mentions like "AAPL", "GOOGL", etc. (uppercase, 1-5 letters)
        // 2. After keywords like "analyze", "分析", etc.

        for word in query.split_whitespace() {
            let clean_word = word.trim_matches(|c: char| !c.is_alphanumeric());

            // Check if it looks like a US stock symbol (1-5 uppercase letters)
            if clean_word.len() >= 1
                && clean_word.len() <= 5
                && clean_word.chars().all(|c| c.is_ascii_uppercase())
            {
                symbols.push(clean_word.to_string());
            }
        }

        // Remove duplicates
        symbols.sort();
        symbols.dedup();
        symbols
    }
}

/// Result of routing a query
#[derive(Debug, Clone)]
pub struct RoutingResult {
    /// The detected intent
    pub intent: QueryIntent,
    /// Agents to invoke
    pub agents: Vec<String>,
    /// Extracted stock symbols
    pub symbols: Vec<String>,
    /// Whether this requires parallel execution
    pub parallel: bool,
}

impl SmartRouter {
    /// Route a query and return the full routing result
    pub fn route(&self, query: &str) -> RoutingResult {
        let intent = self.classify(query);
        let agents = self.get_agents(intent);
        let symbols = self.extract_symbols(query);

        RoutingResult {
            intent,
            agents: agents.iter().map(|s| s.to_string()).collect(),
            symbols,
            parallel: intent.requires_multiple_agents(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_price_query_detection() {
        let router = SmartRouter::new();

        assert_eq!(
            router.classify("What is the price of AAPL?"),
            QueryIntent::PriceQuery
        );
        assert_eq!(router.classify("AAPL 股价多少?"), QueryIntent::PriceQuery);
    }

    #[test]
    fn test_technical_analysis_detection() {
        let router = SmartRouter::new();

        assert_eq!(
            router.classify("Calculate RSI for AAPL"),
            QueryIntent::TechnicalAnalysis
        );
        assert_eq!(
            router.classify("TSLA的技术分析"),
            QueryIntent::TechnicalAnalysis
        );
        assert_eq!(
            router.classify("Show me the MACD indicator"),
            QueryIntent::TechnicalAnalysis
        );
    }

    #[test]
    fn test_fundamental_analysis_detection() {
        let router = SmartRouter::new();

        assert_eq!(
            router.classify("What is the P/E ratio of MSFT?"),
            QueryIntent::FundamentalAnalysis
        );
        assert_eq!(
            router.classify("分析AAPL的基本面"),
            QueryIntent::FundamentalAnalysis
        );
    }

    #[test]
    fn test_comprehensive_analysis_detection() {
        let router = SmartRouter::new();

        assert_eq!(
            router.classify("Give me a comprehensive analysis of AAPL"),
            QueryIntent::ComprehensiveAnalysis
        );
        assert_eq!(
            router.classify("全面分析特斯拉"),
            QueryIntent::ComprehensiveAnalysis
        );
    }

    #[test]
    fn test_comparison_detection() {
        let router = SmartRouter::new();

        assert_eq!(
            router.classify("Compare AAPL and GOOGL"),
            QueryIntent::Comparison
        );
        assert_eq!(
            router.classify("比较苹果和微软"),
            QueryIntent::Comparison
        );
    }

    #[test]
    fn test_symbol_extraction() {
        let router = SmartRouter::new();

        let symbols = router.extract_symbols("Analyze AAPL and GOOGL");
        assert!(symbols.contains(&"AAPL".to_string()));
        assert!(symbols.contains(&"GOOGL".to_string()));

        let symbols = router.extract_symbols("What about MSFT?");
        assert!(symbols.contains(&"MSFT".to_string()));
    }

    #[test]
    fn test_routing_result() {
        let router = SmartRouter::new();

        let result = router.route("Comprehensive analysis of AAPL");
        assert_eq!(result.intent, QueryIntent::ComprehensiveAnalysis);
        assert!(result.parallel);
        assert!(result.agents.len() > 1);
    }

    #[test]
    fn test_agent_mapping() {
        assert_eq!(QueryIntent::PriceQuery.agent_name(), "data-fetcher");
        assert_eq!(
            QueryIntent::TechnicalAnalysis.agent_name(),
            "technical-analyzer"
        );
        assert_eq!(
            QueryIntent::FundamentalAnalysis.agent_name(),
            "fundamental-analyzer"
        );
    }
}
