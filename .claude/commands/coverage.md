---
description: Generate test coverage report
---

Generate test coverage report using tarpaulin (if available):

First, check if cargo-tarpaulin is installed:

```bash
cargo tarpaulin --version || cargo install cargo-tarpaulin
```

Then generate the coverage report:

```bash
cargo tarpaulin --workspace --all-features --out Html --output-dir ./target/coverage
```

Summarize:
1. Overall coverage percentage
2. Crates with low coverage (< 70%)
3. Specific modules/functions that need more tests

Open the report if requested:

```bash
start ./target/coverage/index.html
```
