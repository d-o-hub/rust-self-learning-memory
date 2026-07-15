# ADR-073: Capability-Enforced Agent Code Execution

- **Status**: Proposed
- **Date**: 2026-07-14
- **Deciders**: Project maintainers and security owners
- **Related**: ADR-024, ADR-072; `memory-mcp/src/sandbox/`
- **Plan**: `plans/GOAP_CODEBASE_IMPROVEMENTS_2026-07-14.md` action S1.1

## Context

The repository describes agent code execution as a secure Wasmtime/WASM sandbox, but the current `memory-mcp` manifest has no `wasmtime-backend` feature. A legacy `CodeSandbox` implementation still starts a host Node.js process, but it is disconnected from a usable MCP tool: the handler marks execution deprecated and returns “Code execution is no longer available,” and current tool registration does not expose a working `execute_agent_code` definition.

The disconnected Node code contains regex screening, a JavaScript wrapper, global shadowing/deletion, a child-process timeout, and `kill_on_drop`. An OS isolation helper exists in `sandbox/isolation.rs`, but that Node command does not apply it. Network, filesystem, and subprocess settings select source-pattern checks rather than runtime capability enforcement, and the configured memory limit is not applied to the child process.

The current unavailable MCP behavior is appropriately fail closed. The risks are stale public claims, a misleading/dead implementation that could be accidentally reconnected, and any future attempt to restore execution without a real capability boundary. Static screening can remain defense in depth but cannot become the security boundary for adversarial code.

## Decision

### 1. Preserve fail-closed behavior and report capabilities

While no approved backend exists, `execute_agent_code` remains absent from tool discovery and direct calls are rejected. Stale README/tool metadata is corrected, and disconnected executor code is removed or explicitly quarantined as experimental trusted-development code.

A future `execute_agent_code` is not registered as a production-safe MCP tool unless startup self-tests establish an approved backend and its required controls. If introduced, tool discovery and health output report:

- backend and version;
- execution language/compiler path;
- filesystem, network, clock, randomness, and environment capabilities;
- memory/fuel/CPU/output/process limits;
- artifact/plugin identity; and
- self-test result.

If capability attestation fails, the tool is unavailable while all memory tools remain operational.

### 2. Conditional production backend

If code execution is reintroduced, run a bounded feasibility/security spike and use Wasmtime with WASI/component capabilities as the preferred production boundary. A rejected spike leaves execution unavailable and closes without exposing a replacement tool.

- No network, filesystem, subprocess, or inherited environment capability is supplied by default.
- Optional capabilities are explicit per request/policy, least-privilege, and auditable.
- Execution uses fuel and/or epoch interruption, wall-clock timeout, memory/table limits, output limits, and instance concurrency limits.
- Host functions are allowlisted and validate input/output bounds.
- JavaScript/TypeScript execution requires an explicit compiler pipeline (for example, a pinned Javy component). Compiler/plugin artifacts have version, digest, maximum-size, and provenance checks.
- A failed compile is an execution error; it does not fall back to host Node.

Exact Wasmtime/Javy versions are selected during implementation and pinned through normal dependency/artifact policy.

### 3. Legacy Node classification

The existing disconnected Node executor is removed or retained only as explicitly enabled **trusted-development executor** code. It must not be reconnected to MCP production unless maintainers implement and validate equivalent OS-level isolation on every supported platform.

- It is disabled by default in MCP production configuration.
- It is never called a sandbox or used as fallback for the production backend.
- Regex scanning and wrapper restrictions remain supplementary safeguards, not capability enforcement.
- Configuration and health prominently report `trusted-development` and the residual risk.

### 4. Runtime policy and observability

Each execution records a bounded audit event with request ID, backend identity, granted capabilities, limits, duration, termination reason, and truncated output sizes. User code and sensitive context are not logged by default.

Termination reasons are typed: compile failure, capability denial, fuel/CPU exhaustion, memory limit, wall timeout, output limit, runtime failure, and internal backend failure.

### 5. Security test contract

The test suite includes attempts using direct, aliased, computed, dynamic, encoded, and reflective access to:

- network endpoints and DNS;
- host and parent filesystem paths;
- subprocess/process APIs;
- environment variables and secrets;
- excessive memory, CPU, processes, output, and execution time; and
- host-function input/output limits.

Tests assert runtime denial/termination even when static screening is bypassed. Platform-specific implementation claims require platform CI coverage; otherwise the capability is reported unsupported.

## Consequences

### Positive

- Security depends on capabilities and resource controls rather than source spelling.
- MCP advertises only execution modes that passed startup verification.
- Compiler/runtime artifact trust becomes explicit.
- Typed failures improve operations and testing.

### Negative

- Wasmtime/Javy integration increases binary size, build time, and maintenance.
- Existing JavaScript behavior may be unavailable until a compiler component is installed and verified.
- Some host functionality requires carefully designed allowlisted interfaces.

### Neutral

- Disabling code execution does not disable episodic memory, retrieval, patterns, embeddings, or other MCP tools.
- Static analysis remains useful as an early rejection and telemetry layer.

## Alternatives considered

1. **Keep regex + Node wrapper as the production sandbox**: rejected; lexical checks do not enforce runtime capabilities or memory limits.
2. **Apply only the existing shell/ulimit helper**: rejected as the cross-platform primary design; it improves resource control but does not provide a complete capability model and introduces shell complexity.
3. **Run Node in containers**: viable for deployments with a trusted container runtime, but too environment-dependent as the library/MCP default and still requires capability/resource policy.
4. **Remove code execution permanently**: safest and acceptable fallback, but rejects a useful conditional feature before evaluating a capability-safe backend.
5. **Automatically fall back from Wasmtime to Node**: rejected because fallback would silently weaken the security boundary.

## Migration

1. Preserve and test the current unavailable MCP behavior; correct docs/tool metadata.
2. Remove the disconnected Node executor or quarantine it behind explicit trusted-development configuration and source-reachability checks.
3. Run a bounded Wasmtime/WASI feasibility and threat-model spike.
4. If rejected, record the decision and keep execution unavailable. If approved, build the backend behind a new accurately named feature.
5. Add pinned compiler component handling, startup self-tests, and runtime bypass/resource tests before registration.
6. Promote a backend only after platform/security validation and update generated tool docs.

## Acceptance criteria

**No-backend branch:**

- Tool discovery omits `execute_agent_code`, direct calls fail closed, and public docs state that execution is unavailable.
- No production-safe path reaches the legacy Node executor.

**Approved-backend branch:**

- The selected production backend denies ungranted network/filesystem/process access at runtime.
- Memory, CPU/fuel, timeout, output, and concurrency limits have adversarial tests.
- Compiler/runtime artifacts are pinned and digest-verified.
- There is no silent Node fallback.
- README, feature flags, health, tool discovery, and runtime behavior agree.
- An independent security review signs off before production enablement.
