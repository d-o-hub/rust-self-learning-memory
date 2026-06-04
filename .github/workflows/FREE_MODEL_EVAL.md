# Free OpenRouter Models — Eval Analysis for AI Slop Detection
# Source: openrouter.ai/collections/free-models (March 2026 usage data)
# Task: Binary text classification (SPAM vs LEGITIMATE) + confidence + reason as JSON

## Router Strategy: Use `openrouter/free` (NOT a hardcoded model list)
# Released: Feb 1, 2026 — 200K context
# Key: auto-selects from ALL free models, filters by capability (structured outputs, etc.)
# This means: as new free models are added, you automatically get them. No maintenance.

MODELS:
  - id: openrouter/free
    strategy: RECOMMENDED — single model ID, zero maintenance
    notes: >
      Auto-selects randomly from available free models that match your request's
      required features. Pass response_format: {type: json_object} to ensure it
      only picks models that support structured output — which all top models do.

## Individual Model Eval (for awareness / potential pinning)
# Ranked by March 2026 usage volume × suitability for classification tasks

  - id: stepfun/step-3.5-flash:free
    tokens_used: 1.65T       # #1 most-used free model
    params: 196B MoE (11B active)
    context: 256K
    reasoning: true
    eval_score: 9/10
    verdict: BEST for slop detection — fastest reasoning model, huge usage = stable
    notes: >
      Sparse MoE means low-latency. Reasoning mode catches subtle soft-sell patterns
      ("feel free to close") that pure instruction models may miss. Very reliable JSON.

  - id: openai/gpt-oss-120b:free
    tokens_used: 1.13B
    params: 120B MoE (5.1B active)
    context: 131K
    reasoning: true
    eval_score: 9/10
    verdict: EXCELLENT — OpenAI architecture means very reliable JSON + classification
    notes: >
      Native function calling + structured outputs. Reasoning configurable.
      Strong instruction following = consistent {verdict, confidence, reason} output.

  - id: nvidia/nemotron-3-super-120b-a12b:free
    tokens_used: 614B
    params: 120B MoE (12B active)
    context: 262K
    reasoning: true
    eval_score: 8/10
    verdict: EXCELLENT — #4 in programming benchmarks, multi-agent trained
    notes: >
      1M token context window (262K served free). Very strong at structured tasks
      and instruction following. RL-trained on 10+ environments → nuanced reasoning.

  - id: qwen/qwen3-coder:free
    tokens_used: 1.63B
    params: 480B MoE (35B active)
    context: 262K
    reasoning: true
    eval_score: 8/10
    verdict: VERY GOOD — massive model, strong instruction following
    notes: >
      Primarily code-focused but excellent at structured JSON output and classification.
      480B total params is the largest free model available. May be slow.

  - id: qwen/qwen3-next-80b-a3b-instruct:free
    tokens_used: 1.43B
    params: 80B MoE (3B active)
    context: 262K
    reasoning: false   # stable non-thinking by design
    eval_score: 8/10
    verdict: VERY GOOD — optimized for RAG/tool use, deterministic outputs
    notes: >
      No chain-of-thought = faster. Designed for "final answers rather than
      visible CoT" which is exactly what we want for classification.

  - id: arcee-ai/trinity-large-preview:free
    tokens_used: 158B
    params: 400B MoE (13B active)
    context: 131K
    reasoning: false
    eval_score: 7/10
    verdict: GOOD — huge model but chat/creative focused, not optimal for classification
    notes: >
      Trained for "creative writing, roleplay, voice assistance". Reliable but may
      over-elaborate the reason field. Fine as a fallback.

  - id: z-ai/glm-4.5-air:free
    tokens_used: 57.8B
    params: MoE (compact)
    context: 131K
    reasoning: configurable  # thinking mode toggle
    eval_score: 7/10
    verdict: GOOD — thinking mode useful for ambiguous cases, agent-centric
    notes: >
      Hybrid thinking/non-thinking mode is a nice feature. Use thinking=false for
      low-latency, thinking=true for borderline cases.

  - id: nvidia/nemotron-3-nano-30b-a3b:free
    tokens_used: 37.2B
    params: 30B MoE (3B active)
    context: 256K
    reasoning: false
    eval_score: 7/10
    verdict: GOOD — lightweight, fast, 256K context, good for quick classification
    notes: >
      Small active params (3B) means very fast. Fully open weights/datasets.
      Suitable for the high-volume path when heuristics already scored 3+.

  - id: minimax/minimax-m2.5:free
    tokens_used: 3.41B
    params: unknown
    context: 197K
    reasoning: false
    eval_score: 7/10
    verdict: GOOD — production-grade, office/workflow-optimized
    notes: >
      Strong at structured document tasks (Word/Excel generation). JSON output
      very reliable. 80.2% on SWE-Bench Verified.

  - id: arcee-ai/trinity-mini:free
    tokens_used: 16.7B
    params: 26B MoE (3B active)
    context: 131K
    reasoning: false
    eval_score: 7/10
    verdict: GOOD — efficient, fast, function calling support
    notes: Solid lightweight classifier. 128 experts / 8 active per token.

  - id: meta-llama/llama-3.3-70b-instruct:free
    tokens_used: 848M
    params: 70B dense
    context: 66K
    reasoning: false
    eval_score: 7/10
    verdict: GOOD — battle-tested classifier, multilingual (German relevant for your project)
    notes: >
      Most battle-tested model on the list for classification tasks. German language
      support is a bonus for your DACH-focused repos. Dense (not MoE) = consistent.

  - id: openai/gpt-oss-20b:free
    tokens_used: 718M
    params: 21B MoE (3.6B active)
    context: 131K
    reasoning: configurable
    eval_score: 7/10
    verdict: GOOD — fast, Apache 2.0, reliable
    notes: Low-latency, fine-tunable, strong structured output.

  - id: nvidia/nemotron-nano-12b-v2-vl:free
    tokens_used: 11.3B
    params: 12B
    context: 128K
    reasoning: false
    modal: VISION
    eval_score: 5/10
    verdict: SKIP — vision model, overkill for text-only slop detection
    notes: Included for completeness. Use only if you add screenshot-based issue detection.

  - id: nvidia/nemotron-nano-9b-v2:free
    tokens_used: 10.1B
    params: 9B
    context: 128K
    reasoning: true
    eval_score: 5/10
    verdict: MARGINAL — too small for nuanced soft-sell spam detection
    notes: Reasoning traces help but 9B may miss subtle patterns in ambiguous cases.

  - id: liquid/lfm-2.5-1.2b-thinking:free
    tokens_used: 510M
    params: 1.2B
    context: 32K
    eval_score: 2/10
    verdict: SKIP — too small and too low context for reliable classification
    notes: Designed for edge devices. False positive rate will be high.

  - id: liquid/lfm-2.5-1.2b-instruct:free
    tokens_used: 364M
    params: 1.2B
    context: 32K
    eval_score: 2/10
    verdict: SKIP — same as above
    notes: Even smaller than the thinking variant.

## CONCLUSION
# ─────────────────────────────────────────────────────────────────────────────
# DO: model: "openrouter/free"
#     response_format: { type: "json_object" }
#
# The router automatically:
#   1. Filters for models that support JSON structured output
#   2. Picks from top free models (currently Step 3.5 Flash, Nemotron Super,
#      gpt-oss-120b, Qwen3 Coder — all score 8-9/10 for this task)
#   3. Stays current as new models are added
#
# DON'T: hardcode a model list (it goes stale in weeks)
# DON'T: use :free suffix on 1-2 specific models (loses redundancy)
# DON'T: include tiny models (1-2B) in any fallback chain for this task
# ─────────────────────────────────────────────────────────────────────────────

## Rate limits (free tier)
# Free users: 50 requests/day, 20 requests/minute
# GitHub Actions context: each issue = 1 API call max → 50 issues/day covered
# For repos with >50 issues/day: upgrade to pay-as-you-go or rely on heuristics only
