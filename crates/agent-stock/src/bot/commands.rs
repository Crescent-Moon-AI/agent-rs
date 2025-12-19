//! Command parsing and handling for the stock analysis bot
//!
//! This module provides command-line interface commands for the bot.

use crate::error::{Result, StockError};

/// Parsed command from user input
#[derive(Debug, Clone, PartialEq)]
pub enum Command {
    /// Comprehensive analysis of a stock
    Analyze { symbol: String },
    /// Technical analysis only
    Technical { symbol: String },
    /// Fundamental analysis only
    Fundamental { symbol: String },
    /// News and sentiment analysis
    News { symbol: String },
    /// Earnings analysis
    Earnings { symbol: String },
    /// Macro economic analysis
    Macro,
    /// Geopolitical analysis
    Geopolitical,
    /// Compare multiple stocks
    Compare { symbols: Vec<String> },
    /// Add stock to watchlist
    Watch { symbol: String },
    /// Remove stock from watchlist
    Unwatch { symbol: String },
    /// Show watchlist
    Watchlist,
    /// Clear conversation history
    Clear,
    /// Show help
    Help,
    /// Exit the bot
    Exit,
    /// Natural language query (not a command)
    Query { text: String },
}

impl Command {
    /// Parse a command from user input
    pub fn parse(input: &str) -> Result<Self> {
        let input = input.trim();

        if input.is_empty() {
            return Err(StockError::CommandError("Empty input".to_string()));
        }

        // Check if it's a command (starts with /)
        if !input.starts_with('/') {
            return Ok(Command::Query {
                text: input.to_string(),
            });
        }

        let parts: Vec<&str> = input[1..].split_whitespace().collect();
        if parts.is_empty() {
            return Err(StockError::CommandError("Empty command".to_string()));
        }

        let cmd = parts[0].to_lowercase();
        let args = &parts[1..];

        match cmd.as_str() {
            "analyze" | "a" | "分析" => {
                let symbol = args.first().ok_or_else(|| {
                    StockError::CommandError("Missing symbol for analyze command".to_string())
                })?;
                Ok(Command::Analyze {
                    symbol: symbol.to_uppercase(),
                })
            }
            "technical" | "tech" | "t" | "技术" => {
                let symbol = args.first().ok_or_else(|| {
                    StockError::CommandError("Missing symbol for technical command".to_string())
                })?;
                Ok(Command::Technical {
                    symbol: symbol.to_uppercase(),
                })
            }
            "fundamental" | "fund" | "f" | "基本面" => {
                let symbol = args.first().ok_or_else(|| {
                    StockError::CommandError("Missing symbol for fundamental command".to_string())
                })?;
                Ok(Command::Fundamental {
                    symbol: symbol.to_uppercase(),
                })
            }
            "news" | "n" | "新闻" => {
                let symbol = args.first().ok_or_else(|| {
                    StockError::CommandError("Missing symbol for news command".to_string())
                })?;
                Ok(Command::News {
                    symbol: symbol.to_uppercase(),
                })
            }
            "earnings" | "e" | "财报" => {
                let symbol = args.first().ok_or_else(|| {
                    StockError::CommandError("Missing symbol for earnings command".to_string())
                })?;
                Ok(Command::Earnings {
                    symbol: symbol.to_uppercase(),
                })
            }
            "macro" | "m" | "宏观" => Ok(Command::Macro),
            "geopolitical" | "geo" | "地缘" => Ok(Command::Geopolitical),
            "compare" | "cmp" | "比较" => {
                if args.len() < 2 {
                    return Err(StockError::CommandError(
                        "Compare requires at least 2 symbols".to_string(),
                    ));
                }
                let symbols: Vec<String> = args.iter().map(|s| s.to_uppercase()).collect();
                Ok(Command::Compare { symbols })
            }
            "watch" | "w" | "关注" => {
                let symbol = args.first().ok_or_else(|| {
                    StockError::CommandError("Missing symbol for watch command".to_string())
                })?;
                Ok(Command::Watch {
                    symbol: symbol.to_uppercase(),
                })
            }
            "unwatch" | "取消关注" => {
                let symbol = args.first().ok_or_else(|| {
                    StockError::CommandError("Missing symbol for unwatch command".to_string())
                })?;
                Ok(Command::Unwatch {
                    symbol: symbol.to_uppercase(),
                })
            }
            "watchlist" | "list" | "关注列表" => Ok(Command::Watchlist),
            "clear" | "cls" | "清空" => Ok(Command::Clear),
            "help" | "h" | "?" | "帮助" => Ok(Command::Help),
            "exit" | "quit" | "q" | "退出" => Ok(Command::Exit),
            _ => Err(StockError::CommandError(format!("Unknown command: {}", cmd))),
        }
    }

    /// Get help text for all commands
    pub fn help_text() -> &'static str {
        r#"
Stock Analysis Bot Commands
============================

Analysis Commands:
  /analyze <symbol>      综合分析股票 (Comprehensive analysis)
  /technical <symbol>    技术分析 (Technical analysis)
  /fundamental <symbol>  基本面分析 (Fundamental analysis)
  /news <symbol>         新闻情绪分析 (News & sentiment)
  /earnings <symbol>     财报分析 (Earnings analysis)
  /macro                 宏观经济分析 (Macro economic analysis)
  /geopolitical          地缘政治分析 (Geopolitical analysis)
  /compare <s1> <s2> ... 比较多只股票 (Compare stocks)

Watchlist Commands:
  /watch <symbol>        添加到关注列表 (Add to watchlist)
  /unwatch <symbol>      从关注列表移除 (Remove from watchlist)
  /watchlist             显示关注列表 (Show watchlist)

Other Commands:
  /clear                 清空对话历史 (Clear conversation history)
  /help                  显示帮助 (Show help)
  /exit                  退出 (Exit)

Command Aliases:
  /a = /analyze         /t = /technical      /f = /fundamental
  /n = /news           /e = /earnings       /m = /macro
  /w = /watch          /cmp = /compare      /q = /exit

Natural Language:
  You can also ask questions in natural language:
  - "苹果股票最近表现怎么样?"
  - "Analyze Tesla's technical indicators"
  - "比较微软和谷歌"
"#
    }

    /// Get a short description of the command
    pub fn description(&self) -> &'static str {
        match self {
            Command::Analyze { .. } => "Comprehensive stock analysis",
            Command::Technical { .. } => "Technical analysis",
            Command::Fundamental { .. } => "Fundamental analysis",
            Command::News { .. } => "News and sentiment analysis",
            Command::Earnings { .. } => "Earnings analysis",
            Command::Macro => "Macro economic analysis",
            Command::Geopolitical => "Geopolitical risk analysis",
            Command::Compare { .. } => "Stock comparison",
            Command::Watch { .. } => "Add to watchlist",
            Command::Unwatch { .. } => "Remove from watchlist",
            Command::Watchlist => "Show watchlist",
            Command::Clear => "Clear conversation history",
            Command::Help => "Show help",
            Command::Exit => "Exit the bot",
            Command::Query { .. } => "Natural language query",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_analyze() {
        let cmd = Command::parse("/analyze AAPL").unwrap();
        assert_eq!(
            cmd,
            Command::Analyze {
                symbol: "AAPL".to_string()
            }
        );

        let cmd = Command::parse("/a aapl").unwrap();
        assert_eq!(
            cmd,
            Command::Analyze {
                symbol: "AAPL".to_string()
            }
        );
    }

    #[test]
    fn test_parse_compare() {
        let cmd = Command::parse("/compare AAPL GOOGL MSFT").unwrap();
        assert_eq!(
            cmd,
            Command::Compare {
                symbols: vec!["AAPL".to_string(), "GOOGL".to_string(), "MSFT".to_string()]
            }
        );
    }

    #[test]
    fn test_parse_natural_language() {
        let cmd = Command::parse("What is the price of AAPL?").unwrap();
        assert_eq!(
            cmd,
            Command::Query {
                text: "What is the price of AAPL?".to_string()
            }
        );
    }

    #[test]
    fn test_parse_chinese() {
        let cmd = Command::parse("/分析 AAPL").unwrap();
        assert_eq!(
            cmd,
            Command::Analyze {
                symbol: "AAPL".to_string()
            }
        );
    }

    #[test]
    fn test_parse_missing_arg() {
        let result = Command::parse("/analyze");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_compare_too_few() {
        let result = Command::parse("/compare AAPL");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_help() {
        let cmd = Command::parse("/help").unwrap();
        assert_eq!(cmd, Command::Help);

        let cmd = Command::parse("/帮助").unwrap();
        assert_eq!(cmd, Command::Help);
    }
}
