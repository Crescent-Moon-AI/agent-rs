//! Platform-specific bot implementations

pub mod cli;
pub mod telegram;
pub mod dingtalk;
pub mod feishu;

pub use cli::CliBot;
pub use telegram::{TelegramBot, TelegramConfig};
pub use dingtalk::{DingTalkBot, DingTalkConfig};
pub use feishu::{FeishuBot, FeishuConfig};
