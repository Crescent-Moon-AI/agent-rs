# agent-stock

Comprehensive stock market analysis framework using multi-agent LLM architecture.

## Features

- **Multi-Agent Architecture**: Specialized agents for different analysis types
  - `DataFetcherAgent`: Retrieves stock prices and company data
  - `TechnicalAnalyzerAgent`: Performs technical analysis (RSI, MACD, Bollinger Bands, etc.)
  - `FundamentalAnalyzerAgent`: Analyzes company fundamentals (P/E, market cap, financials)
  - `NewsAnalyzerAgent`: Analyzes news and market sentiment
  - `EarningsAnalyzerAgent`: Analyzes SEC filings (10-K, 10-Q) and financial statements
  - `MacroAnalyzerAgent`: Analyzes macroeconomic conditions, Fed policy, and geopolitical risks
  - `StockAnalysisAgent`: Top-level delegating agent that coordinates specialists

- **70+ Technical Indicators**: Powered by `rust_ti` crate
  - RSI, MACD, SMA, EMA, Bollinger Bands, ATR, Stochastic, and more

- **Multiple Data Sources**:
  - Yahoo Finance (primary, no API key required)
  - Alpha Vantage (fundamental data and news sentiment)
  - SEC EDGAR (10-K, 10-Q filings and financial data)
  - FRED (Federal Reserve Economic Data for macro indicators)
  - Finnhub (market news)

- **Comprehensive Analysis Capabilities**:
  - Earnings report analysis from SEC filings
  - Macroeconomic indicators (Fed rates, inflation, GDP, unemployment)
  - Sector rotation analysis
  - Geopolitical risk assessment

- **Smart Caching**: Multi-tiered caching system reduces API calls
  - Real-time data: 60-second TTL
  - Fundamental data: 1-hour TTL
  - News data: 5-minute TTL
  - Earnings data: 24-hour TTL
  - Macro data: 1-hour TTL
  - Sector data: 30-minute TTL

- **Robust Error Handling**: Automatic retries with exponential backoff

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
agent-stock = "0.0.0"
agent-runtime = "0.0.0"
agent-llm = "0.0.0"
```

## Quick Start

```rust
use agent_stock::{StockAnalysisAgent, StockConfig};
use agent_runtime::AgentRuntime;
use agent_llm::providers::AnthropicProvider;
use std::sync::Arc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Configure
    let config = Arc::new(StockConfig::builder()
        .with_env_api_key()  // Load Alpha Vantage key from env
        .build()?);

    // Create LLM provider
    let provider = Arc::new(AnthropicProvider::new(
        std::env::var("ANTHROPIC_API_KEY")?
    )?);

    // Create runtime
    let runtime = Arc::new(AgentRuntime::builder()
        .provider(provider)
        .build()?);

    // Create stock analysis agent
    let agent = StockAnalysisAgent::new(runtime, config).await?;

    // Analyze a stock
    let analysis = agent.analyze("AAPL").await?;
    println!("{}", analysis);

    Ok(())
}
```

## Configuration

### Environment Variables

```bash
# Required for LLM
export ANTHROPIC_API_KEY=your_anthropic_key

# Optional - for fundamental data and news sentiment
export ALPHA_VANTAGE_API_KEY=your_alpha_vantage_key

# Optional - for market news
export FINNHUB_API_KEY=your_finnhub_key

# Optional - for macroeconomic data (FRED)
export FRED_API_KEY=your_fred_key

# Optional - configure response language (default is Chinese)
export STOCK_RESPONSE_LANGUAGE=chinese  # or: english, zh, en
```

### Configuration Builder

```rust
use agent_stock::{ResponseLanguage, StockConfig};
use std::time::Duration;

let config = StockConfig::builder()
    .cache_ttl_realtime(Duration::from_secs(60))
    .cache_ttl_fundamental(Duration::from_secs(3600))
    .max_retries(3)
    .response_language(ResponseLanguage::Chinese)  // Chinese (default) or English
    .with_env_api_key()
    .from_env_model()  // Load language settings from environment
    .build()?;
```

### Language Configuration

The agent can respond in Chinese (default) or English. Configure it via:

**Method 1: Environment Variable**
```bash
export STOCK_RESPONSE_LANGUAGE=chinese  # or: english, zh, en
```

**Method 2: Code Configuration**
```rust
use agent_stock::ResponseLanguage;

let config = StockConfig::builder()
    .response_language(ResponseLanguage::Chinese)  // or ResponseLanguage::English
    .build()?;
```

All agents (DataFetcher, TechnicalAnalyzer, FundamentalAnalyzer, NewsAnalyzer) will use the configured language for their responses.

## Architecture

### Multi-Agent System

The system uses a delegating agent pattern:

```
StockAnalysisAgent (Coordinator)
│
├── DataFetcherAgent
│   ├── StockDataTool
│   └── FundamentalDataTool
│
├── TechnicalAnalyzerAgent
│   ├── StockDataTool
│   ├── TechnicalIndicatorTool
│   └── ChartDataTool
│
├── FundamentalAnalyzerAgent
│   └── FundamentalDataTool
│
├── NewsAnalyzerAgent
│   └── NewsTool
│
├── EarningsAnalyzerAgent
│   └── EarningsReportTool (SEC EDGAR)
│
└── MacroAnalyzerAgent
    ├── MacroEconomicTool (FRED)
    ├── GeopoliticalTool
    └── SectorAnalysisTool
```

### Tools

Each tool is a self-contained unit that performs a specific task:

- **StockDataTool**: Fetch current quotes and historical prices
- **TechnicalIndicatorTool**: Calculate 70+ technical indicators
- **FundamentalDataTool**: Retrieve company fundamentals
- **NewsTool**: Fetch news and sentiment
- **ChartDataTool**: Prepare data for visualization
- **EarningsReportTool**: Fetch SEC filings (10-K, 10-Q) and financial data
- **MacroEconomicTool**: Fetch FRED data (rates, inflation, GDP, employment)
- **SectorAnalysisTool**: Analyze sector performance and rotation
- **GeopoliticalTool**: Analyze geopolitical risks and market impact

## Usage Examples

### Get Current Price

```rust
let result = agent.process(
    "What is the current price of AAPL?".to_string(),
    &mut Context::new()
).await?;
```

### Technical Analysis

```rust
let analysis = agent.analyze_technical("AAPL").await?;
```

### Fundamental Analysis

```rust
let analysis = agent.analyze_fundamental("AAPL").await?;
```

### News & Sentiment

```rust
let news = agent.analyze_news("AAPL").await?;
```

### Comprehensive Analysis

```rust
let full_analysis = agent.analyze("AAPL").await?;
```

### Earnings Analysis

```rust
// Analyze SEC filings and financial reports
let earnings = agent.analyze_earnings("AAPL").await?;
```

### Macroeconomic Analysis

```rust
// Analyze Fed policy, inflation, GDP, and economic conditions
let macro_view = agent.analyze_macro().await?;
```

### Geopolitical Analysis

```rust
// Assess geopolitical risks and market impact
let geo_analysis = agent.analyze_geopolitical().await?;
```

### Comprehensive Analysis with Macro Factors

```rust
// Full investment analysis including macro environment
let comprehensive = agent.analyze_comprehensive("AAPL").await?;
```

## Supported Indicators

### Trend Indicators
- SMA (Simple Moving Average)
- EMA (Exponential Moving Average)
- MACD (Moving Average Convergence Divergence)

### Momentum Indicators
- RSI (Relative Strength Index)
- Stochastic Oscillator

### Volatility Indicators
- Bollinger Bands
- ATR (Average True Range)

### Volume Indicators
- Volume analysis

## Data Sources

### Yahoo Finance (Primary)
- **Advantages**: No API key required, good free tier, comprehensive historical data
- **Use cases**: Price data, historical quotes, basic company info
- **Rate limits**: Generous for individual use

### Alpha Vantage
- **Advantages**: Comprehensive fundamental data, news sentiment analysis
- **Use cases**: Company overview, financial metrics (P/E, EPS, market cap), news
- **Requirements**: API key (free tier available)
- **Rate limits**: 5 requests/minute (free tier)

### SEC EDGAR
- **Advantages**: Official SEC filings, free, no API key required
- **Use cases**: 10-K annual reports, 10-Q quarterly reports, financial statements
- **Rate limits**: 10 requests/second

### FRED (Federal Reserve Economic Data)
- **Advantages**: Official economic data, comprehensive time series
- **Use cases**: Interest rates, inflation (CPI, PCE), GDP, unemployment
- **Requirements**: API key (free)
- **Rate limits**: 120 requests/minute

### Finnhub
- **Advantages**: Real-time market news, company news
- **Use cases**: News aggregation, market sentiment
- **Requirements**: API key (free tier: 60 requests/minute)

## Caching Strategy

The system uses a multi-tiered caching approach:

```rust
CacheManager {
    realtime: StockCache,      // 60s TTL for quotes/prices
    fundamental: StockCache,    // 1h TTL for company data
    news: StockCache,          // 5m TTL for news articles
    earnings: StockCache,      // 24h TTL for SEC filings
    macro: StockCache,         // 1h TTL for economic data
    sector: StockCache,        // 30m TTL for sector data
}
```

Benefits:
- Reduces API calls during multi-turn conversations
- Faster response times
- Stays within rate limits
- Configurable TTL per data type

## Error Handling

The system handles errors gracefully with:

- **Automatic Retries**: Exponential backoff for transient errors
- **Multiple Providers**: Fallback to secondary data sources
- **Partial Results**: Returns what's available rather than failing completely
- **Clear Error Messages**: Descriptive errors for debugging

## Examples

See [`examples/basic_analysis.rs`](examples/basic_analysis.rs) for a complete working example.

Run with:
```bash
export ANTHROPIC_API_KEY=your_key
export ALPHA_VANTAGE_API_KEY=your_key  # Optional
cargo run --example basic_analysis AAPL
```

## Testing

Run tests:
```bash
cargo test --package agent-stock

# Include network tests (requires API keys)
cargo test --package agent-stock -- --ignored
```

## Limitations & Future Work

### Current Limitations
1. Some news data may use mock responses if API keys not configured
2. Company info from Yahoo Finance API is limited
3. No options chain data yet
4. No portfolio analysis

### Planned Enhancements
- [x] Real news API integration (Finnhub, Alpha Vantage)
- [x] SEC EDGAR integration for earnings reports
- [x] FRED integration for macroeconomic data
- [x] Sector rotation analysis
- [x] Geopolitical risk assessment
- [ ] MCP server integration
- [ ] Portfolio analysis and tracking
- [ ] Backtesting capabilities
- [ ] Options chain analysis
- [ ] Real-time streaming data (WebSockets)
- [ ] Custom indicator support
- [ ] Stock screening
- [ ] Alerts and notifications

## Contributing

Contributions are welcome! Please ensure:
- All tests pass (`cargo test`)
- Code is formatted (`cargo fmt`)
- Lints pass (`cargo clippy`)
- Documentation is updated

## License

This crate is part of the agent-rs workspace.

## Dependencies

- `rust_ti` v2.2.0 - Technical indicators
- `yahoo_finance_api` v4.1.0 - Yahoo Finance data
- `cached` v0.56.0 - Caching layer
- `chrono` v0.4.42 - Date/time handling
- `tokio` v1.48 - Async runtime
- `serde` v1.0.228 - Serialization
- `reqwest` v0.12.24 - HTTP client

## References

- [Yahoo Finance API](https://finance.yahoo.com/)
- [Alpha Vantage API](https://www.alphavantage.co/)
- [SEC EDGAR API](https://www.sec.gov/edgar/sec-api-documentation)
- [FRED API](https://fred.stlouisfed.org/docs/api/fred/)
- [Finnhub API](https://finnhub.io/docs/api)
- [rust_ti Documentation](https://crates.io/crates/rust_ti)
- [Technical Analysis Primer](https://www.investopedia.com/technical-analysis-4689657)
