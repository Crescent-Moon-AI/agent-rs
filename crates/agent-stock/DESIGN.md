# Agent Stock - 增强设计文档

## 概述

本文档描述了 agent-stock 模块的增强设计，旨在提供更全面的股票分析能力，包括：

1. **财报分析** - 收集和分析公司季度/年度财报
2. **新闻分析** - 增强的新闻聚合和情感分析
3. **宏观经济分析** - 美联储政策、经济指标
4. **国际形势分析** - 地缘政治、贸易政策
5. **行业分析** - 板块趋势、竞争对手分析

## 架构设计

### 增强后的多代理架构

```
StockAnalysisAgent (Coordinator)
│
├── DataFetcherAgent (现有)
│   ├── StockDataTool
│   └── FundamentalDataTool
│
├── TechnicalAnalyzerAgent (现有)
│   ├── StockDataTool
│   ├── TechnicalIndicatorTool
│   └── ChartDataTool
│
├── FundamentalAnalyzerAgent (现有 - 增强)
│   ├── FundamentalDataTool
│   └── EarningsReportTool (新增)
│
├── NewsAnalyzerAgent (现有 - 增强)
│   ├── NewsTool (增强)
│   └── GeopoliticalTool (新增)
│
├── EarningsAnalyzerAgent (新增)
│   ├── EarningsReportTool
│   ├── SECFilingTool
│   └── AnalystRatingTool
│
├── MacroAnalyzerAgent (新增)
│   ├── MacroEconomicTool
│   ├── FedPolicyTool
│   └── GeopoliticalTool
│
└── SectorAnalyzerAgent (新增)
    ├── SectorAnalysisTool
    └── CompetitorTool
```

## 新增数据源

### 1. SEC EDGAR API (免费)
- **用途**: 获取上市公司财务报告 (10-K, 10-Q, 8-K)
- **优势**: 官方数据源，免费无限制
- **数据**: 财务报表、管理层讨论、风险因素

### 2. FRED API (Federal Reserve Economic Data) (免费)
- **用途**: 获取美国经济指标数据
- **优势**: 官方数据源，免费，数据丰富
- **数据**: 
  - 利率 (联邦基金利率、国债收益率)
  - 通胀 (CPI, PCE)
  - 就业 (失业率、非农就业)
  - GDP增长率
  - 货币供应量 (M2)

### 3. 增强的新闻数据源
- **Finnhub** (现有): 公司新闻
- **Alpha Vantage** (现有): 新闻情感分析
- **新增考虑**:
  - GNews API: 全球新闻聚合
  - Mediastack: 多语言新闻
  - RSS feeds: 财经媒体 (Bloomberg, Reuters, WSJ)

## 新增工具详细设计

### 1. EarningsReportTool

```rust
/// 获取和分析公司财务报告
pub struct EarningsReportTool {
    sec_client: SecEdgarClient,
    cache: StockCache,
    config: Arc<StockConfig>,
}

// 输入参数
struct EarningsParams {
    symbol: String,           // 股票代码
    report_type: ReportType,  // Annual, Quarterly, Current
    years: Option<u32>,       // 获取年数
}

// 输出数据
struct EarningsReport {
    symbol: String,
    fiscal_year: String,
    fiscal_quarter: Option<String>,
    revenue: f64,
    net_income: f64,
    eps: f64,
    eps_diluted: f64,
    revenue_growth_yoy: f64,
    net_income_growth_yoy: f64,
    gross_margin: f64,
    operating_margin: f64,
    net_margin: f64,
    filing_date: String,
    report_url: String,
}
```

### 2. MacroEconomicTool

```rust
/// 获取宏观经济指标
pub struct MacroEconomicTool {
    fred_client: FredClient,
    cache: StockCache,
    config: Arc<StockConfig>,
}

// 支持的经济指标
enum EconomicIndicator {
    FedFundsRate,      // 联邦基金利率
    Treasury10Y,       // 10年期国债收益率
    Treasury2Y,        // 2年期国债收益率
    YieldCurve,        // 收益率曲线 (2Y-10Y利差)
    CPI,               // 消费者价格指数
    CorePCE,           // 核心PCE (美联储关注的通胀指标)
    UnemploymentRate,  // 失业率
    NonFarmPayrolls,   // 非农就业
    GDP,               // GDP增长率
    RetailSales,       // 零售销售
    ConsumerConfidence,// 消费者信心指数
    PMI,               // 采购经理人指数
    M2,                // M2货币供应
    HousingStarts,     // 新屋开工
}

// 输出数据
struct MacroData {
    indicator: String,
    value: f64,
    previous_value: f64,
    change: f64,
    change_percent: f64,
    date: String,
    trend: String,        // "rising", "falling", "stable"
    interpretation: String,
}
```

### 3. FedPolicyTool

```rust
/// 分析美联储政策和决议
pub struct FedPolicyTool {
    fred_client: FredClient,
    news_client: NewsClient,
    cache: StockCache,
}

// 输出数据
struct FedPolicyAnalysis {
    current_rate: f64,
    rate_decision: String,       // "hike", "cut", "hold"
    last_meeting_date: String,
    next_meeting_date: String,
    dot_plot_median: Option<f64>,
    market_expectation: String,
    fomc_statement_summary: String,
    economic_projections: EconomicProjections,
}
```

### 4. SectorAnalysisTool

```rust
/// 行业板块分析
pub struct SectorAnalysisTool {
    yahoo_client: YahooFinanceClient,
    cache: StockCache,
}

// 支持的板块
enum Sector {
    Technology,
    Healthcare,
    Financials,
    ConsumerDiscretionary,
    ConsumerStaples,
    Energy,
    Materials,
    Industrials,
    Utilities,
    RealEstate,
    CommunicationServices,
}

// 输出数据
struct SectorAnalysis {
    sector: String,
    performance_1d: f64,
    performance_1w: f64,
    performance_1m: f64,
    performance_ytd: f64,
    top_gainers: Vec<StockPerformance>,
    top_losers: Vec<StockPerformance>,
    sector_pe: f64,
    sector_market_cap: f64,
    sector_outlook: String,
}
```

### 5. GeopoliticalTool

```rust
/// 地缘政治和国际形势分析
pub struct GeopoliticalTool {
    news_clients: Vec<Box<dyn NewsProvider>>,
    cache: StockCache,
}

// 关注的主题
enum GeopoliticalTopic {
    UsChinaRelations,      // 中美关系
    TradePolicies,         // 贸易政策
    Sanctions,             // 制裁
    MiddleEast,            // 中东局势
    EuropeanUnion,         // 欧盟动态
    EmergingMarkets,       // 新兴市场
    CurrencyPolicies,      // 货币政策
    SupplyChain,           // 供应链
}

// 输出数据
struct GeopoliticalAnalysis {
    topic: String,
    summary: String,
    impact_assessment: String,  // "positive", "negative", "neutral"
    affected_sectors: Vec<String>,
    affected_stocks: Vec<String>,
    risk_level: String,         // "low", "medium", "high"
    recent_events: Vec<NewsEvent>,
}
```

## 新增代理详细设计

### 1. EarningsAnalyzerAgent

```rust
/// 财报分析专家代理
pub struct EarningsAnalyzerAgent {
    runtime: Arc<AgentRuntime>,
    earnings_tool: Arc<EarningsReportTool>,
    sec_filing_tool: Arc<SecFilingTool>,
    config: Arc<StockConfig>,
}

// System Prompt
const EARNINGS_AGENT_PROMPT: &str = r#"
你是一位专业的财务分析师，专注于分析公司财务报告。

你的职责：
1. 分析季度/年度财报，提取关键财务指标
2. 比较同比和环比变化
3. 评估盈利质量和可持续性
4. 识别财务风险和机会
5. 解读管理层讨论和分析 (MD&A)

分析要点：
- 收入增长趋势
- 利润率变化
- 现金流状况
- 资产负债表健康度
- 关键业务指标 (KPIs)
- 管理层指引

输出格式：
1. 财务摘要
2. 关键指标对比
3. 亮点与风险
4. 投资建议
"#;
```

### 2. MacroAnalyzerAgent

```rust
/// 宏观经济分析专家代理
pub struct MacroAnalyzerAgent {
    runtime: Arc<AgentRuntime>,
    macro_tool: Arc<MacroEconomicTool>,
    fed_policy_tool: Arc<FedPolicyTool>,
    geopolitical_tool: Arc<GeopoliticalTool>,
    config: Arc<StockConfig>,
}

// System Prompt
const MACRO_AGENT_PROMPT: &str = r#"
你是一位宏观经济分析师，专注于分析经济形势对股市的影响。

你的职责：
1. 跟踪关键经济指标
2. 分析美联储政策取向
3. 评估通胀和就业形势
4. 分析国际形势影响
5. 预测市场走向

重点关注：
- 美联储利率决策
- 通胀数据 (CPI, PCE)
- 就业数据 (非农、失业率)
- GDP增长
- 地缘政治风险

分析框架：
1. 当前经济状态
2. 政策环境
3. 风险因素
4. 市场影响
5. 投资策略建议
"#;
```

### 3. SectorAnalyzerAgent

```rust
/// 行业分析专家代理
pub struct SectorAnalyzerAgent {
    runtime: Arc<AgentRuntime>,
    sector_tool: Arc<SectorAnalysisTool>,
    competitor_tool: Arc<CompetitorTool>,
    config: Arc<StockConfig>,
}

// System Prompt
const SECTOR_AGENT_PROMPT: &str = r#"
你是一位行业研究分析师，专注于板块和竞争分析。

你的职责：
1. 分析行业趋势和周期
2. 评估板块估值水平
3. 比较同行业公司
4. 识别行业龙头和新秀
5. 分析竞争格局

分析要点：
- 板块轮动趋势
- 行业生命周期
- 竞争优势分析
- 市场份额变化
- 技术创新影响

输出格式：
1. 行业概况
2. 关键参与者
3. 竞争分析
4. 趋势预测
5. 投资机会
"#;
```

## 增强的路由逻辑

```rust
/// 智能路由到合适的代理
fn route_to_agent(input: &str, context: &Context) -> String {
    let input_lower = input.to_lowercase();
    
    // 财报/盈利相关
    if input_lower.contains("earnings") || 
       input_lower.contains("财报") ||
       input_lower.contains("10-k") || 
       input_lower.contains("10-q") ||
       input_lower.contains("季报") ||
       input_lower.contains("年报") {
        return "earnings-analyzer".to_string();
    }
    
    // 宏观经济相关
    if input_lower.contains("fed") || 
       input_lower.contains("美联储") ||
       input_lower.contains("interest rate") ||
       input_lower.contains("利率") ||
       input_lower.contains("inflation") ||
       input_lower.contains("通胀") ||
       input_lower.contains("gdp") ||
       input_lower.contains("unemployment") ||
       input_lower.contains("macro") ||
       input_lower.contains("宏观") {
        return "macro-analyzer".to_string();
    }
    
    // 地缘政治相关
    if input_lower.contains("geopolitical") ||
       input_lower.contains("trade war") ||
       input_lower.contains("贸易战") ||
       input_lower.contains("sanctions") ||
       input_lower.contains("制裁") ||
       input_lower.contains("国际形势") {
        return "macro-analyzer".to_string();
    }
    
    // 行业分析相关
    if input_lower.contains("sector") ||
       input_lower.contains("板块") ||
       input_lower.contains("industry") ||
       input_lower.contains("行业") ||
       input_lower.contains("competitor") ||
       input_lower.contains("竞争") {
        return "sector-analyzer".to_string();
    }
    
    // ... 现有路由逻辑
}
```

## 配置增强

```rust
pub struct StockConfig {
    // ... 现有配置
    
    // SEC EDGAR 配置
    pub sec_user_agent: String,        // SEC要求的User-Agent
    
    // FRED API 配置
    pub fred_api_key: Option<String>,  // FRED API密钥
    pub fred_rate_limit: u32,          // 请求限制
    
    // 新闻API配置
    pub gnews_api_key: Option<String>,
    
    // 分析偏好
    pub analysis_depth: AnalysisDepth, // Quick, Standard, Deep
    pub focus_regions: Vec<Region>,    // 关注的地区
    pub focus_sectors: Vec<Sector>,    // 关注的行业
}

pub enum AnalysisDepth {
    Quick,     // 快速分析，主要指标
    Standard,  // 标准分析，全面覆盖
    Deep,      // 深度分析，详细研究
}
```

## 缓存策略

```rust
pub struct CacheManager {
    // 现有缓存
    realtime: StockCache,      // 60s TTL - 价格数据
    fundamental: StockCache,   // 1h TTL - 基本面数据
    news: StockCache,          // 5m TTL - 新闻数据
    
    // 新增缓存
    earnings: StockCache,      // 24h TTL - 财报数据 (更新不频繁)
    macro: StockCache,         // 1h TTL - 宏观数据
    sector: StockCache,        // 30m TTL - 板块数据
    geopolitical: StockCache,  // 15m TTL - 地缘政治新闻
}
```

## API 接口设计

### SEC EDGAR Client

```rust
pub struct SecEdgarClient {
    client: Client,
    user_agent: String,
    rate_limiter: RateLimiter,
}

impl SecEdgarClient {
    /// 获取公司SEC文件列表
    pub async fn get_filings(
        &self, 
        cik: &str, 
        form_type: &str
    ) -> Result<Vec<SecFiling>>;
    
    /// 获取10-K年报
    pub async fn get_10k(&self, cik: &str) -> Result<AnnualReport>;
    
    /// 获取10-Q季报
    pub async fn get_10q(&self, cik: &str) -> Result<QuarterlyReport>;
    
    /// 获取8-K重大事项
    pub async fn get_8k(&self, cik: &str) -> Result<Vec<CurrentReport>>;
    
    /// 搜索公司CIK
    pub async fn search_company(&self, query: &str) -> Result<Vec<CompanyInfo>>;
}
```

### FRED API Client

```rust
pub struct FredClient {
    client: Client,
    api_key: String,
    rate_limiter: RateLimiter,
}

impl FredClient {
    /// 获取经济指标数据
    pub async fn get_series(
        &self, 
        series_id: &str,
        start_date: Option<&str>,
        end_date: Option<&str>
    ) -> Result<EconomicSeries>;
    
    /// 获取最新观测值
    pub async fn get_latest(&self, series_id: &str) -> Result<f64>;
    
    /// 搜索系列
    pub async fn search(&self, query: &str) -> Result<Vec<SeriesInfo>>;
    
    /// 常用系列ID
    pub const FED_FUNDS_RATE: &'static str = "FEDFUNDS";
    pub const TREASURY_10Y: &'static str = "DGS10";
    pub const TREASURY_2Y: &'static str = "DGS2";
    pub const UNEMPLOYMENT: &'static str = "UNRATE";
    pub const CPI: &'static str = "CPIAUCSL";
    pub const GDP: &'static str = "GDP";
}
```

## 实现优先级

### Phase 1: 核心增强 (高优先级)
1. ✅ 设计文档完成
2. SEC EDGAR API 客户端
3. FRED API 客户端
4. EarningsReportTool
5. MacroEconomicTool

### Phase 2: 分析能力 (中优先级)
6. EarningsAnalyzerAgent
7. MacroAnalyzerAgent
8. 增强 StockAnalysisAgent 路由

### Phase 3: 扩展功能 (低优先级)
9. SectorAnalysisTool
10. GeopoliticalTool
11. SectorAnalyzerAgent
12. 更多新闻源集成

## 使用示例

```rust
// 完整分析
let agent = StockAnalysisAgent::new(runtime, config).await?;

// 财报分析
let earnings = agent.analyze_earnings("AAPL").await?;

// 宏观分析
let macro_view = agent.analyze_macro_environment().await?;

// 综合分析 (包含所有维度)
let full_analysis = agent.analyze_comprehensive("AAPL").await?;

// 自定义分析
let custom = agent.process(
    "分析苹果公司最近的财报，结合当前美联储加息环境，
     给出投资建议".to_string(),
    &mut context
).await?;
```

## 风险与注意事项

1. **API 限制**: 
   - SEC EDGAR: 10 requests/second
   - FRED: 120 requests/minute
   - 需要合理的速率限制

2. **数据延迟**:
   - SEC 文件处理需要时间
   - 宏观数据有发布滞后

3. **分析局限**:
   - LLM 分析仅供参考
   - 不构成投资建议
   - 需要人工验证

4. **合规要求**:
   - SEC 要求提供 User-Agent
   - 遵守各 API 服务条款

## 测试策略

1. **单元测试**: 每个工具的参数解析和输出格式
2. **集成测试**: API 客户端 (需要网络)
3. **Mock 测试**: 模拟 API 响应
4. **端到端测试**: 完整分析流程

---

**设计日期**: 2024-12-18
**版本**: 2.0
**状态**: 设计中
