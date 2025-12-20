//! Bot platform interfaces
//!
//! Platform-agnostic interfaces for building stock analysis bots

pub mod interface;
pub mod session;
pub mod formatter;
pub mod message;

pub use interface::{BotInterface, BotPlatform, BotResponse};
pub use session::{SessionManager, UserSession, SessionStorage};
pub use formatter::{Formatter, FormatterFactory};
pub use message::{Message, MessageType};
