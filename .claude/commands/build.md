---
description: Build all workspace crates
---

Build all workspace members with full output:

```bash
cargo build --workspace --all-features
```

If the build succeeds, also verify that all tests compile:

```bash
cargo test --workspace --all-features --no-run
```

Report any build errors and suggest fixes.
