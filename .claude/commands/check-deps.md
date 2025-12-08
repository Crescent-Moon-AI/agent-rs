---
description: Check for outdated dependencies
---

Check for outdated dependencies in the workspace.

First, check if cargo-outdated is installed:

```bash
cargo outdated --version || cargo install cargo-outdated
```

Then run the check:

```bash
cargo outdated --workspace --root-deps-only
```

Provide a summary of:
1. Dependencies that have major version updates available
2. Dependencies with minor/patch updates
3. Recommendations for which to update

**IMPORTANT**: Before suggesting any dependency updates, search crates.io for the latest stable version to confirm.
