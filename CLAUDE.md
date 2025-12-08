# agent-rs Project Context

## Project Overview

Agent-rs is an exploratory Rust workspace for building LLM-powered agents. The project is in early stages with a focus on experimentation and learning. The architecture is modular and flexible to support various agent patterns and use cases.

## Architecture Principles

1. **Modular Design**: Each crate has a single, well-defined responsibility
2. **Provider Agnostic**: LLM provider abstractions allow easy swapping between OpenAI, Anthropic, Ollama, etc.
3. **Async First**: All I/O operations use Tokio async runtime for high performance
4. **Type Safety**: Leverage Rust's type system for correctness and reliability
5. **Flexibility**: Architecture supports experimentation and iteration

## Workspace Structure

```
agent-rs/
├── crates/
│   ├── agent-core/       # Core abstractions (Agent trait, Message, Context, Error)
│   ├── agent-llm/        # LLM provider abstraction (LLMProvider trait)
│   ├── agent-providers/  # Concrete providers (OpenAI, Anthropic, Ollama)
│   ├── agent-tools/      # Tool framework (Tool trait, ToolRegistry)
│   ├── agent-workflow/   # Multi-agent orchestration (Workflow)
│   ├── agent-derive/     # Procedural macros (future: #[derive(Agent)])
│   ├── agent-utils/      # Shared utilities (logging, config)
│   ├── agent-cli/        # CLI application (binary)
│   └── xtask/            # Project automation (binary)
├── examples/             # Example code
├── docs/                 # Documentation
├── .claude/              # Claude Code configuration
│   ├── commands/         # Custom slash commands
│   └── settings.json     # Claude Code settings
└── Cargo.toml            # Workspace root
```

### Crate Dependencies

- **agent-cli** → agent-workflow, agent-core, agent-utils
- **agent-workflow** → agent-core, agent-tools, agent-utils
- **agent-tools** → agent-core, agent-utils
- **agent-providers** → agent-llm, agent-core, agent-utils
- **agent-llm** → agent-core, agent-utils
- **agent-core** → agent-utils
- **agent-derive** → independent (proc-macro only)
- **agent-utils** → minimal dependencies (base layer)
- **xtask** → independent (build automation)

## Key Design Patterns

- **Trait-based abstractions**: Core functionality defined as traits (`Agent`, `LLMProvider`, `Tool`)
- **Feature flags**: Optional dependencies controlled via Cargo features (e.g., `anthropic`, `openai`, `ollama`)
- **Workspace dependencies**: Centralized version management in root Cargo.toml
- **Derive macros**: Future plans to reduce boilerplate with `#[derive(Agent)]`

## Development Workflow

### Critical Rules

1. **Before adding dependencies**: ALWAYS search for the latest stable version on crates.io
2. **Code style**: Run `cargo fmt` before committing
3. **Testing**: Maintain test coverage above 70%
4. **Documentation**: Document all public APIs with rustdoc comments
5. **Commits**: Use conventional commit format (feat:, fix:, docs:, etc.)

### Common Commands

- Build workspace: `/build` or `cargo build --workspace --all-features`
- Run tests: `/test` or `cargo test --workspace --all-features`
- Run lints: `/lint` or `cargo clippy --workspace --all-features`
- Check deps: `/check-deps`
- Update deps: `/update-deps` (includes automatic version search!)
- Generate coverage: `/coverage`

### Dependency Management Protocol

**CRITICAL**: Before adding or updating ANY dependency:

1. Use `/update-deps` command OR
2. Search crates.io manually: "crate-name latest version 2025"
3. Verify the latest stable version
4. Update workspace.dependencies in root Cargo.toml
5. Run `cargo update` and `cargo test`

Never hardcode versions without verifying they are the latest!

## Technology Stack

### Rust Version

- **Minimum**: Rust 1.85.0 (first version with Edition 2024 support)
- **Edition**: 2024 (stable since Feb 2025)
- **Current**: Latest stable is 1.91.1

### Key Dependencies (Latest Versions)

All dependencies are managed in workspace.dependencies:

- **tokio**: 1.48 - Async runtime
- **serde**: 1.0.228 - Serialization
- **reqwest**: 0.12.24 - HTTP client
- **clap**: 4.5.51 - CLI parsing
- **tracing**: 0.1.41 - Logging/tracing
- **anyhow**: 1.0 - Error handling (applications)
- **thiserror**: 2.0 - Error handling (libraries)
- **async-trait**: 0.1.89 - Async trait methods

### Testing Strategy

- **Unit tests**: In each crate's `src/` files using `#[cfg(test)]`
- **Integration tests**: Workspace-level tests in `tests/`
- **Mocking**: Use `mockall` for mocking LLM providers and external dependencies
- **Coverage target**: 70%+ overall, measured with tarpaulin

## Current Status

### Completed

- [x] Workspace architecture designed
- [x] All 9 crates scaffolded with basic structure
- [x] Core traits defined (Agent, LLMProvider, Tool)
- [x] Basic error handling implemented
- [x] Claude Code integration (commands, settings)
- [x] Project documentation (CLAUDE.md, README.md)

### In Progress

- [ ] Implement concrete LLM providers (Anthropic, OpenAI, Ollama)
- [ ] Build example tools (file operations, web search)
- [ ] Create workflow orchestration patterns
- [ ] Develop CLI commands
- [ ] Write comprehensive tests

### Future Plans

- [ ] Implement `#[derive(Agent)]` macro
- [ ] Add streaming support for LLM responses
- [ ] Create example applications
- [ ] Add telemetry and observability
- [ ] Consider adding: agent-storage, agent-rag, agent-embeddings

## Known Issues & Considerations

1. **Edition 2024**: Requires Rust 1.85.0+ (released Feb 2025)
2. **Async trait limitations**: Using `async-trait` crate for now; native async traits don't support dyn Trait yet
3. **Provider authentication**: Need to implement secure API key management
4. **Windows paths**: Project developed on Windows; use proper path handling

## References & Inspiration

### Documentation

- [Rust Edition 2024](https://blog.rust-lang.org/2025/02/20/Rust-1.85.0/)
- [Cargo Workspaces](https://doc.rust-lang.org/book/ch14-03-cargo-workspaces.html)
- [Tokio Async Runtime](https://tokio.rs/)

### Similar Projects

- [AutoAgents](https://github.com/liquidos-ai/AutoAgents) - Rust LLM agent framework
- [Kowalski](https://dev.to/yarenty/kowalski-the-rust-native-agentic-ai-framework-53k4) - Rust-native agentic AI
- [Rig](https://rig.rs/) - Rust library for LLM applications

## Tips for Claude Code

1. **Use slash commands**: They enforce best practices (like version checking)
2. **Check CLAUDE.md first**: This file contains project-specific context
3. **Follow dependency protocol**: Always search before adding/updating deps
4. **Run tests frequently**: Use `/test` to verify changes
5. **Keep it simple**: Avoid over-engineering; this is an exploratory project

## Quick Reference

### File Locations

- Main config: [Cargo.toml](c:\Users\qiufeng\Desktop\agent-rs\Cargo.toml)
- Core traits: [agent-core/src/agent.rs](c:\Users\qiufeng\Desktop\agent-rs\crates\agent-core\src\agent.rs)
- LLM abstraction: [agent-llm/src/provider.rs](c:\Users\qiufeng\Desktop\agent-rs\crates\agent-llm\src\provider.rs)
- Tool framework: [agent-tools/src/tool.rs](c:\Users\qiufeng\Desktop\agent-rs\crates\agent-tools\src\tool.rs)
- Workflow: [agent-workflow/src/workflow.rs](c:\Users\qiufeng\Desktop\agent-rs\crates\agent-workflow\src\workflow.rs)

### Useful Commands

```bash
# Build everything
cargo build --workspace --all-features

# Run all tests
cargo test --workspace --all-features

# Check formatting
cargo fmt --all -- --check

# Run clippy
cargo clippy --workspace --all-features -- -D warnings

# Update dependencies (manually)
cargo update

# Run specific crate
cargo run -p agent-cli

# Check for outdated deps
cargo outdated --workspace
```

### Automation (xtask)

```bash
# Use xtask for project automation
cargo xtask check-deps    # Check outdated dependencies
cargo xtask update-deps   # Update deps (with version search!)
cargo xtask test          # Run all tests
cargo xtask lint          # Run clippy
cargo xtask coverage      # Generate coverage report
```

---

**Last Updated**: 2025-12-09
**Rust Version**: 1.91.1
**Edition**: 2024
