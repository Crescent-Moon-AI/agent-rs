//! Response formatting utilities

use crate::engine::{AnalysisContext, AnalysisResult};
use crate::interface::BotPlatform;

pub trait Formatter: Send + Sync {
    fn platform(&self) -> BotPlatform;
    fn format_analysis(&self, result: &AnalysisResult, context: &AnalysisContext) -> String;
    fn format_table(&self, headers: &[String], rows: &[Vec<String>]) -> String;
    fn format_error(&self, error: &str) -> String;
    fn format_help(&self) -> String;
}

pub struct CliFormatter;

impl Formatter for CliFormatter {
    fn platform(&self) -> BotPlatform {
        BotPlatform::CLI
    }
    
    fn format_analysis(&self, result: &AnalysisResult, _context: &AnalysisContext) -> String {
        format!("{}\n\n{}", result.summary(), result.content)
    }
    
    fn format_table(&self, headers: &[String], rows: &[Vec<String>]) -> String {
        let mut output = String::new();
        output.push_str(&headers.join(" | "));
        output.push('\n');
        for row in rows {
            output.push_str(&row.join(" | "));
            output.push('\n');
        }
        output
    }
    
    fn format_error(&self, error: &str) -> String {
        format!("❌ Error: {}", error)
    }
    
    fn format_help(&self) -> String {
        "Stock Analysis Bot Commands:\n\
        /analyze <symbol> - Comprehensive analysis\n\
        /technical <symbol> - Technical analysis\n\
        /help - Show help\n\
        /exit - Exit".to_string()
    }
}

pub struct TelegramFormatter;

impl Formatter for TelegramFormatter {
    fn platform(&self) -> BotPlatform {
        BotPlatform::Telegram
    }
    
    fn format_analysis(&self, result: &AnalysisResult, _context: &AnalysisContext) -> String {
        format!("*{}*\n\n{}", result.summary(), result.content)
    }
    
    fn format_table(&self, headers: &[String], rows: &[Vec<String>]) -> String {
        let mut output = String::from("```\n");
        output.push_str(&headers.join(" | "));
        output.push('\n');
        for row in rows {
            output.push_str(&row.join(" | "));
            output.push('\n');
        }
        output.push_str("```");
        output
    }
    
    fn format_error(&self, error: &str) -> String {
        format!("❌ *Error:* {}", error)
    }
    
    fn format_help(&self) -> String {
        "*Stock Analysis Bot*\n\
        /analyze - Comprehensive analysis\n\
        /technical - Technical analysis\n\
        /help - Show help".to_string()
    }
}

pub struct FormatterFactory;

impl FormatterFactory {
    pub fn create(platform: BotPlatform) -> Box<dyn Formatter> {
        match platform {
            BotPlatform::CLI => Box::new(CliFormatter),
            BotPlatform::Telegram => Box::new(TelegramFormatter),
            _ => Box::new(CliFormatter),
        }
    }
}
