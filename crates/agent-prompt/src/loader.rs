//! File-based template loader
//!
//! This module provides [`FileLoader`] for loading templates from the filesystem.
//! It is gated behind the `file-loader` feature.

use crate::{JinjaTemplate, JinjaTemplateBuilder, Language, PromptError, Result};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// A file-based template loader
///
/// `FileLoader` loads templates from a directory structure. Templates are organized
/// by name and language, following the naming convention:
/// - `{name}_en.jinja` for English
/// - `{name}_zh.jinja` for Chinese
/// - `{name}_{lang_code}.jinja` for other languages
/// - `{name}.jinja` for single-language templates (defaults to English)
///
/// # Directory Structure
///
/// ```text
/// templates/
/// ├── greeting_en.jinja
/// ├── greeting_zh.jinja
/// ├── system_prompt_en.jinja
/// ├── system_prompt_zh.jinja
/// └── simple.jinja          # Defaults to English
/// ```
///
/// # Examples
///
/// ```ignore
/// use agent_prompt::FileLoader;
///
/// let loader = FileLoader::new("./templates");
///
/// // Load a specific template
/// let template = loader.load_template("greeting")?;
///
/// // Load all templates in the directory
/// let templates = loader.load_all()?;
/// ```
#[derive(Debug, Clone)]
pub struct FileLoader {
    base_path: PathBuf,
}

impl FileLoader {
    /// Create a new file loader with the given base path
    pub fn new(base_path: impl Into<PathBuf>) -> Self {
        Self {
            base_path: base_path.into(),
        }
    }

    /// Get the base path
    pub fn base_path(&self) -> &Path {
        &self.base_path
    }

    /// Load a single template by name
    ///
    /// This will search for files matching the pattern `{name}_{lang}.jinja` or
    /// `{name}.jinja` in the base directory.
    pub fn load_template(&self, name: &str) -> Result<JinjaTemplate> {
        let mut templates: HashMap<Language, String> = HashMap::new();

        // Try to find language-specific files
        let patterns = vec![
            (Language::English, format!("{}_en.jinja", name)),
            (Language::English, format!("{}_en.j2", name)),
            (Language::Chinese, format!("{}_zh.jinja", name)),
            (Language::Chinese, format!("{}_zh.j2", name)),
        ];

        for (lang, filename) in patterns {
            let path = self.base_path.join(&filename);
            if path.exists() {
                let content = std::fs::read_to_string(&path).map_err(|e| {
                    PromptError::FileLoadError {
                        path: path.display().to_string(),
                        detail: e.to_string(),
                    }
                })?;
                templates.insert(lang, content);
            }
        }

        // Try to find a single default file
        if templates.is_empty() {
            for ext in &["jinja", "j2"] {
                let path = self.base_path.join(format!("{}.{}", name, ext));
                if path.exists() {
                    let content = std::fs::read_to_string(&path).map_err(|e| {
                        PromptError::FileLoadError {
                            path: path.display().to_string(),
                            detail: e.to_string(),
                        }
                    })?;
                    templates.insert(Language::English, content);
                    break;
                }
            }
        }

        if templates.is_empty() {
            return Err(PromptError::FileLoadError {
                path: self.base_path.join(name).display().to_string(),
                detail: "No template files found".to_string(),
            });
        }

        // Build the template
        let mut builder = JinjaTemplateBuilder::new(name);
        for (lang, content) in templates {
            builder = builder.template(lang, content);
        }
        builder.build()
    }

    /// Load all templates from the base directory
    ///
    /// This scans the directory for `.jinja` and `.j2` files and groups them
    /// by template name.
    pub fn load_all(&self) -> Result<Vec<JinjaTemplate>> {
        let mut template_files: HashMap<String, HashMap<Language, String>> = HashMap::new();

        // Read directory
        let entries = std::fs::read_dir(&self.base_path).map_err(|e| {
            PromptError::FileLoadError {
                path: self.base_path.display().to_string(),
                detail: e.to_string(),
            }
        })?;

        for entry in entries {
            let entry = entry.map_err(PromptError::IoError)?;
            let path = entry.path();

            if !path.is_file() {
                continue;
            }

            let filename = match path.file_name().and_then(|n| n.to_str()) {
                Some(n) => n.to_string(),
                None => continue,
            };

            // Check for valid extensions
            if !filename.ends_with(".jinja") && !filename.ends_with(".j2") {
                continue;
            }

            // Parse filename to extract name and language
            let (name, lang) = self.parse_filename(&filename)?;

            // Read file content
            let content =
                std::fs::read_to_string(&path).map_err(|e| PromptError::FileLoadError {
                    path: path.display().to_string(),
                    detail: e.to_string(),
                })?;

            // Add to map
            template_files
                .entry(name)
                .or_default()
                .insert(lang, content);
        }

        // Build templates
        let mut templates = Vec::new();
        for (name, lang_contents) in template_files {
            let mut builder = JinjaTemplateBuilder::new(&name);
            for (lang, content) in lang_contents {
                builder = builder.template(lang, content);
            }
            templates.push(builder.build()?);
        }

        Ok(templates)
    }

    /// Parse a filename to extract template name and language
    fn parse_filename(&self, filename: &str) -> Result<(String, Language)> {
        // Remove extension
        let stem = filename
            .strip_suffix(".jinja")
            .or_else(|| filename.strip_suffix(".j2"))
            .unwrap_or(filename);

        // Check for language suffix
        if let Some(name) = stem.strip_suffix("_en") {
            return Ok((name.to_string(), Language::English));
        }
        if let Some(name) = stem.strip_suffix("_zh") {
            return Ok((name.to_string(), Language::Chinese));
        }

        // Check for other language codes (2 characters after underscore)
        if stem.len() > 3 {
            let potential_split = stem.len() - 3;
            if stem.chars().nth(potential_split) == Some('_') {
                let lang_code = &stem[potential_split + 1..];
                if lang_code.len() == 2 && lang_code.chars().all(|c| c.is_ascii_lowercase()) {
                    let name = &stem[..potential_split];
                    let lang = Language::from_code(lang_code);
                    return Ok((name.to_string(), lang));
                }
            }
        }

        // No language suffix, default to English
        Ok((stem.to_string(), Language::English))
    }

    /// Check if the base directory exists
    pub fn exists(&self) -> bool {
        self.base_path.exists() && self.base_path.is_dir()
    }

    /// List all template names (without loading content)
    pub fn list_templates(&self) -> Result<Vec<String>> {
        let mut names = std::collections::HashSet::new();

        let entries = std::fs::read_dir(&self.base_path).map_err(|e| {
            PromptError::FileLoadError {
                path: self.base_path.display().to_string(),
                detail: e.to_string(),
            }
        })?;

        for entry in entries {
            let entry = entry.map_err(PromptError::IoError)?;
            let path = entry.path();

            if !path.is_file() {
                continue;
            }

            if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
                if filename.ends_with(".jinja") || filename.ends_with(".j2") {
                    let (name, _) = self.parse_filename(filename)?;
                    names.insert(name);
                }
            }
        }

        let mut result: Vec<_> = names.into_iter().collect();
        result.sort();
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::PromptTemplate;
    use std::fs;
    use tempfile::tempdir;

    fn create_test_file(dir: &Path, name: &str, content: &str) {
        let path = dir.join(name);
        fs::write(&path, content).unwrap();
    }

    #[test]
    fn test_parse_filename() {
        let loader = FileLoader::new("./");

        let (name, lang) = loader.parse_filename("greeting_en.jinja").unwrap();
        assert_eq!(name, "greeting");
        assert_eq!(lang, Language::English);

        let (name, lang) = loader.parse_filename("greeting_zh.jinja").unwrap();
        assert_eq!(name, "greeting");
        assert_eq!(lang, Language::Chinese);

        let (name, lang) = loader.parse_filename("simple.jinja").unwrap();
        assert_eq!(name, "simple");
        assert_eq!(lang, Language::English);

        let (name, lang) = loader.parse_filename("prompt_ja.j2").unwrap();
        assert_eq!(name, "prompt");
        assert_eq!(lang, Language::Other("ja".to_string()));
    }

    #[test]
    fn test_load_template() {
        let dir = tempdir().unwrap();

        create_test_file(dir.path(), "greeting_en.jinja", "Hello, {{ name }}!");
        create_test_file(dir.path(), "greeting_zh.jinja", "你好，{{ name }}！");

        let loader = FileLoader::new(dir.path());
        let template = loader.load_template("greeting").unwrap();

        assert_eq!(template.name(), "greeting");
        assert!(template.supports_language(&Language::English));
        assert!(template.supports_language(&Language::Chinese));
    }

    #[test]
    fn test_load_single_file() {
        let dir = tempdir().unwrap();

        create_test_file(dir.path(), "simple.jinja", "Hello!");

        let loader = FileLoader::new(dir.path());
        let template = loader.load_template("simple").unwrap();

        assert_eq!(template.name(), "simple");
        assert!(template.supports_language(&Language::English));
    }

    #[test]
    fn test_load_all() {
        let dir = tempdir().unwrap();

        create_test_file(dir.path(), "greeting_en.jinja", "Hello!");
        create_test_file(dir.path(), "greeting_zh.jinja", "你好！");
        create_test_file(dir.path(), "farewell_en.jinja", "Goodbye!");

        let loader = FileLoader::new(dir.path());
        let templates = loader.load_all().unwrap();

        assert_eq!(templates.len(), 2);

        let names: Vec<_> = templates.iter().map(|t| t.name()).collect();
        assert!(names.contains(&"greeting"));
        assert!(names.contains(&"farewell"));
    }

    #[test]
    fn test_load_not_found() {
        let dir = tempdir().unwrap();
        let loader = FileLoader::new(dir.path());

        let result = loader.load_template("nonexistent");
        assert!(result.is_err());
    }

    #[test]
    fn test_list_templates() {
        let dir = tempdir().unwrap();

        create_test_file(dir.path(), "a_en.jinja", "A");
        create_test_file(dir.path(), "a_zh.jinja", "A");
        create_test_file(dir.path(), "b.jinja", "B");

        let loader = FileLoader::new(dir.path());
        let names = loader.list_templates().unwrap();

        assert_eq!(names, vec!["a", "b"]);
    }

    #[test]
    fn test_exists() {
        let dir = tempdir().unwrap();
        let loader = FileLoader::new(dir.path());
        assert!(loader.exists());

        let loader = FileLoader::new("/nonexistent/path");
        assert!(!loader.exists());
    }
}
