//! Fluent prompt builder
//!
//! This module provides [`PromptBuilder`], a fluent API for constructing prompts
//! programmatically with sections, conditionals, and formatting.

use crate::Language;

/// A fluent builder for constructing prompts
///
/// `PromptBuilder` provides a convenient way to build prompts piece by piece,
/// especially useful for dynamic prompt construction where parts may be
/// conditional or data-driven.
///
/// # Examples
///
/// ```
/// use agent_prompt::PromptBuilder;
///
/// let prompt = PromptBuilder::new()
///     .text("You are a helpful assistant.")
///     .newline()
///     .section("Your Capabilities")
///     .bullet("Answer questions")
///     .bullet("Provide explanations")
///     .when(true, "\nYou have access to tools.")
///     .build();
///
/// assert!(prompt.contains("Your Capabilities"));
/// assert!(prompt.contains("- Answer questions"));
/// ```
#[derive(Debug, Clone)]
pub struct PromptBuilder {
    parts: Vec<String>,
    language: Language,
}

impl PromptBuilder {
    /// Create a new prompt builder
    pub fn new() -> Self {
        Self {
            parts: Vec::new(),
            language: Language::English,
        }
    }

    /// Set the language context (for future language-aware features)
    pub fn language(mut self, lang: Language) -> Self {
        self.language = lang;
        self
    }

    /// Add static text
    ///
    /// # Examples
    ///
    /// ```
    /// use agent_prompt::PromptBuilder;
    ///
    /// let prompt = PromptBuilder::new()
    ///     .text("Hello, ")
    ///     .text("World!")
    ///     .build();
    /// assert_eq!(prompt, "Hello, World!");
    /// ```
    pub fn text(mut self, content: impl Into<String>) -> Self {
        self.parts.push(content.into());
        self
    }

    /// Add a newline
    pub fn newline(self) -> Self {
        self.text("\n")
    }

    /// Add multiple newlines
    pub fn newlines(self, count: usize) -> Self {
        self.text("\n".repeat(count))
    }

    /// Add a blank line (two newlines)
    pub fn blank_line(self) -> Self {
        self.text("\n\n")
    }

    /// Add a section header (markdown h2)
    ///
    /// # Examples
    ///
    /// ```
    /// use agent_prompt::PromptBuilder;
    ///
    /// let prompt = PromptBuilder::new()
    ///     .section("Overview")
    ///     .text("This is the overview section.")
    ///     .build();
    /// assert!(prompt.contains("## Overview"));
    /// ```
    pub fn section(self, title: impl Into<String>) -> Self {
        self.text(format!("\n## {}\n", title.into()))
    }

    /// Add a subsection header (markdown h3)
    pub fn subsection(self, title: impl Into<String>) -> Self {
        self.text(format!("\n### {}\n", title.into()))
    }

    /// Add a header at a specific level (h1-h6)
    pub fn header(self, level: u8, title: impl Into<String>) -> Self {
        let hashes = "#".repeat(level.clamp(1, 6) as usize);
        self.text(format!("\n{} {}\n", hashes, title.into()))
    }

    /// Add content conditionally
    ///
    /// # Examples
    ///
    /// ```
    /// use agent_prompt::PromptBuilder;
    ///
    /// let include_extra = true;
    /// let prompt = PromptBuilder::new()
    ///     .text("Base content")
    ///     .when(include_extra, "\nExtra content")
    ///     .build();
    /// assert!(prompt.contains("Extra content"));
    ///
    /// let prompt2 = PromptBuilder::new()
    ///     .text("Base content")
    ///     .when(false, "\nExtra content")
    ///     .build();
    /// assert!(!prompt2.contains("Extra content"));
    /// ```
    pub fn when(self, condition: bool, content: impl Into<String>) -> Self {
        if condition {
            self.text(content)
        } else {
            self
        }
    }

    /// Add content conditionally, with an else case
    pub fn when_else(
        self,
        condition: bool,
        if_true: impl Into<String>,
        if_false: impl Into<String>,
    ) -> Self {
        if condition {
            self.text(if_true)
        } else {
            self.text(if_false)
        }
    }

    /// Add a bullet point
    ///
    /// # Examples
    ///
    /// ```
    /// use agent_prompt::PromptBuilder;
    ///
    /// let prompt = PromptBuilder::new()
    ///     .bullet("First item")
    ///     .bullet("Second item")
    ///     .build();
    /// assert!(prompt.contains("- First item"));
    /// assert!(prompt.contains("- Second item"));
    /// ```
    pub fn bullet(self, content: impl Into<String>) -> Self {
        self.text(format!("- {}\n", content.into()))
    }

    /// Add multiple bullet points
    pub fn bullets<I, S>(mut self, items: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        for item in items {
            self = self.bullet(item);
        }
        self
    }

    /// Add a numbered item
    ///
    /// # Examples
    ///
    /// ```
    /// use agent_prompt::PromptBuilder;
    ///
    /// let prompt = PromptBuilder::new()
    ///     .numbered(1, "First step")
    ///     .numbered(2, "Second step")
    ///     .build();
    /// assert!(prompt.contains("1. First step"));
    /// ```
    pub fn numbered(self, num: usize, content: impl Into<String>) -> Self {
        self.text(format!("{}. {}\n", num, content.into()))
    }

    /// Add multiple numbered items starting from 1
    pub fn numbered_list<I, S>(mut self, items: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        for (i, item) in items.into_iter().enumerate() {
            self = self.numbered(i + 1, item);
        }
        self
    }

    /// Add a code block
    ///
    /// # Examples
    ///
    /// ```
    /// use agent_prompt::PromptBuilder;
    ///
    /// let prompt = PromptBuilder::new()
    ///     .code_block("rust", "fn main() {}")
    ///     .build();
    /// assert!(prompt.contains("```rust"));
    /// assert!(prompt.contains("fn main()"));
    /// ```
    pub fn code_block(self, language: impl Into<String>, code: impl Into<String>) -> Self {
        self.text(format!("```{}\n{}\n```\n", language.into(), code.into()))
    }

    /// Add inline code
    pub fn code(self, code: impl Into<String>) -> Self {
        self.text(format!("`{}`", code.into()))
    }

    /// Add bold text
    pub fn bold(self, text: impl Into<String>) -> Self {
        self.text(format!("**{}**", text.into()))
    }

    /// Add italic text
    pub fn italic(self, text: impl Into<String>) -> Self {
        self.text(format!("*{}*", text.into()))
    }

    /// Add a horizontal rule
    pub fn horizontal_rule(self) -> Self {
        self.text("\n---\n")
    }

    /// Add a quote/blockquote
    pub fn quote(self, text: impl Into<String>) -> Self {
        let quoted = text
            .into()
            .lines()
            .map(|line| format!("> {}", line))
            .collect::<Vec<_>>()
            .join("\n");
        self.text(format!("{}\n", quoted))
    }

    /// Add a key-value pair
    pub fn field(self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.text(format!("**{}**: {}\n", key.into(), value.into()))
    }

    /// Join parts with a custom separator
    pub fn join_with(self, separator: &str) -> String {
        self.parts.join(separator)
    }

    /// Build the final prompt string
    pub fn build(self) -> String {
        self.parts.join("")
    }

    /// Build with trimmed whitespace
    pub fn build_trimmed(self) -> String {
        self.build().trim().to_string()
    }

    /// Get the current language
    pub fn get_language(&self) -> &Language {
        &self.language
    }

    /// Check if the builder is empty
    pub fn is_empty(&self) -> bool {
        self.parts.is_empty()
    }

    /// Get the number of parts
    pub fn parts_count(&self) -> usize {
        self.parts.len()
    }
}

impl Default for PromptBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl From<PromptBuilder> for String {
    fn from(builder: PromptBuilder) -> Self {
        builder.build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_text() {
        let prompt = PromptBuilder::new().text("Hello").text(", World!").build();
        assert_eq!(prompt, "Hello, World!");
    }

    #[test]
    fn test_newlines() {
        let prompt = PromptBuilder::new()
            .text("Line 1")
            .newline()
            .text("Line 2")
            .build();
        assert_eq!(prompt, "Line 1\nLine 2");
    }

    #[test]
    fn test_blank_line() {
        let prompt = PromptBuilder::new()
            .text("Paragraph 1")
            .blank_line()
            .text("Paragraph 2")
            .build();
        assert_eq!(prompt, "Paragraph 1\n\nParagraph 2");
    }

    #[test]
    fn test_sections() {
        let prompt = PromptBuilder::new()
            .section("Main Section")
            .subsection("Sub Section")
            .build();
        assert!(prompt.contains("## Main Section"));
        assert!(prompt.contains("### Sub Section"));
    }

    #[test]
    fn test_conditional() {
        let prompt = PromptBuilder::new()
            .text("Base")
            .when(true, " - Included")
            .when(false, " - Excluded")
            .build();
        assert_eq!(prompt, "Base - Included");
    }

    #[test]
    fn test_when_else() {
        let prompt = PromptBuilder::new()
            .when_else(true, "Yes", "No")
            .build();
        assert_eq!(prompt, "Yes");

        let prompt2 = PromptBuilder::new()
            .when_else(false, "Yes", "No")
            .build();
        assert_eq!(prompt2, "No");
    }

    #[test]
    fn test_bullets() {
        let prompt = PromptBuilder::new()
            .bullets(vec!["One", "Two", "Three"])
            .build();
        assert!(prompt.contains("- One"));
        assert!(prompt.contains("- Two"));
        assert!(prompt.contains("- Three"));
    }

    #[test]
    fn test_numbered_list() {
        let prompt = PromptBuilder::new()
            .numbered_list(vec!["First", "Second", "Third"])
            .build();
        assert!(prompt.contains("1. First"));
        assert!(prompt.contains("2. Second"));
        assert!(prompt.contains("3. Third"));
    }

    #[test]
    fn test_code_block() {
        let prompt = PromptBuilder::new()
            .code_block("python", "print('hello')")
            .build();
        assert!(prompt.contains("```python"));
        assert!(prompt.contains("print('hello')"));
        assert!(prompt.contains("```"));
    }

    #[test]
    fn test_formatting() {
        let prompt = PromptBuilder::new()
            .bold("Bold")
            .text(" and ")
            .italic("Italic")
            .text(" and ")
            .code("code")
            .build();
        assert_eq!(prompt, "**Bold** and *Italic* and `code`");
    }

    #[test]
    fn test_quote() {
        let prompt = PromptBuilder::new()
            .quote("Line 1\nLine 2")
            .build();
        assert!(prompt.contains("> Line 1"));
        assert!(prompt.contains("> Line 2"));
    }

    #[test]
    fn test_field() {
        let prompt = PromptBuilder::new()
            .field("Name", "John")
            .field("Age", "30")
            .build();
        assert!(prompt.contains("**Name**: John"));
        assert!(prompt.contains("**Age**: 30"));
    }

    #[test]
    fn test_build_trimmed() {
        let prompt = PromptBuilder::new()
            .newline()
            .text("Content")
            .newline()
            .build_trimmed();
        assert_eq!(prompt, "Content");
    }

    #[test]
    fn test_is_empty() {
        let builder = PromptBuilder::new();
        assert!(builder.is_empty());

        let builder = PromptBuilder::new().text("Hello");
        assert!(!builder.is_empty());
    }

    #[test]
    fn test_into_string() {
        let builder = PromptBuilder::new().text("Hello");
        let s: String = builder.into();
        assert_eq!(s, "Hello");
    }

    #[test]
    fn test_complex_prompt() {
        let prompt = PromptBuilder::new()
            .text("You are a helpful assistant.")
            .blank_line()
            .section("Your Capabilities")
            .bullets(vec![
                "Answer questions accurately",
                "Provide detailed explanations",
                "Help with code",
            ])
            .section("Guidelines")
            .numbered_list(vec![
                "Be concise",
                "Be accurate",
                "Be helpful",
            ])
            .when(true, "\n**Note**: Always be polite.")
            .build();

        assert!(prompt.contains("You are a helpful assistant."));
        assert!(prompt.contains("## Your Capabilities"));
        assert!(prompt.contains("- Answer questions accurately"));
        assert!(prompt.contains("1. Be concise"));
        assert!(prompt.contains("**Note**: Always be polite."));
    }

    #[test]
    fn test_language() {
        let builder = PromptBuilder::new().language(Language::Chinese);
        assert_eq!(builder.get_language(), &Language::Chinese);
    }
}
