//! Tool for analyzing geopolitical events and their market impact
//!
//! Aggregates news from multiple sources to assess geopolitical risks

use agent_core::Result as AgentResult;
use agent_tools::Tool;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::sync::Arc;

use crate::api::{FinnhubClient, AlphaVantageClient};
use crate::cache::{CacheKey, StockCache};
use crate::config::StockConfig;
use crate::error::Result;

/// Geopolitical topic categories
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GeopoliticalTopic {
    /// US-China relations and trade
    UsChinaRelations,
    /// Trade policies and tariffs
    TradePolicies,
    /// Economic sanctions
    Sanctions,
    /// Middle East situation
    MiddleEast,
    /// European Union policies
    EuropeanUnion,
    /// Emerging markets
    EmergingMarkets,
    /// Currency and monetary policies
    CurrencyPolicies,
    /// Global supply chain
    SupplyChain,
    /// Federal Reserve and central banks
    CentralBanks,
    /// General market news
    General,
}

impl GeopoliticalTopic {
    /// Get search keywords for this topic
    pub fn keywords(&self) -> Vec<&'static str> {
        match self {
            GeopoliticalTopic::UsChinaRelations => vec!["china", "us china", "tariff", "trade war", "decoupling"],
            GeopoliticalTopic::TradePolicies => vec!["trade", "tariff", "import", "export", "trade agreement"],
            GeopoliticalTopic::Sanctions => vec!["sanction", "embargo", "restriction", "ban"],
            GeopoliticalTopic::MiddleEast => vec!["middle east", "oil", "opec", "israel", "iran", "saudi"],
            GeopoliticalTopic::EuropeanUnion => vec!["eu", "europe", "ecb", "euro", "brexit"],
            GeopoliticalTopic::EmergingMarkets => vec!["emerging market", "developing", "brics"],
            GeopoliticalTopic::CurrencyPolicies => vec!["dollar", "currency", "forex", "exchange rate", "yen"],
            GeopoliticalTopic::SupplyChain => vec!["supply chain", "semiconductor", "shortage", "logistics"],
            GeopoliticalTopic::CentralBanks => vec!["fed", "federal reserve", "central bank", "interest rate", "monetary policy"],
            GeopoliticalTopic::General => vec!["market", "economy", "global"],
        }
    }

    /// Get topic display name
    pub fn name(&self) -> &'static str {
        match self {
            GeopoliticalTopic::UsChinaRelations => "US-China Relations",
            GeopoliticalTopic::TradePolicies => "Trade Policies",
            GeopoliticalTopic::Sanctions => "Sanctions",
            GeopoliticalTopic::MiddleEast => "Middle East",
            GeopoliticalTopic::EuropeanUnion => "European Union",
            GeopoliticalTopic::EmergingMarkets => "Emerging Markets",
            GeopoliticalTopic::CurrencyPolicies => "Currency Policies",
            GeopoliticalTopic::SupplyChain => "Supply Chain",
            GeopoliticalTopic::CentralBanks => "Central Banks",
            GeopoliticalTopic::General => "General Market",
        }
    }

    /// Get affected sectors for this topic
    pub fn affected_sectors(&self) -> Vec<&'static str> {
        match self {
            GeopoliticalTopic::UsChinaRelations => vec!["Technology", "Industrials", "Consumer Discretionary"],
            GeopoliticalTopic::TradePolicies => vec!["Industrials", "Materials", "Consumer Discretionary"],
            GeopoliticalTopic::Sanctions => vec!["Energy", "Financials", "Technology"],
            GeopoliticalTopic::MiddleEast => vec!["Energy", "Utilities", "Industrials"],
            GeopoliticalTopic::EuropeanUnion => vec!["Financials", "Industrials", "Consumer Staples"],
            GeopoliticalTopic::EmergingMarkets => vec!["Financials", "Materials", "Consumer Discretionary"],
            GeopoliticalTopic::CurrencyPolicies => vec!["Financials", "Industrials", "Technology"],
            GeopoliticalTopic::SupplyChain => vec!["Technology", "Industrials", "Consumer Discretionary"],
            GeopoliticalTopic::CentralBanks => vec!["Financials", "Real Estate", "Utilities"],
            GeopoliticalTopic::General => vec!["All Sectors"],
        }
    }

    /// Parse topic from string
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "us-china" | "us china" | "china" => Some(GeopoliticalTopic::UsChinaRelations),
            "trade" | "tariff" | "trade policy" => Some(GeopoliticalTopic::TradePolicies),
            "sanction" | "sanctions" => Some(GeopoliticalTopic::Sanctions),
            "middle east" | "middleeast" | "oil" | "opec" => Some(GeopoliticalTopic::MiddleEast),
            "eu" | "europe" | "european union" => Some(GeopoliticalTopic::EuropeanUnion),
            "emerging" | "emerging markets" | "em" => Some(GeopoliticalTopic::EmergingMarkets),
            "currency" | "forex" | "dollar" => Some(GeopoliticalTopic::CurrencyPolicies),
            "supply chain" | "supplychain" | "semiconductor" => Some(GeopoliticalTopic::SupplyChain),
            "fed" | "central bank" | "interest rate" => Some(GeopoliticalTopic::CentralBanks),
            _ => Some(GeopoliticalTopic::General),
        }
    }

    /// Get all topics
    pub fn all() -> Vec<GeopoliticalTopic> {
        vec![
            GeopoliticalTopic::UsChinaRelations,
            GeopoliticalTopic::TradePolicies,
            GeopoliticalTopic::Sanctions,
            GeopoliticalTopic::MiddleEast,
            GeopoliticalTopic::EuropeanUnion,
            GeopoliticalTopic::EmergingMarkets,
            GeopoliticalTopic::CurrencyPolicies,
            GeopoliticalTopic::SupplyChain,
            GeopoliticalTopic::CentralBanks,
        ]
    }
}

/// Geopolitical event/news item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeopoliticalEvent {
    pub title: String,
    pub summary: String,
    pub source: String,
    pub published_at: String,
    pub topic: String,
    pub sentiment: String,
    pub impact_level: String,
    pub affected_sectors: Vec<String>,
    pub url: Option<String>,
}

/// Geopolitical risk assessment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskAssessment {
    pub topic: String,
    pub risk_level: String,
    pub recent_events_count: usize,
    pub sentiment_distribution: SentimentDistribution,
    pub key_developments: Vec<String>,
    pub market_implications: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SentimentDistribution {
    pub positive: usize,
    pub negative: usize,
    pub neutral: usize,
}

/// Parameters for geopolitical analysis
#[derive(Debug, Deserialize)]
struct GeopoliticalParams {
    /// Specific topic to analyze (optional)
    topic: Option<String>,
    /// Type of analysis: "news", "risk", "overview"
    #[serde(default = "default_analysis_type")]
    analysis_type: String,
    /// Number of news items to fetch
    #[serde(default = "default_limit")]
    limit: usize,
}

fn default_analysis_type() -> String {
    "overview".to_string()
}

fn default_limit() -> usize {
    10
}

/// Tool for geopolitical analysis
pub struct GeopoliticalTool {
    finnhub_client: Option<FinnhubClient>,
    alpha_vantage_client: Option<AlphaVantageClient>,
    cache: StockCache,
    _config: Arc<StockConfig>,
}

impl GeopoliticalTool {
    /// Create a new geopolitical analysis tool
    pub fn new(config: Arc<StockConfig>, cache: StockCache) -> Self {
        let finnhub_client = config.finnhub_api_key.as_ref().map(|key| {
            FinnhubClient::new(key.clone(), 60)
        });

        let alpha_vantage_client = config.alpha_vantage_api_key.as_ref().map(|key| {
            AlphaVantageClient::new(key.clone(), config.alpha_vantage_rate_limit)
        });

        Self {
            finnhub_client,
            alpha_vantage_client,
            cache,
            _config: config,
        }
    }

    /// Fetch geopolitical analysis data
    async fn fetch_geopolitical_data(&self, params: GeopoliticalParams) -> Result<Value> {
        // Create cache key
        let cache_key = CacheKey::new(
            "geopolitical",
            &params.analysis_type,
            json!({
                "topic": params.topic,
                "limit": params.limit
            }),
        );

        // Try to get from cache
        self.cache
            .get_or_fetch(cache_key, || async {
                self.analyze_geopolitics(&params).await
            })
            .await
    }

    /// Analyze geopolitical situation
    async fn analyze_geopolitics(&self, params: &GeopoliticalParams) -> Result<Value> {
        match params.analysis_type.to_lowercase().as_str() {
            "news" => {
                let topic = params.topic.as_ref().map(|t| GeopoliticalTopic::from_str(t)).flatten();
                self.fetch_geopolitical_news(topic, params.limit).await
            }
            "risk" => self.assess_geopolitical_risks().await,
            "overview" | _ => self.get_geopolitical_overview(params.limit).await,
        }
    }

    /// Fetch news related to geopolitical topics
    async fn fetch_geopolitical_news(
        &self,
        topic: Option<GeopoliticalTopic>,
        limit: usize,
    ) -> Result<Value> {
        let news = self.get_market_news("general", limit).await?;

        // Filter and categorize news by topic
        let categorized = self.categorize_news(&news, topic);

        let topic_name = topic.map(|t| t.name()).unwrap_or("All Topics");

        Ok(json!({
            "type": "geopolitical_news",
            "topic": topic_name,
            "news_count": categorized.len(),
            "articles": categorized,
            "affected_sectors": topic.map(|t| t.affected_sectors()).unwrap_or_default(),
            "as_of_date": chrono::Utc::now().format("%Y-%m-%d %H:%M UTC").to_string(),
        }))
    }

    /// Get market news from available providers
    async fn get_market_news(&self, category: &str, limit: usize) -> Result<Vec<Value>> {
        // Try Finnhub first
        if let Some(ref client) = self.finnhub_client {
            let articles = client.get_market_news(category).await?;
            return Ok(articles
                .into_iter()
                .take(limit)
                .map(|a| {
                    json!({
                        "title": a.headline,
                        "summary": a.summary,
                        "source": a.source,
                        "published_at": chrono::DateTime::from_timestamp(a.datetime, 0)
                            .map(|dt| dt.to_rfc3339())
                            .unwrap_or_default(),
                        "url": a.url,
                        "category": a.category,
                    })
                })
                .collect());
        }

        // Fall back to mock data if no API configured
        Ok(self.get_mock_geopolitical_news(limit))
    }

    /// Categorize news by geopolitical topic
    fn categorize_news(&self, news: &[Value], filter_topic: Option<GeopoliticalTopic>) -> Vec<Value> {
        news.iter()
            .filter_map(|article| {
                let title = article.get("title")?.as_str()?;
                let summary = article.get("summary").and_then(|s| s.as_str()).unwrap_or("");
                let content = format!("{} {}", title, summary).to_lowercase();

                // Identify topic
                let topic = self.identify_topic(&content);
                
                // Filter by topic if specified
                if let Some(filter) = filter_topic {
                    if topic != filter {
                        return None;
                    }
                }

                // Assess sentiment and impact
                let sentiment = self.assess_sentiment(&content);
                let impact = self.assess_impact(&content, &topic);

                Some(json!({
                    "title": title,
                    "summary": summary,
                    "source": article.get("source"),
                    "published_at": article.get("published_at"),
                    "url": article.get("url"),
                    "topic": topic.name(),
                    "sentiment": sentiment,
                    "impact_level": impact,
                    "affected_sectors": topic.affected_sectors(),
                }))
            })
            .collect()
    }

    /// Identify the geopolitical topic from content
    fn identify_topic(&self, content: &str) -> GeopoliticalTopic {
        for topic in GeopoliticalTopic::all() {
            for keyword in topic.keywords() {
                if content.contains(keyword) {
                    return topic;
                }
            }
        }
        GeopoliticalTopic::General
    }

    /// Assess sentiment from content
    fn assess_sentiment(&self, content: &str) -> String {
        let negative_words = ["crisis", "war", "conflict", "sanctions", "decline", "fear", 
                            "crash", "risk", "threat", "tension", "collapse", "recession"];
        let positive_words = ["growth", "deal", "agreement", "recovery", "boost", "rally",
                            "strong", "surge", "gain", "optimism", "breakthrough"];

        let negative_count = negative_words.iter().filter(|w| content.contains(*w)).count();
        let positive_count = positive_words.iter().filter(|w| content.contains(*w)).count();

        if negative_count > positive_count + 1 {
            "Negative".to_string()
        } else if positive_count > negative_count + 1 {
            "Positive".to_string()
        } else {
            "Neutral".to_string()
        }
    }

    /// Assess market impact level
    fn assess_impact(&self, content: &str, topic: &GeopoliticalTopic) -> String {
        let high_impact_words = ["major", "significant", "breaking", "unprecedented", 
                                "emergency", "crisis", "war", "collapse"];
        let medium_impact_words = ["important", "notable", "concern", "tension", "policy"];

        let has_high_impact = high_impact_words.iter().any(|w| content.contains(*w));
        let has_medium_impact = medium_impact_words.iter().any(|w| content.contains(*w));

        // Some topics are inherently higher impact
        let topic_weight = match topic {
            GeopoliticalTopic::MiddleEast | GeopoliticalTopic::UsChinaRelations => 1,
            GeopoliticalTopic::CentralBanks | GeopoliticalTopic::Sanctions => 1,
            _ => 0,
        };

        if has_high_impact || topic_weight > 0 && has_medium_impact {
            "High".to_string()
        } else if has_medium_impact {
            "Medium".to_string()
        } else {
            "Low".to_string()
        }
    }

    /// Assess geopolitical risks across all topics
    async fn assess_geopolitical_risks(&self) -> Result<Value> {
        let news = self.get_market_news("general", 50).await?;
        
        let mut risk_assessments = Vec::new();

        for topic in GeopoliticalTopic::all() {
            let topic_news: Vec<_> = news
                .iter()
                .filter(|a| {
                    let content = format!(
                        "{} {}",
                        a.get("title").and_then(|t| t.as_str()).unwrap_or(""),
                        a.get("summary").and_then(|s| s.as_str()).unwrap_or("")
                    ).to_lowercase();
                    
                    topic.keywords().iter().any(|k| content.contains(k))
                })
                .collect();

            if topic_news.is_empty() {
                continue;
            }

            // Count sentiments
            let mut positive = 0;
            let mut negative = 0;
            let mut neutral = 0;

            for article in &topic_news {
                let content = format!(
                    "{} {}",
                    article.get("title").and_then(|t| t.as_str()).unwrap_or(""),
                    article.get("summary").and_then(|s| s.as_str()).unwrap_or("")
                ).to_lowercase();

                match self.assess_sentiment(&content).as_str() {
                    "Positive" => positive += 1,
                    "Negative" => negative += 1,
                    _ => neutral += 1,
                }
            }

            // Determine risk level
            let risk_level = if negative > positive * 2 {
                "High"
            } else if negative > positive {
                "Elevated"
            } else if positive > negative {
                "Low"
            } else {
                "Moderate"
            };

            let key_developments: Vec<_> = topic_news
                .iter()
                .take(3)
                .filter_map(|a| a.get("title").and_then(|t| t.as_str()))
                .map(|s| s.to_string())
                .collect();

            let market_implications = self.get_market_implications(&topic, risk_level);

            risk_assessments.push(json!({
                "topic": topic.name(),
                "risk_level": risk_level,
                "recent_events_count": topic_news.len(),
                "sentiment_distribution": {
                    "positive": positive,
                    "negative": negative,
                    "neutral": neutral,
                },
                "key_developments": key_developments,
                "affected_sectors": topic.affected_sectors(),
                "market_implications": market_implications,
            }));
        }

        // Sort by risk level
        risk_assessments.sort_by(|a, b| {
            let risk_order = |r: &str| match r {
                "High" => 0,
                "Elevated" => 1,
                "Moderate" => 2,
                "Low" => 3,
                _ => 4,
            };
            let a_risk = a.get("risk_level").and_then(|r| r.as_str()).unwrap_or("Low");
            let b_risk = b.get("risk_level").and_then(|r| r.as_str()).unwrap_or("Low");
            risk_order(a_risk).cmp(&risk_order(b_risk))
        });

        // Overall risk assessment
        let high_risk_count = risk_assessments
            .iter()
            .filter(|a| a.get("risk_level").and_then(|r| r.as_str()) == Some("High"))
            .count();

        let overall_risk = if high_risk_count >= 2 {
            "Elevated - Multiple high-risk areas"
        } else if high_risk_count == 1 {
            "Moderate - One significant risk area"
        } else {
            "Low - No major geopolitical concerns"
        };

        Ok(json!({
            "type": "geopolitical_risk_assessment",
            "overall_risk": overall_risk,
            "risk_areas": risk_assessments,
            "as_of_date": chrono::Utc::now().format("%Y-%m-%d %H:%M UTC").to_string(),
        }))
    }

    /// Get market implications for a topic and risk level
    fn get_market_implications(&self, topic: &GeopoliticalTopic, risk_level: &str) -> Vec<String> {
        let base_implications = match topic {
            GeopoliticalTopic::UsChinaRelations => vec![
                "Tech sector volatility on trade news",
                "Supply chain concerns for manufacturing",
                "Semiconductor stocks sensitive to policy changes",
            ],
            GeopoliticalTopic::MiddleEast => vec![
                "Oil price sensitivity",
                "Energy sector volatility",
                "Safe-haven flows to gold/bonds",
            ],
            GeopoliticalTopic::CentralBanks => vec![
                "Rate-sensitive sectors react to policy signals",
                "Bank stocks respond to yield curve changes",
                "Growth stocks sensitive to rate expectations",
            ],
            GeopoliticalTopic::SupplyChain => vec![
                "Inventory build-up may benefit logistics",
                "Reshoring beneficiaries in industrials",
                "Tech hardware may face component constraints",
            ],
            _ => vec![
                "Monitor for sector-specific impacts",
                "Consider hedging strategies",
            ],
        };

        let mut implications: Vec<String> = base_implications.iter().map(|s| s.to_string()).collect();

        if risk_level == "High" {
            implications.push("Consider reducing position size".to_string());
            implications.push("Volatility hedges may be warranted".to_string());
        }

        implications
    }

    /// Get comprehensive geopolitical overview
    async fn get_geopolitical_overview(&self, limit: usize) -> Result<Value> {
        let news = self.get_market_news("general", limit * 2).await?;
        let categorized = self.categorize_news(&news, None);

        // Group by topic
        let mut topic_groups: std::collections::HashMap<String, Vec<&Value>> = std::collections::HashMap::new();
        for article in &categorized {
            if let Some(topic) = article.get("topic").and_then(|t| t.as_str()) {
                topic_groups.entry(topic.to_string()).or_default().push(article);
            }
        }

        // Build topic summaries
        let topic_summaries: Vec<Value> = topic_groups
            .iter()
            .map(|(topic, articles)| {
                let sentiments: Vec<_> = articles
                    .iter()
                    .filter_map(|a| a.get("sentiment").and_then(|s| s.as_str()))
                    .collect();
                
                let negative_pct = sentiments.iter().filter(|&&s| s == "Negative").count() as f64 
                    / sentiments.len().max(1) as f64 * 100.0;

                json!({
                    "topic": topic,
                    "article_count": articles.len(),
                    "negative_sentiment_pct": negative_pct,
                    "top_headline": articles.first().and_then(|a| a.get("title")),
                })
            })
            .collect();

        // Overall market sentiment
        let total_negative = categorized
            .iter()
            .filter(|a| a.get("sentiment").and_then(|s| s.as_str()) == Some("Negative"))
            .count();
        
        let market_mood = if total_negative as f64 / categorized.len().max(1) as f64 > 0.5 {
            "Risk-off - Caution warranted"
        } else if total_negative as f64 / categorized.len().max(1) as f64 > 0.3 {
            "Mixed - Selective opportunities"
        } else {
            "Risk-on - Favorable backdrop"
        };

        Ok(json!({
            "type": "geopolitical_overview",
            "market_mood": market_mood,
            "topic_summaries": topic_summaries,
            "recent_articles": categorized.into_iter().take(limit).collect::<Vec<_>>(),
            "as_of_date": chrono::Utc::now().format("%Y-%m-%d %H:%M UTC").to_string(),
        }))
    }

    /// Get mock geopolitical news for testing
    fn get_mock_geopolitical_news(&self, limit: usize) -> Vec<Value> {
        let mock_news = vec![
            json!({
                "title": "Federal Reserve Signals Patience on Rate Cuts",
                "summary": "Fed officials indicate they need more evidence of cooling inflation before cutting rates",
                "source": "Financial News",
                "published_at": chrono::Utc::now().to_rfc3339(),
                "category": "general"
            }),
            json!({
                "title": "US-China Trade Talks Resume Amid Tech Tensions",
                "summary": "Officials from both countries meet to discuss trade issues and technology restrictions",
                "source": "Market Watch",
                "published_at": chrono::Utc::now().to_rfc3339(),
                "category": "general"
            }),
            json!({
                "title": "Oil Prices Rise on Middle East Supply Concerns",
                "summary": "Crude oil gains as tensions in the Middle East raise supply disruption fears",
                "source": "Energy News",
                "published_at": chrono::Utc::now().to_rfc3339(),
                "category": "general"
            }),
            json!({
                "title": "European Central Bank Holds Rates Steady",
                "summary": "ECB maintains current policy stance while monitoring economic conditions",
                "source": "European Markets",
                "published_at": chrono::Utc::now().to_rfc3339(),
                "category": "general"
            }),
        ];

        mock_news.into_iter().take(limit).collect()
    }
}

#[async_trait]
impl Tool for GeopoliticalTool {
    async fn execute(&self, params: Value) -> AgentResult<Value> {
        let params: GeopoliticalParams = serde_json::from_value(params).map_err(|e| {
            agent_core::Error::ProcessingFailed(format!("Invalid parameters: {}", e))
        })?;

        self.fetch_geopolitical_data(params)
            .await
            .map_err(|e| agent_core::Error::ProcessingFailed(e.to_string()))
    }

    fn name(&self) -> &str {
        "geopolitical"
    }

    fn description(&self) -> &str {
        "Analyze geopolitical events and their market impact. \
         Tracks US-China relations, trade policies, sanctions, Middle East situation, \
         central bank policies, supply chain issues, and more. \
         Provides risk assessments and market implications for each topic."
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "topic": {
                    "type": "string",
                    "description": "Specific topic: 'us-china', 'trade', 'sanctions', 'middle east', 'eu', 'emerging', 'currency', 'supply chain', 'fed'",
                },
                "analysis_type": {
                    "type": "string",
                    "enum": ["news", "risk", "overview"],
                    "description": "Type of analysis: news articles, risk assessment, or overview",
                    "default": "overview"
                },
                "limit": {
                    "type": "integer",
                    "description": "Number of news items to fetch",
                    "default": 10,
                    "minimum": 1,
                    "maximum": 50
                }
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_topic_keywords() {
        assert!(GeopoliticalTopic::UsChinaRelations.keywords().contains(&"china"));
        assert!(GeopoliticalTopic::MiddleEast.keywords().contains(&"oil"));
        assert!(GeopoliticalTopic::CentralBanks.keywords().contains(&"fed"));
    }

    #[test]
    fn test_topic_from_str() {
        assert_eq!(GeopoliticalTopic::from_str("china"), Some(GeopoliticalTopic::UsChinaRelations));
        assert_eq!(GeopoliticalTopic::from_str("fed"), Some(GeopoliticalTopic::CentralBanks));
        assert_eq!(GeopoliticalTopic::from_str("oil"), Some(GeopoliticalTopic::MiddleEast));
    }

    #[test]
    fn test_affected_sectors() {
        assert!(GeopoliticalTopic::UsChinaRelations.affected_sectors().contains(&"Technology"));
        assert!(GeopoliticalTopic::MiddleEast.affected_sectors().contains(&"Energy"));
        assert!(GeopoliticalTopic::CentralBanks.affected_sectors().contains(&"Financials"));
    }

    #[test]
    fn test_tool_metadata() {
        let config = Arc::new(StockConfig::default());
        let cache = StockCache::new(Duration::from_secs(900));
        let tool = GeopoliticalTool::new(config, cache);

        assert_eq!(tool.name(), "geopolitical");
        assert!(tool.description().contains("geopolitical"));
    }
}
