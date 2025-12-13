# GOAP Plan: Fix Memory-MCP Server Initialization

## Phase 1: Task Analysis

### Primary Goal
Fix the memory-mcp server to properly implement MCP protocol initialization by including required `protocolVersion` and `serverInfo` fields in the initialization response.

### Constraints
- Time: Urgent (server currently failing to connect)
- Resources: Available agents (Explore, web-search-researcher, feature-implementer, test-runner)
- Dependencies: MCP protocol specification, existing server implementation

### Complexity Level
**Medium**: 2-3 agents, some dependencies, requires understanding MCP protocol spec and modifying existing code

### Quality Requirements
- Testing: Integration test with MCP inspector
- Standards: AGENTS.md compliance, Rust best practices
- Documentation: Update if needed
- Performance: Minimal impact on server startup

### Error Details
From debug log (lines 123-162):
```
MCP server "memory-mcp": Connection failed
- Missing "protocolVersion" (expected string, received undefined)
- Missing "serverInfo" (expected object, received undefined)
```

## Phase 2: Task Decomposition

### Main Goal
Ensure memory-mcp server returns proper MCP initialization response with all required fields.

### Sub-Goals

1. **Research MCP Protocol** - Priority: P0
   - Success Criteria: Understand exact initialization response format
   - Dependencies: None
   - Complexity: Low
   - Agent: web-search-researcher

2. **Locate Implementation** - Priority: P0
   - Success Criteria: Find MCP server initialization code
   - Dependencies: None (parallel with research)
   - Complexity: Low
   - Agent: Explore

3. **Implement Fix** - Priority: P1
   - Success Criteria: Server returns proper initialization response
   - Dependencies: Research + Locate
   - Complexity: Medium
   - Agent: feature-implementer

4. **Validate Fix** - Priority: P2
   - Success Criteria: Server connects successfully, passes MCP inspector
   - Dependencies: Implement Fix
   - Complexity: Low
   - Agent: test-runner

### Atomic Tasks

**Component 1: Research**
- Task 1.1: Search MCP protocol docs for initialization spec (Agent: web-search-researcher)
- Task 1.2: Identify required fields and their formats (Agent: web-search-researcher)

**Component 2: Locate**
- Task 2.1: Find memory-mcp server source files (Agent: Explore)
- Task 2.2: Identify initialization handler code (Agent: Explore)

**Component 3: Fix**
- Task 3.1: Add protocolVersion field to initialization response (Agent: feature-implementer)
- Task 3.2: Add serverInfo object to initialization response (Agent: feature-implementer)
- Task 3.3: Ensure response format matches MCP spec (Agent: feature-implementer)

**Component 4: Validate**
- Task 4.1: Build and run memory-mcp server (Agent: test-runner)
- Task 4.2: Test connection and verify initialization (Agent: test-runner)
- Task 4.3: Run MCP inspector for validation (Agent: test-runner)

### Dependency Graph
```
Task 1.1 (Research) → Task 1.2 → Task 3.1
                                ↘
Task 2.1 (Locate) → Task 2.2 → Task 3.2 → Task 4.1 → Task 4.2 → Task 4.3
                                ↗
                     Task 3.3 ↗
```

## Phase 3: Strategy Selection

### Strategy: **Hybrid Execution**

**Rationale:**
- Phase 1 (Research + Locate): **PARALLEL** - Independent tasks, can run simultaneously
- Phase 2 (Implement): **SEQUENTIAL** - Must wait for Phase 1 results
- Phase 3 (Validate): **SEQUENTIAL** - Must validate after implementation

**Execution Pattern:**
```
[PARALLEL]
  ├─ Research (web-search-researcher)
  └─ Explore (Explore agent)
      ↓
[SEQUENTIAL]
  → Implement (feature-implementer)
      ↓
[SEQUENTIAL]
  → Validate (test-runner)
```

## Phase 4: Agent Assignment

### Agent Capability Mapping

| Phase | Tasks | Agent Type | Rationale |
|-------|-------|------------|-----------|
| Research | 1.1, 1.2 | web-search-researcher | Search MCP docs, latest spec |
| Locate | 2.1, 2.2 | Explore | Find files, understand code structure |
| Implement | 3.1, 3.2, 3.3 | feature-implementer | Modify code, add fields |
| Validate | 4.1, 4.2, 4.3 | test-runner | Build, test, verify |

## Phase 5: Execution Plan

### Overview
- Strategy: Hybrid (Parallel → Sequential → Sequential)
- Total Tasks: 10 atomic tasks across 4 components
- Estimated Duration: 15-20 minutes
- Quality Gates: 3 checkpoints

### Phase 1: Research & Discovery (PARALLEL)
**Tasks:**
- Task Group A (web-search-researcher):
  - Search https://modelcontextprotocol.io for initialization spec
  - Document required fields: protocolVersion, serverInfo structure

- Task Group B (Explore):
  - Find memory-mcp-server source code
  - Locate initialization/handshake handler

**Quality Gate 1**:
- [ ] MCP initialization spec documented
- [ ] Server initialization code located

### Phase 2: Implementation (SEQUENTIAL)
**Tasks:**
- feature-implementer:
  - Add `protocolVersion` field (use MCP spec version)
  - Add `serverInfo` object with name and version
  - Ensure JSON-RPC response format is correct

**Quality Gate 2**:
- [ ] Code modified with required fields
- [ ] Code compiles without errors
- [ ] Follows Rust best practices

### Phase 3: Validation (SEQUENTIAL)
**Tasks:**
- test-runner:
  - Build memory-mcp-server binary
  - Test server connection (check no validation errors)
  - Verify with MCP inspector if available

**Quality Gate 3**:
- [ ] Server builds successfully
- [ ] Connection succeeds (no protocol errors)
- [ ] Initialization fields present and valid

### Overall Success Criteria
- [x] MCP spec understood
- [ ] Server code located and modified
- [ ] protocolVersion field added
- [ ] serverInfo object added
- [ ] Server connects without validation errors
- [ ] No regression in existing functionality

### Contingency Plans
- If Phase 1 research fails → Read MCP SDK source code directly
- If implementation unclear → Create minimal MCP server example first
- If tests fail → Add debug logging to see actual response

## Phase 6: Execution Status

### ✅ Phase 1 Complete: Research & Discovery (PARALLEL)

**Research Results (web-search-researcher):**
- MCP Protocol Version: `2025-06-18` (latest) or `2024-11-05` (current in code)
- Required fields in InitializeResult:
  - `protocolVersion` (string, camelCase)
  - `serverInfo` (object with name, version, optional title - camelCase)
  - `capabilities` (object describing server features)

**Code Examination Results (Explore):**
- **File:** `/workspaces/feat-phase3/memory-mcp/src/bin/server.rs`
- **Lines:** 48-54 (InitializeResult struct), 455-493 (handler)
- **Root Cause:** Struct uses snake_case (`protocol_version`, `server_info`) instead of camelCase
- **Fix Required:** Add `#[serde(rename = "...")]` attributes to struct fields

**Quality Gate 1:** ✅ PASSED
- [x] MCP initialization spec documented
- [x] Server initialization code located
- [x] Root cause identified

### ✅ Phase 2 Complete: Implementation (SEQUENTIAL)

**Changes Applied:**
- **File:** `/workspaces/feat-phase3/memory-mcp/src/bin/server.rs`
- **Lines 51, 54:** Added `#[serde(rename = "...")]` attributes
  - `protocol_version` → serializes as `"protocolVersion"`
  - `server_info` → serializes as `"serverInfo"`

**Build Result:**
- Compilation: ✅ Success (3m 14s)
- Binary: `target/release/memory-mcp-server`

**Quality Gate 2:** ✅ PASSED
- [x] Code modified with required fields
- [x] Code compiles without errors
- [x] Follows Rust best practices

### ✅ Phase 3 Complete: Validation (SEQUENTIAL)

**Test Results:**
- Test script: `test_mcp_init.sh`
- Server response validated with `jq`

**MCP Initialization Response:**
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "protocolVersion": "2024-11-05",
    "serverInfo": {
      "name": "memory-mcp-server",
      "version": "0.1.6"
    },
    "capabilities": {
      "tools": {
        "listChanged": false
      }
    }
  }
}
```

**Quality Gate 3:** ✅ PASSED
- [x] Server builds successfully
- [x] Connection succeeds (no protocol errors)
- [x] Initialization fields present and valid (camelCase)

### 🎉 Mission Complete: All Quality Gates Passed

---

## Notes
- MCP config location: `.mcp.json` or `.claude/settings.local.json`
- Server binary: `target/release/memory-mcp-server`
- Test script: `scripts/test-mcp-tools.sh`
