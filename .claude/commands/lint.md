---
description: Run clippy lints across the workspace
---

Run Clippy with strict lints:

```bash
cargo clippy --workspace --all-features -- -D warnings -W clippy::pedantic -W clippy::nursery
```

Show any warnings or errors and suggest fixes. Focus on:
1. Performance issues
2. Correctness problems
3. Idiomatic Rust patterns
4. Potential bugs

If there are many warnings, prioritize the most important ones.
