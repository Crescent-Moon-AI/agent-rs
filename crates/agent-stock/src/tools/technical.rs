//! Tool for calculating technical indicators

use agent_core::Result as AgentResult;
use agent_tools::Tool;
use async_trait::async_trait;
use serde::Deserialize;
use serde_json::{Value, json};
use std::sync::Arc;
use ta::{
    Next,
    indicators::{
        AverageTrueRange, BollingerBands, ExponentialMovingAverage, RelativeStrengthIndex,
        SimpleMovingAverage,
    },
};

use crate::api::YahooFinanceClient;
use crate::cache::StockCache;
use crate::config::StockConfig;
use crate::error::{Result, StockError};

/// Tool for calculating technical indicators
pub struct TechnicalIndicatorTool {
    yahoo_client: YahooFinanceClient,
    _cache: StockCache,
    _config: Arc<StockConfig>,
}

#[derive(Debug, Deserialize)]
struct TechnicalParams {
    symbol: String,
    indicator: String,
    #[serde(default = "default_period")]
    period: usize,
    #[serde(default)]
    range: Option<String>,
}

fn default_period() -> usize {
    14
}

impl TechnicalIndicatorTool {
    /// Create a new technical indicator tool
    pub fn new(config: Arc<StockConfig>, cache: StockCache) -> Self {
        Self {
            yahoo_client: YahooFinanceClient::new(),
            _cache: cache,
            _config: config,
        }
    }

    /// Calculate technical indicator
    async fn calculate_indicator(&self, params: TechnicalParams) -> Result<Value> {
        let symbol = params.symbol.to_uppercase();
        let range = params.range.unwrap_or_else(|| "3mo".to_string());

        // Fetch historical data
        let quotes = self
            .yahoo_client
            .get_historical_range(&symbol, &range)
            .await?;

        if quotes.is_empty() {
            return Err(StockError::DataUnavailable {
                symbol: symbol.clone(),
                reason: "No historical data available".to_string(),
            });
        }

        // Extract closing prices
        let closes: Vec<f64> = quotes.iter().map(|q| q.close).collect();
        let highs: Vec<f64> = quotes.iter().map(|q| q.high).collect();
        let lows: Vec<f64> = quotes.iter().map(|q| q.low).collect();
        let volumes: Vec<f64> = quotes.iter().map(|q| q.volume as f64).collect();

        // Calculate indicator based on type
        let result = match params.indicator.to_uppercase().as_str() {
            "RSI" => {
                let mut rsi = RelativeStrengthIndex::new(params.period)
                    .map_err(|e| StockError::IndicatorError(e.to_string()))?;
                let mut rsi_values = Vec::new();
                for &close in &closes {
                    rsi_values.push(rsi.next(close));
                }
                let current_rsi = rsi_values.last().copied().unwrap_or(0.0);

                json!({
                    "indicator": "RSI",
                    "period": params.period,
                    "current_value": current_rsi,
                    "interpretation": interpret_rsi(current_rsi),
                    "recent_values": &rsi_values[rsi_values.len().saturating_sub(10)..],
                })
            }
            "SMA" => {
                let mut sma = SimpleMovingAverage::new(params.period)
                    .map_err(|e| StockError::IndicatorError(e.to_string()))?;
                let mut sma_values = Vec::new();
                for &close in &closes {
                    sma_values.push(sma.next(close));
                }
                let current_sma = sma_values.last().copied().unwrap_or(0.0);
                let current_price = closes.last().copied().unwrap_or(0.0);

                json!({
                    "indicator": "SMA",
                    "period": params.period,
                    "current_value": current_sma,
                    "current_price": current_price,
                    "price_vs_sma": if current_price > current_sma { "above" } else { "below" },
                    "recent_values": &sma_values[sma_values.len().saturating_sub(10)..],
                })
            }
            "EMA" => {
                let mut ema = ExponentialMovingAverage::new(params.period)
                    .map_err(|e| StockError::IndicatorError(e.to_string()))?;
                let mut ema_values = Vec::new();
                for &close in &closes {
                    ema_values.push(ema.next(close));
                }
                let current_ema = ema_values.last().copied().unwrap_or(0.0);
                let current_price = closes.last().copied().unwrap_or(0.0);

                json!({
                    "indicator": "EMA",
                    "period": params.period,
                    "current_value": current_ema,
                    "current_price": current_price,
                    "price_vs_ema": if current_price > current_ema { "above" } else { "below" },
                    "recent_values": &ema_values[ema_values.len().saturating_sub(10)..],
                })
            }
            "MACD" => {
                // Calculate simple MACD using EMA difference
                let mut ema12 = ExponentialMovingAverage::new(12)
                    .map_err(|e| StockError::IndicatorError(e.to_string()))?;
                let mut ema26 = ExponentialMovingAverage::new(26)
                    .map_err(|e| StockError::IndicatorError(e.to_string()))?;

                let mut macd_line = Vec::new();
                for &close in &closes {
                    let e12 = ema12.next(close);
                    let e26 = ema26.next(close);
                    macd_line.push(e12 - e26);
                }

                let current_macd = macd_line.last().copied().unwrap_or(0.0);

                json!({
                    "indicator": "MACD",
                    "current_value": current_macd,
                    "interpretation": if current_macd > 0.0 { "Bullish" } else { "Bearish" },
                    "recent_values": &macd_line[macd_line.len().saturating_sub(10)..],
                })
            }
            "BBANDS" | "BB" => {
                let mut bb = BollingerBands::new(params.period, 2.0)
                    .map_err(|e| StockError::IndicatorError(e.to_string()))?;
                let mut bb_values: Vec<f64> = Vec::new();
                for &close in &closes {
                    let bb_result = bb.next(close);
                    // BollingerBands returns a struct with average, upper, lower
                    bb_values.push(bb_result.average);
                }
                let current_bb_avg = bb_values.last().copied().unwrap_or(0.0);
                let current_price = closes.last().copied().unwrap_or(0.0);

                json!({
                    "indicator": "Bollinger Bands",
                    "period": params.period,
                    "current_average": current_bb_avg,
                    "current_price": current_price,
                    "interpretation": "Volatility bands around price",
                })
            }
            "ATR" => {
                let mut atr = AverageTrueRange::new(params.period)
                    .map_err(|e| StockError::IndicatorError(e.to_string()))?;
                let mut atr_values = Vec::new();
                for i in 0..highs.len() {
                    let bar = ta::DataItem::builder()
                        .high(highs[i])
                        .low(lows[i])
                        .close(closes[i])
                        .volume(volumes[i] as f64)
                        .build()
                        .unwrap();
                    atr_values.push(atr.next(&bar));
                }
                let current_atr = atr_values.last().copied().unwrap_or(0.0);

                json!({
                    "indicator": "ATR",
                    "period": params.period,
                    "current_value": current_atr,
                    "interpretation": "Measures market volatility",
                })
            }
            _ => {
                return Err(StockError::IndicatorError(format!(
                    "Unsupported indicator: {}. Supported: RSI, SMA, EMA, MACD, BBANDS, ATR",
                    params.indicator
                )));
            }
        };

        Ok(json!({
            "symbol": symbol,
            "indicator_data": result,
            "data_points": closes.len(),
            "time_range": range,
        }))
    }
}

/// Interpret RSI value
fn interpret_rsi(rsi: f64) -> &'static str {
    if rsi > 70.0 {
        "Overbought - potential sell signal"
    } else if rsi < 30.0 {
        "Oversold - potential buy signal"
    } else {
        "Neutral"
    }
}

#[async_trait]
impl Tool for TechnicalIndicatorTool {
    async fn execute(&self, params: Value) -> AgentResult<Value> {
        let params: TechnicalParams = serde_json::from_value(params).map_err(|e| {
            agent_core::Error::ProcessingFailed(format!("Invalid parameters: {}", e))
        })?;

        self.calculate_indicator(params)
            .await
            .map_err(|e| agent_core::Error::ProcessingFailed(e.to_string()))
    }

    fn name(&self) -> &str {
        "technical_indicator"
    }

    fn description(&self) -> &str {
        "Calculate technical indicators for stock analysis. \
         Supports RSI, SMA, EMA, MACD, Bollinger Bands, ATR, and Stochastic oscillator."
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "symbol": {
                    "type": "string",
                    "description": "Stock ticker symbol"
                },
                "indicator": {
                    "type": "string",
                    "description": "Technical indicator to calculate",
                    "enum": ["RSI", "SMA", "EMA", "MACD", "BBANDS", "BB", "ATR", "STOCH"]
                },
                "period": {
                    "type": "integer",
                    "description": "Period for the indicator calculation",
                    "default": 14
                },
                "range": {
                    "type": "string",
                    "description": "Time range for historical data",
                    "enum": ["1mo", "3mo", "6mo", "1y"],
                    "default": "3mo"
                }
            },
            "required": ["symbol", "indicator"]
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_interpret_rsi() {
        assert_eq!(interpret_rsi(75.0), "Overbought - potential sell signal");
        assert_eq!(interpret_rsi(25.0), "Oversold - potential buy signal");
        assert_eq!(interpret_rsi(50.0), "Neutral");
    }

    #[test]
    fn test_tool_metadata() {
        let config = Arc::new(StockConfig::default());
        let cache = StockCache::new(Duration::from_secs(60));
        let tool = TechnicalIndicatorTool::new(config, cache);

        assert_eq!(tool.name(), "technical_indicator");
        let schema = tool.input_schema();
        assert_eq!(schema["type"], "object");
    }
}
