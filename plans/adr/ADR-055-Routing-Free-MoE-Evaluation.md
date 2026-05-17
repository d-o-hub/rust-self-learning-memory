# ADR-055: Evaluation of Routing-Free MoE for DyMoE Replacement

**Status**: Evaluation
**Date**: 2026-05-01
**Deciders**: WG-125 Analysis
**Related**: WG-089 (DyMoE), ADR-053 (Comprehensive Analysis)

---

## Context

Current routing-drift protection in `do-memory-core` is based on **LLaVA-DyMoE** (CVPR 2026). It utilizes a centralized `PatternAffinityClassifier` to compute relative affinity ($D_{rel}$) between existing and new pattern clusters, gating mutation via an `EpisodeAssignmentGuard`.

**Routing-Free MoE** (arXiv:2604.00801, April 2026) proposes a fully decentralized architecture where experts determine their own activation via internal confidence scores, learnable per-expert biases, and a global threshold. It eliminates centralized routers, Softmax, and Top-K selection, addressing the routing-drift problem at the source.

WG-125 requires an evaluation of Routing-Free MoE as a potential replacement for the DyMoE-based logic currently implemented.

## Comparison

| Feature | DyMoE (Current) | Routing-Free MoE (arXiv:2604.00801) |
|---------|-----------------|-----------------------------------|
| **Architecture** | Centralized router/classifier | Decentralized, self-activating experts |
| **Selection** | Top-K or gated relative affinity | Bias + Threshold ($G_i(\mathbf{x}) \ge \theta$) |
| **Drift Protection** | $D_{rel}$ (Relative Affinity) | Elimination of centralized competition |
| **Load Balancing** | Static or Dual Reward signals | Unified $\mathcal{L}_{EB} + \mathcal{L}_{TB}$ interpolation |
| **Scalability** | O(N) router bottleneck | Parallelizable scoring, lower overhead |
| **Metrics** | Stability vs. Novelty | Token vs. Expert Balance |

## Evaluation Findings

### 1. Applicability to Episodic Memory
Routing-Free MoE is primarily designed for high-throughput LLM FFN layers. In `do-memory-core`, "routing" refers to the assignment of episodes to pattern clusters.
- **DyMoE Strength**: The $D_{rel}$ metric is highly effective at detecting "ambiguous" episodes that sit between two specialized patterns, which is critical for maintaining pattern purity.
- **Routing-Free Strength**: RF-MoE allows patterns (experts) to "opt-in" to an episode independently. This naturally supports multi-pattern assignment without a centralized Top-K constraint.

### 2. Implementation Complexity
- Replacing DyMoE with a full RF-MoE implementation would require adding learnable biases to `Pattern` entities and implementing the unified $\mathcal{L}_{LB}$ loss in the training/extraction loop.
- The current `DualRewardScore` provides stability/novelty signals that are valuable for the agent's meta-cognition, which the RF-MoE paper does not explicitly replace (it focuses on training efficiency and performance).

### 3. Performance & Stability
The RF-MoE paper demonstrates superior training stability and inference throughput, especially as the number of experts (patterns) scales. However, for the current pattern counts (< 1,000), the centralized bottleneck of DyMoE is negligible.

## Recommendation

**Maintain DyMoE for v0.1.32, but incorporate Routing-Free principles in v0.1.33.**

Full replacement is not recommended at this stage because the `DualRewardScore` and $D_{rel}$ gating provide specific safeguards against "concept corruption" in episodic memory that RF-MoE solves via gradient flow—which is less applicable to the discrete, often non-differentiable pattern extraction steps in our current pipeline.

### Proposed Path: "Self-Learning Patterns"
Instead of a full swap, we should evolve the `Pattern` architecture:
1. **Decentralized Activation**: Allow each `Pattern` to store its own `activation_threshold` (bias analog).
2. **Global Sparsity**: Implement a global `theta` for retrieval to allow users to trade precision for recall without retraining.
3. **Hybrid Reward**: Keep `DualRewardScore` but use RF-MoE's $\mu$-interpolation for balancing expert load during long-term pattern consolidation.

## Decision

We will NOT replace DyMoE with Routing-Free MoE in the upcoming sprint. Instead, we will:
1. Retain the current `PatternAffinityClassifier` as the primary drift protection.
2. Update the `Pattern` struct in `do-memory-core` to include a `bias` field for future RF-MoE inspired self-activation experiments.
3. Keep this evaluation as a reference for the next major refactor of the `patterns` module.

## References

- Routing-Free MoE: <https://arxiv.org/abs/2604.00801>
- LLaVA-DyMoE: <https://zhaoc5.github.io/DyMoE/>
- Reference Impl: <https://github.com/liuyilun2000/RoutingFreeMoE/>
