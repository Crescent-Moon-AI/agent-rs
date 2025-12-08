---
description: Run all tests across the workspace
---

Run comprehensive tests across all workspace crates:

```bash
cargo test --workspace --all-features -- --nocapture
```

Pay attention to:
1. Any failing tests
2. Tests that panic
3. Tests that are ignored

If there are failures, show the test output and suggest fixes.
