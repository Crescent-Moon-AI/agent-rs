# agent-workflow

Multi-agent orchestration for agent-rs - workflows, pipelines, and agent coordination.

[![Crates.io](https://img.shields.io/crates/v/agent-workflow.svg)](https://crates.io/crates/agent-workflow)
[![Documentation](https://docs.rs/agent-workflow/badge.svg)](https://docs.rs/agent-workflow)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

Part of the [agent-rs](https://github.com/Crescent-Moon-AI/agent-rs) framework.

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
agent-workflow = "0.0.1-alpha.1"
```

## Features

- `Workflow` - define multi-step agent workflows
- `WorkflowBuilder` - fluent API for building workflows
- `WorkflowStep` - individual steps in a workflow
- Sequential and parallel step execution

## Usage

```rust
use agent_workflow::{Workflow, WorkflowBuilder, WorkflowStep};

async fn example() -> anyhow::Result<()> {
    let workflow = WorkflowBuilder::new()
        .add_step(WorkflowStep::new("step1", agent1))
        .add_step(WorkflowStep::new("step2", agent2))
        .build();

    let result = workflow.execute("initial input").await?;
    Ok(())
}
```

## License

MIT License - see [LICENSE](../../LICENSE) for details.
