---
description: Update dependencies interactively with version search
---

Interactive dependency update workflow:

**CRITICAL REQUIREMENT**: Before updating ANY dependency, you MUST search for the latest version on crates.io using WebSearch.

1. First, run cargo outdated to see what needs updating:
```bash
cargo outdated --workspace
```

2. For EACH outdated dependency:
   - Use WebSearch to find: "[crate-name] latest version crates.io 2025"
   - Verify the latest stable version
   - Ask me if I want to update it

3. Update the dependency in the root Cargo.toml workspace.dependencies section

4. After updates, run:
```bash
cargo update
cargo test --workspace --all-features
```

5. If tests pass, ask if I want to commit the changes

Never update a dependency without first searching for and confirming the latest version!
