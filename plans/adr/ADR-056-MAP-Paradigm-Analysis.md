# ADR-056: MAP Paradigm Impact Analysis

## 1. Relevance Score
**7/10**

## 2. Verdict
The MAP (Map-then-Act) paradigm is a highly relevant conceptual shift for `rust-self-learning-memory`, moving away from reactive "trial-and-error" to proactive "explore-and-model" execution. While the paper's specific evaluation is focused on LLM interactions in text-based games and structured environments (ALFWorld, ARC-AGI-3), the core insight—constructing a causal, structured cognitive map (spatial layouts, affordances, constraints) prior to task execution—directly complements our GOAP (Goal-Oriented Action Planning) agent skill and episodic/semantic memory retrieval pipelines. However, directly porting MAP's LLM-driven "exploration phase" is too slow and non-deterministic for our strict performance constraints (WASM compatibility, DuckDB/Redb storage limits, SIMD-accelerated cascading retrieval). Instead, we can adapt the *concept* of the cognitive map as a structured context payload retrieved *before* task planning, utilizing our existing HDC/DyMoE pattern systems to pre-warm the agent's context rather than forcing it to blindly explore first.

## 3. Idea Evaluation Table

| Paper Idea | Fit for Project | Why it matters | Implementation Difficulty | Expected Benefit |
| :--- | :--- | :--- | :--- | :--- |
| **Decoupling Understanding from Execution** | High | Prevents GOAP agents from getting stuck in reactive failure loops by ensuring they understand the "environment" (API schemas, codebase layouts, tool limits) first. | Medium | High: Reduction in redundant tool calls and token usage; faster goal resolution. |
| **Cross-Task Global Exploration** | Low (Mostly Irrelevant) | Autonomous wandering to build priors is too expensive and risky for a local codebase/MCP tool environment. | High | Low: We rely on static analysis and documentation (`AGENTS.md`) for priors, not random exploration. |
| **Task-Specific Cognitive Mapping** | High | Extracting structured layouts (e.g., file trees, API dependencies) and affordances (tool parameters) *before* planning aligns with our GoAP ANALYZE phase. | Medium | High: More accurate `set_plan` generation and fewer `ExecutionResult::Timeout` or blocked results. |
| **MAP-2K Distillation (SFT)** | Low | We do not train local LLMs; we manage memory and state for frozen models via RAG/HDC retrieval. | Very High | None: Out of scope for a Rust memory/retrieval engine. |
| **Dual-Convergence Stopping Criterion** | Partial | Knowing when "exploration" is complete based on Novelty and Knowledge Increment maps well to our DyMoE cluster management (novelty/stability scoring). | Medium | Medium: Can prevent over-retrieval or excessive context stuffing during the ANALYZE phase. |

## 4. Concrete Impact on Current Architecture
- **GOAP Agent Skill**: The `SKILL.md` methodology already has an `ANALYZE` phase, but it currently relies on reading `ADR`s and passive instruction. MAP suggests formalizing this into a "Mapping" sub-routine: proactively mapping codebase dependencies or tool constraints *before* calling `set_plan`.
- **Memory Storage (`do-memory-storage-duckdb`)**: We could define a new table/struct for `CognitiveMap` or `TaskMap`—a structured summary of an environment state that is cached and updated, separate from raw `Episode` logs.
- **Retrieval Pipeline (`do-memory-core`)**: The 4-tier CSM cascade currently retrieves *historical episodes*. We could adapt it to retrieve *Environment Maps* (affordances, constraints) as a distinct context layer injected before action generation.
- **DyMoE Architecture**: The paper's novelty/knowledge increment triggers closely mirror our `DualRewardScore` triggers for cluster splitting/merging. MAP's logic validates our existing approach but suggests applying it to state-space mapping rather than just pattern categorization.

## 5. Recommended Changes Now
1. **Enhance GOAP `ANALYZE` Phase**: Update `plans/adr/ADR-022-GOAP-Agent-System.md` or the `goap-agent/methodology.md` to mandate a formal "Environment Mapping" step (e.g., running `list_files`, exploring API endpoints) before issuing `set_plan`, explicitly preventing blind "Act-during-Think" execution.
2. **Implement `EnvironmentMap` Struct**: Add a lightweight struct in `do-memory-core::types` alongside `Episode` and `Pattern`, capable of storing spatial/dependency layouts and tool affordances, serialized via `postcard` (avoiding internally tagged enums).
3. **Pre-warm Context**: Modify `do-memory-mcp` to optionally accept and inject a `TaskMap` into the prompt context prior to tool execution, explicitly separating the "What is the world?" context from the "What should I do?" instruction.

## 6. Recommended Experiments
- **Experiment 1: Map-then-Plan GOAP**: Run a complex multi-step task using two GOAP strategies. Strategy A: Standard reactive planning. Strategy B: Explicit "Mapping" phase that generates a markdown layout of the relevant codebase/tools before planning. Compare latency, token usage, and success rate.
- **Experiment 2: DyMoE for State Novelty**: Adapt the `DualRewardScore` logic used for clustering to score the "novelty" of codebase/tool exploration. Use the `should_spawn_new_cluster` threshold to dynamically decide when the agent has explored enough of the environment and is ready to act.

## 7. Not Worth Implementing
- **Autonomous Global Exploration**: Do not implement a background process that autonomously randomly executes MCP tools or reads random files just to build a map. It violates safety and resource constraints.
- **Synthetic Trajectory Distillation (MAP-2K)**: Fine-tuning local models is out of scope for this memory system layer.

## 8. Final Decision: ADAPT
Adapt the core philosophy of "Map-then-Act" to our GOAP workflow and memory structures. Specifically, adapt the concept of decoupling environment mapping (codebase exploration, API discovery) from task execution, leveraging our existing memory and DyMoE architectures to store and retrieve these "cognitive maps" efficiently. Do not adopt the expensive active exploration or model fine-tuning components.
