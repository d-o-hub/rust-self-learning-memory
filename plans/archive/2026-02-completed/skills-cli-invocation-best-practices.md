# CLI Tool Invocation Best Practices for SKILL.md Files (2026)

**Last Updated**: 2026-02-13  
**Research Coverage**: Claude Code, Cursor, VS Code, OpenCode, MCP Protocol  
**Focus**: 2026 current best practices across AI coding environments

---

## Executive Summary

**Key Finding (2026)**: The AI coding landscape has converged around **three primary patterns** for CLI tool invocation from SKILL.md files:

1. **Direct CLI Bash Blocks** (OpenCode pattern) - Most explicit and portable
2. **Structured Tool Definitions** (Claude Code pattern) - For API-driven environments  
3. **MCP Protocol Integration** - Emerging standard for cross-environment compatibility

**Universal Pattern Across All Environments**: Use **YAML frontmatter** for metadata + **markdown code blocks** for CLI commands.

---

## Quick Reference Cheat Sheet

### Pattern Comparison Table

| Environment | Syntax Pattern | Progressive Disclosure | Error Handling | Best For |
|-------------|----------------|----------------------|-----------------|-----------|
| **OpenCode** | `bash` code blocks in markdown | Sub-file references (`[Subdoc](file.md)`) | Bash error messaging | High-frequency CLI ops |
| **Claude Code** | Tool definitions in API | Lazy loading via tool choice | API error responses | Cloud/API workflows |
| **Cursor** | Notepad/Cursor rules files | Section headers + collapsible | IDE error display | VS Code integration |
| **VS Code** | GitHub Copilot CLI | Profile-based commands | Extension errors | Editor workflows |
| **MCP** | `input_schema` JSON Schema | Tool registry | MCP error protocol | Cross-platform tools |

### 2026 Universal Best Practices

```yaml
---
# Universal YAML Frontmatter Pattern (works across all environments)
name: your-skill-name
description: One-line description for when to use this
triggers:
  keywords: ["keyword1", "keyword2"]
  patterns: [".*pattern.*"]
  files: ["**/specific-files"]
priority: high
---
```

```markdown
# Universal CLI Command Pattern
```bash
# Comment explaining what this does
your_command --with --flags
```

## When to use
- **Situation 1**: When X is true
- **Situation 2**: When Y is needed
```

---

## Environment-Specific Deep Dives

### 1. OpenCode (opencode.ai)

**Current Implementation Status**: ✅ **Fully Supported** (researched 2026-02-13)

#### Invocation Syntax

OpenCode uses **bash code blocks** in SKILL.md files for CLI invocation:

```markdown
---
name: build-rust
description: Optimized Rust build operations
---

# Build Operations

## Usage

```bash
# Development build
./scripts/build-rust.sh dev

# Release build
./scripts/build-rust.sh release
```
```

**Key Pattern**: CLI commands are **directly embedded** in markdown code blocks with the `bash` language tag.

#### Progressive Disclosure

OpenCode implements progressive disclosure through **sub-file references**:

```markdown
## Quick Reference

- **[Strategies](strategies.md)** - Query-type-specific search approaches
- **[Progressive Search](progressive.md)** - Time-boxed research rounds
- **[Synthesis Format](synthesis.md)** - Organizing findings
```

**Implementation**: These are markdown links to separate files in the same skill directory:
```
.opencode/skill/web-search-researcher/
├── SKILL.md                    # Main file (loads sub-files)
├── strategies.md                # Loaded on-demand
├── progressive.md               # Loaded on-demand
└── synthesis.md                 # Loaded on-demand
```

#### Tool Descriptions

OpenCode uses the `description` field in YAML frontmatter:

```yaml
---
name: web-search-researcher
description: Research topics using web search and content fetching to find accurate, current information. Use when you need modern information, official documentation, best practices, technical solutions, or comparisons beyond your training data.
---
```

**Best Practice**: 1-2 sentences covering:
1. **What** the skill does
2. **When** to use it
3. **What** it's for (modern info, docs, best practices)

#### Error Handling Patterns

```markdown
## Troubleshooting

### Connection Issues
**Symptom**: Connection refused
```bash
# Check environment variables
echo $TURSO_URL
echo $TURSO_TOKEN

# Use test database
export DATABASE_URL="file:./test.db"
```

### Timeout Errors
**Symptom**: Build timeout after 120s
- Use `dev` mode for faster iteration
- Reduce parallel jobs: `CARGO_BUILD_JOBS=4 cargo build`
```

**Pattern**: 
1. Identify symptoms
2. Provide diagnostic commands
3. Offer solutions with specific commands

#### Skill Triggers

OpenCode uses `skill-rules.json` for automatic skill loading:

```json
{
  "rules": [
    {
      "skill": "rust-async-testing",
      "triggers": {
        "keywords": ["async", "tokio", "test", "await"],
        "patterns": ["test_.*async", ".*_async_test"],
        "files": ["**/tests/**/*.rs"]
      },
      "priority": "high"
    }
  ]
}
```

**How It Works**:
1. **Keyword Matching**: Scans user messages for keywords
2. **Pattern Matching**: Applies regex patterns to file paths
3. **File Detection**: Checks workspace for matching files
4. **Priority System**: High/medium/low determines which skill to load

#### OpenCode-Specific Features

**On-Demand CLI Pattern** (High-frequency operations):
```markdown
## Usage

```bash
# Direct CLI invocation for high-frequency operations
cargo build --release --workspace

# Alternative: Use dedicated script for complex workflows
./scripts/build-rust.sh release
```

**Best Practice**: 
- **Simple commands**: Embed directly in skill
- **Complex workflows**: Reference bash scripts
- **Performance**: CLI pattern minimizes token overhead

---

### 2. Claude Code (claude.ai)

**Current Implementation Status**: ✅ **Fully Documented** (API-based, researched 2026-02-13)

#### Invocation Syntax

Claude Code uses the **Messages API** with structured tool definitions:

```python
tools = [
    {
        "name": "get_weather",
        "description": "Get the current weather in a given location",
        "input_schema": {
            "type": "object",
            "properties": {
                "location": {
                    "type": "string",
                    "description": "The city and state, e.g. San Francisco, CA"
                },
                "unit": {
                    "type": "string",
                    "enum": ["celsius", "fahrenheit"],
                    "description": "The temperature unit"
                }
            },
            "required": ["location"]
        }
    }
]
```

**Response Format**:
```json
{
  "type": "tool_use",
  "id": "toolu_01A09q90qw90lq917835lq9",
  "name": "get_weather",
  "input": {"location": "San Francisco, CA", "unit": "celsius"}
}
```

**Key Pattern**: **Structured tool definitions** with JSON Schema for validation.

#### Progressive Disclosure

Claude Code implements progressive disclosure through:

1. **Tool Choice Parameter**: `tool_choice: "auto"` vs `"any"` vs `["tool_name"]`
2. **System Prompt Token Management**: Tools add 313-346 tokens per tool
3. **Client vs Server Tools**: Server tools don't require implementation

**Token Optimization Strategy**:
```python
# Minimal tool definition (313 tokens)
{
  "name": "simple_tool",
  "description": "Brief description",
  "input_schema": {"type": "object", "properties": {}}
}

# Complex tool (346+ tokens)
{
  "name": "complex_tool", 
  "description": "Detailed description with examples...",
  "input_schema": {/* complex schema */}
}
```

**Best Practice**: Use `tool_choice: "any"` to force tool use, `"auto"` for automatic decision.

#### Tool Descriptions

Claude's best practices for tool descriptions (from official docs):

```python
{
  "description": """Get the current weather in a given location.
  
  Provides temperature, conditions, and forecast for any city.
  Requires location and optionally temperature unit preference.
  """
}
```

**Effective Description Pattern**:
1. **First sentence**: What the tool does (verb-first)
2. **Second sentence**: What it provides/returns
3. **Third sentence**: Requirements and constraints
4. **Length**: 1-3 sentences for best performance

**Anti-Pattern** (from docs):
```python
# Too verbose - causes higher token usage
{
  "description": """
  This tool is designed to retrieve weather information by making
  API calls to weather services and parsing the response data...
  """  # Don't write essays!
}
```

#### Error Handling

Claude Code implements error handling through the **tool result loop**:

```python
# User sends tool result
{
  "role": "user",
  "content": [
    {
      "type": "tool_result",
      "tool_use_id": "toolu_01A09q90qw90lq917835lq9",
      "content": "Error: Connection timeout",  # Error message
      "is_error": True  # Flag as error (optional)
    }
  ]
}
```

**Claude's Response**:
- Claude analyzes the error message
- May retry with different parameters
- May report error to user with explanation
- May suggest alternative approaches

**Best Practice**: Always include `is_error: True` for failures to help Claude understand.

#### Parallel Tool Invocation (2026 Feature)

Claude can call multiple tools in parallel:

```json
{
  "content": [
    {
      "type": "tool_use",
      "id": "toolu_01A1",
      "name": "get_weather",
      "input": {"location": "San Francisco, CA"}
    },
    {
      "type": "tool_use", 
      "id": "toolu_01A2",
      "name": "get_time",
      "input": {"timezone": "America/Los_Angeles"}
    }
  ]
}
```

**Response Requirement**: ALL tool results must be returned in a SINGLE user message:

```python
{
  "content": [
    {
      "type": "tool_result",
      "tool_use_id": "toolu_01A1",
      "content": "65°F"
    },
    {
      "type": "tool_result",
      "tool_use_id": "toolu_01A2", 
      "content": "3:47 PM"
    }
  ]
}
```

**Critical**: Don't send separate messages for each result - Claude expects parallel tools to have parallel results.

#### Claude-Specific Features

**Strict Tool Use** (2025 feature):
```python
tools = [{
  "name": "structured_tool",
  "strict": True,  # Add for guaranteed schema conformance
  "input_schema": {...}
}]
```

**Benefits**:
- Guaranteed schema validation
- No type mismatches
- Perfect for production agents
- Prevents invalid tool parameter failures

**Server-Side Tools** (2025-2026):
- `web_search_20250305`: Web search without implementation
- `web_fetch_20250305`: URL fetching without implementation
- `computer_use_20241022`: Desktop automation
- `text_editor_20250124`: File editing automation

**Advantage**: No client implementation needed, just specify in API request.

---

### 3. Cursor (cursor.sh)

**Current Implementation Status**: ⚠️ **Limited Documentation** (certificates expired, inferred patterns)

#### Invocation Syntax

Based on available research, Cursor uses **Notepad files** and **.cursor/rules** for customization:

```markdown
## .cursor/rules

When I ask for help with Rust:
- Prefer async/await patterns
- Use Result<T, E> for errors
- Suggest tokio for async runtime
```

**Pattern**: **Natural language rules** rather than structured tool definitions.

#### Progressive Disclosure

Cursor implements disclosure through:
- **Section headers**: Collapsible sections in rules files
- **Multiple rule files**: Separate files for different contexts
- **Context awareness**: IDE detects file type and loads relevant rules

**Example Structure**:
```
.cursor/
├── rules/
│   ├── rust.md          # Rust-specific rules
│   ├── typescript.md     # TypeScript-specific rules
│   └── general.md       # Universal rules
└── rules                # Legacy single file
```

#### Error Handling

Cursor handles errors through:
- **IDE error display**: Standard VS Code error panels
- **Inline suggestions**: Red squigglies with fixes
- **Chat context**: Errors appear in AI chat panel

**Limited Research**: Certificate issues prevented full documentation access (2026-02-13).

---

### 4. VS Code / GitHub Copilot

**Current Implementation Status**: ✅ **Documented** (researched 2026-02-13)

#### Invocation Syntax

GitHub Copilot supports CLI through **GitHub CLI (`gh`)** integration:

```bash
# Copilot in terminal
gh copilot suggest "Create a function to sort an array"
gh copilot explain "How does this regex work?"

# Copilot code review
gh copilot code-review --pr-number 123
```

**Pattern**: **Natural language CLI commands** processed by Copilot.

#### Progressive Disclosure

Copilot implements disclosure through:
- **Profile-based commands**: Different commands for different contexts
- **Context awareness**: Detects repo type and adjusts suggestions
- **Model selection**: Haiku (fast) vs Sonnet (balanced) vs Opus (quality)

**Example**:
```bash
# Fast suggestions (Haiku 4.5)
gh copilot suggest --model haiku "Quick completion"

# Quality suggestions (Opus 4.6)  
gh copilot suggest --model opus "Complex algorithm"
```

#### Error Handling

Copilot CLI errors follow standard patterns:
```bash
Error: Authentication required
> Please run: gh auth login

Error: Rate limit exceeded
> Upgrade to Copilot Pro for higher limits

Error: Context too large
> Reduce selection or split into smaller requests
```

#### Copilot-Specific Features

**Agent Mode** (2025 feature):
- Assign issues to AI agents
- Agents create pull requests
- Agents respond to feedback
- Multiple agents: Claude, Codex, custom

**Spaces** (2025 feature):
- Shared knowledge base
- Organizes docs and repos
- Project-specific context
- Team consistency

---

### 5. MCP (Model Context Protocol)

**Current Implementation Status**: ✅ **Emerging Standard** (hosted by Linux Foundation, researched 2026-02-13)

#### What is MCP?

**MCP (Model Context Protocol)** is an open standard for connecting AI applications to external systems - like **USB-C for AI**.

**Key Insight**: MCP provides **standardized tool integration** across environments:
- Claude Code: Native MCP support
- OpenCode: MCP server support
- VS Code: Via GitHub Copilot
- Cursor: Via MCP-compatible servers

#### Invocation Syntax

MCP uses **JSON Schema** for tool definitions (similar to Claude Code):

```typescript
// MCP Server Tool Definition
{
  name: "get_weather",
  description: "Get current weather for a location",
  inputSchema: {
    type: "object",
    properties: {
      location: {
        type: "string",
        description: "City and state, e.g. San Francisco, CA"
      },
      unit: {
        type: "string",
        enum: ["celsius", "fahrenheit"]
      }
    },
    required: ["location"]
  }
}
```

**Key Difference**: `inputSchema` (camelCase) vs `input_schema` (snake_case) for Claude.

#### Converting MCP to Claude Format

```python
# Convert MCP tools to Claude format
async def get_claude_tools(mcp_session):
    mcp_tools = await mcp_session.list_tools()
    
    claude_tools = []
    for tool in mcp_tools.tools:
        claude_tools.append({
            "name": tool.name,
            "description": tool.description or "",
            "input_schema": tool.inputSchema  # Rename inputSchema
        })
    
    return claude_tools
```

**Why MCP Matters (2026)**:

1. **Developers**: Reduces dev time and complexity
2. **AI Applications**: Access to ecosystem of data sources and tools
3. **End Users**: More capable AI with personal data access

**Real Examples**:
- **Google Calendar integration**: Agents can access your calendar
- **Notion connection**: AI reads your notes for context
- **Figma integration**: Claude generates web apps from designs
- **Database queries**: Enterprise chatbots analyze multiple databases

#### Progressive Disclosure

MCP implements disclosure through:
- **Tool Registry**: Servers register available tools
- **Dynamic Discovery**: Clients query `list_tools()` on demand
- **Resource Templates**: Predefined prompts for common tasks
- **Prompt Templates**: Reusable prompt patterns

#### MCP Server Implementation Pattern

```typescript
// MCP Server exposing CLI tools
import { Server } from '@modelcontextprotocol/sdk/server/index.js';

const server = new Server(
  {
    name: "my-cli-tools",
    version: "1.0.0"
  },
  {
    capabilities: {
      tools: {}
    }
  }
);

// Register CLI tool
server.setRequestHandler(ListToolsRequestSchema, async () => ({
  tools: [{
    name: "run_build",
    description: "Execute build commands",
    inputSchema: {
      type: "object",
      properties: {
        target: {
          type: "string",
          description: "Build target"
        }
      }
    }
  }]
}));

// Execute CLI tool
server.setRequestHandler(CallToolRequestSchema, async (request) => {
  const { name, arguments: { target } } = request.params;
  
  if (name === "run_build") {
    const result = execSync(`cargo build --${target}`);
    return {
      content: [{ type: "text", text: result }]
    };
  }
});
```

**Benefits**:
- **Standardized**: Same protocol works across Claude, OpenCode, etc.
- **Language Agnostic**: SDKs for TypeScript, Python, Rust, Go, etc.
- **Extensible**: Easy to add new tools and resources
- **Discoverable**: Clients can query available capabilities

#### MCP Ecosystem (2026)

**Official SDKs**:
- TypeScript SDK: https://github.com/modelcontextprotocol/typescript-sdk
- Python SDK: https://github.com/modelcontextprotocol/python-sdk
- Rust SDK: https://github.com/modelcontextprotocol/rust-sdk
- Go SDK: https://github.com/modelcontextprotocol/go-sdk
- + Java, Kotlin, C#, PHP, Ruby, Swift

**Notable Servers**:
- `@modelcontextprotocol/server-github`: GitHub integration
- `@modelcontextprotocol/server-filesystem`: File system access
- `@modelcontextprotocol/server-postgres`: PostgreSQL database
- `@modelcontextprotocol/server-sqlite`: SQLite database
- Custom servers: 78.6k stars on official servers repo

---

## Universal Best Practices (Cross-Environment)

### 1. YAML Frontmatter (Universal)

**All environments support YAML frontmatter** for skill metadata:

```yaml
---
name: your-skill-name
description: Clear, concise description of when to use this
version: "1.0.0"
author: "Your Name or Team"
tags: [rust, testing, async]
triggers:
  keywords: ["keyword1", "keyword2"]
  patterns: [".*pattern.*"]
  files: ["**/matching-files"]
priority: high  # high | medium | low
depends_on: [other-skill-name]
---
```

**Field Guide**:
- `name`: kebab-case identifier
- `description`: 1 sentence, action-oriented ("Do X when Y")
- `version`: Semantic versioning
- `priority`: Determines which skill to load when multiple match
- `depends_on`: Prevents circular dependencies

### 2. CLI Command Patterns

**For High-Frequency Operations** (direct bash in markdown):

```markdown
## Quick Commands

```bash
# Single-line commands (simple, safe)
cargo test --lib
cargo build --release
cargo clippy --all -- -D warnings
```

**Best For**: Development workflow commands
```

**For Complex Workflows** (reference scripts):

```markdown
## Operations

```bash
# Use dedicated scripts for complex/multi-step operations
./scripts/build-rust.sh release
./scripts/deploy-production.sh
./scripts/run-benchmarks.sh
```

**Best For**: Production deployments, multi-step workflows
```

**For Dynamic Operations** (bash with variables):

```bash
# Allow parameterization
cargo build --$BUILD_PROFILE
cargo test --test-threads=$THREAD_COUNT
./scripts/deploy.sh --env=$ENVIRONMENT
```

**Best For**: Environment-specific operations, configurable workflows
```

### 3. Tool Description Best Practices

**Universal Pattern** (works across Claude, OpenCode, MCP):

```yaml
---
description: >
  Verb-first description of what the tool does.
  Provides specific output/details about results.
  Use when X condition or Y situation is true.
---
```

**Examples**:

❌ **Too vague**:
```yaml
description: "A tool for building things"
```

✅ **Specific and actionable**:
```yaml
description: "Build Rust workspace with optimizations. Use when preparing production releases or running performance benchmarks."
```

❌ **Too verbose**:
```yaml
description: "This tool is designed to execute the Rust build process using the cargo build system which will compile all the crates in the workspace and link them together into binary executables..."
```

✅ **Concise and complete**:
```yaml
description: "Execute cargo build for Rust workspace. Supports debug, release, and profile modes with configurable optimization levels."
```

### 4. Error Handling Patterns

**Universal Error Handling Template**:

```markdown
## Troubleshooting

### Error Name/Symptom
**Symptom**: What the user sees or experiences

**Diagnosis**:
```bash
# Diagnostic command to verify issue
diagnostic_command_here
```

**Solution**:
```bash
# Fix command
fix_command_here

# Alternative fix
alternative_fix_command_here
```

**Prevention**:
- How to avoid this error
- Best practices to prevent recurrence
```

**Example**:

```markdown
### Out of Memory During Build
**Symptom**: Build process killed with "out of memory" error

**Diagnosis**:
```bash
# Check available memory
free -h

# Check current parallel job count
echo $CARGO_BUILD_JOBS
```

**Solution**:
```bash
# Reduce parallel jobs to 4
CARGO_BUILD_JOBS=4 cargo build --release

# Or use check instead of full build
cargo check --all
```

**Prevention**:
- Use `check` for faster iteration during development
- Reserve `build --release` for final testing only
- Close memory-intensive applications (browsers, IDEs) during release builds
```

### 5. Progressive Disclosure Implementation

**Universal Progressive Disclosure Pattern**:

```markdown
## Quick Reference

- **[Concept Guide](concept.md)** - Learn the fundamentals
- **[Tutorial](tutorial.md)** - Step-by-step walkthrough
- **[API Reference](api.md)** - Complete API documentation
- **[Examples](examples/)** - Real-world code samples

## When to Use

Use this skill when:
- **Scenario 1**: Description of when to use
- **Scenario 2**: Another scenario
- **Scenario 3**: Third scenario

## Overview

High-level overview of the skill...
(Keep this brief, 3-5 sentences)

## Detailed Documentation

Full documentation...
(Deep dive, can be extensive)
```

**Implementation**:
1. **Quick Reference Section**: Links to sub-files
2. **When to Use Section**: Trigger scenarios
3. **Brief Overview**: Essential context
4. **Detailed Docs**: The rest can be extensive

**Directory Structure**:
```
.opencode/skill/my-skill/
├── SKILL.md              # Main file (overview + quick reference)
├── concept.md            # Loaded on-demand via link
├── tutorial.md           # Loaded on-demand via link
├── api.md                # Loaded on-demand via link
└── examples/
    ├── basic.md           # Loaded on-demand via link
    └── advanced.md         # Loaded on-demand via link
```

**Benefits**:
- **Token Optimization**: Only load what's needed
- **Faster Loading**: Smaller initial files
- **Better Navigation**: Clear organization
- **Maintainability**: Easier to update specific sections

### 6. Security Considerations

**Universal Security Best Practices**:

#### Sensitive Data Handling

❌ **NEVER DO THIS**:
```bash
# Don't hardcode secrets!
curl -H "Authorization: Bearer sk_live_abc123..." https://api.example.com
mysql -u admin -ppassword123 database
```

✅ **ALWAYS DO THIS**:
```bash
# Use environment variables
curl -H "Authorization: Bearer $API_KEY" https://api.example.com
mysql -u $DB_USER -p$DB_PASSWORD $DB_NAME

# Document required env vars
# Required: API_KEY, DB_USER, DB_PASSWORD
```

#### Input Validation

**Before executing CLI commands**:
1. **Validate file paths**: Check for directory traversal (`../`)
2. **Sanitize input**: Remove or escape special characters
3. **Whitelist allowed values**: Reject unexpected inputs
4. **Limit command length**: Prevent buffer overflow attacks

**Example**:
```bash
# Validate file path before use
if [[ ! "$FILE_PATH" =~ ^[a-zA-Z0-9_/-]+$ ]]; then
  echo "Error: Invalid file path"
  exit 1
fi

# Use arrays to prevent word splitting
files=("file1.rs" "file2.rs")
cargo build -- "${files[@]}"
```

#### Injection Prevention

**Command Injection Risks**:
```bash
# DANGEROUS - User input directly in command
eval "cargo build $USER_INPUT"  # NEVER use eval!
cargo build $USER_INPUT           # Unsafe without quotes

# SAFE - Proper quoting and validation
cargo build -- "$VALIDATED_INPUT"
```

**SQL Injection** (when CLI tools interact with databases):
```bash
# DANGEROUS - Direct string interpolation
mysql -e "SELECT * FROM users WHERE id = $USER_ID"

# SAFE - Use parameterized queries
mysql --execute="SELECT * FROM users WHERE id = ?" -- "$USER_ID"
```

#### Principle of Least Privilege

```bash
# Run commands with minimum required permissions
# Instead of: sudo cargo build
# Use: cargo build (only requires write to ./target)

# Instead of: chmod 777 directory
# Use: chmod 755 directory (only owner needs write)

# Instead of running as root
# Use: Specific user with limited permissions
```

### 7. Testing CLI Invocations

**Universal Testing Pattern**:

```markdown
## Testing

Verify the tool works correctly:

```bash
# Test basic functionality
./scripts/build-rust.sh test
# Expected: Builds successfully, tests pass

# Test with specific target
./scripts/build-rust.sh dev memory-core
# Expected: Builds only memory-core crate

# Test error handling
./scripts/build-rust.sh invalid-target
# Expected: Error message with helpful guidance
```

## Validation Checklist

- [ ] Command executes without errors
- [ ] Output format is as documented
- [ ] Error messages are helpful
- [ ] Security best practices followed
- [ ] Performance is acceptable
```

---

## Code Examples by Environment

### OpenCode Example (Real from Project)

**File**: `.opencode/skill/test-runner/SKILL.md`

```markdown
---
name: test-runner
description: Execute and manage Rust tests including unit tests, integration tests, and doc tests. Use when running tests to ensure code quality and correctness.
---

# Test Runner

Execute and manage Rust tests for the self-learning memory project.

## Test Categories

| Category | Command | Scope |
|----------|---------|-------|
| Unit | `cargo test --lib` | Individual functions |
| Integration | `cargo test --test '*'` | End-to-end workflows |
| Doc | `cargo test --doc` | Documentation examples |
| All | `cargo test --all` | Complete validation |

## Execution Strategy

### Step 1: Quick Check (Unit Tests)
```bash
cargo test --lib
```
- Fast feedback (< 30s)
- Catch basic logic errors

### Step 2: Integration Tests
```bash
cargo test --test '*'
```
- Tests database interactions
- Requires Turso/redb setup

### Step 3: Full Suite
```bash
cargo test --all
```
- Complete validation before commit

## Troubleshooting

### Async/Await Issues
**Symptom**: Test hangs
```rust
#[tokio::test]
async fn test_async() {
    let result = async_fn().await;  // Don't forget .await
}
```

### Database Connection
**Symptom**: Connection refused
- Check TURSO_URL, TURSO_TOKEN
- Use test database

### Race Conditions
**Symptom**: Intermittent failures
```bash
cargo test -- --test-threads=1
```

### redb Lock Errors
**Symptom**: "Database is locked"
- Use separate DB per test
- Close transactions promptly

## Coverage

```bash
cargo install cargo-llvm-cov
cargo llvm-cov --html --output-dir coverage
```

## Best Practices

- Isolation: Each test independent
- Cleanup: Remove test data
- Speed: < 1s per unit test
- Naming: Describe behavior
```

**Key Takeaways**:
- ✅ **YAML frontmatter** with name + description
- ✅ **Table format** for comparing commands
- ✅ **Step-by-step sections** for workflows
- ✅ **Code blocks** for each command
- ✅ **Troubleshooting section** with symptoms and solutions
- ✅ **Best practices** list at end

### Claude Code Example (API-Based)

**From**: Claude official documentation

```python
import anthropic

client = anthropic.Anthropic()

response = client.messages.create(
    model="claude-opus-4-6",
    max_tokens=1024,
    tools=[
        {
            "name": "get_weather",
            "description": "Get the current weather in a given location",
            "input_schema": {
                "type": "object",
                "properties": {
                    "location": {
                        "type": "string",
                        "description": "The city and state, e.g. San Francisco, CA",
                    },
                    "unit": {
                        "type": "string",
                        "enum": ["celsius", "fahrenheit"],
                        "description": 'The unit of temperature, either "celsius" or "fahrenheit"',
                    },
                },
                "required": ["location"],
            },
        }
    ],
    messages=[
        {"role": "user", "content": "What's the weather like in San Francisco?"}
    ],
)

print(response)
```

**Key Takeaways**:
- ✅ **Structured tool definitions** with JSON Schema
- ✅ **Enum constraints** for valid inputs
- ✅ **Required fields** specified
- ✅ **Descriptions** for each property
- ✅ **Python SDK** (also TypeScript, Java, etc.)

### MCP Example (Cross-Environment)

**File**: TypeScript MCP Server

```typescript
import { Server } from '@modelcontextprotocol/sdk/server/index.js';
import { StdioServerTransport } from '@modelcontextprotocol/sdk/server/stdio.js';
import { ListToolsRequestSchema, CallToolRequestSchema } from '@modelcontextprotocol/sdk/types.js';

const server = new Server(
  {
    name: "rust-build-server",
    version: "1.0.0"
  },
  {
    capabilities: {
      tools: {}
    }
  }
);

// Register tools
server.setRequestHandler(ListToolsRequestSchema, async () => ({
  tools: [
    {
      name: "build_release",
      description: "Build Rust workspace in release mode with optimizations. Use when preparing production builds or running performance benchmarks.",
      inputSchema: {
        type: "object",
        properties: {
          target: {
            type: "string",
            description: "Specific crate to build (optional, builds entire workspace if omitted)"
          }
        }
      }
    },
    {
      name: "run_tests",
      description: "Execute Rust test suite. Supports unit, integration, and doc tests.",
      inputSchema: {
        type: "object",
        properties: {
          test_type: {
            type: "string",
            enum: ["unit", "integration", "doc", "all"],
            description: "Type of tests to run"
          }
        },
        required: ["test_type"]
      }
    }
  ]
}));

// Execute tools
server.setRequestHandler(CallToolRequestSchema, async (request) => {
  const { name, arguments } = request.params;
  
  try {
    if (name === "build_release") {
      const target = arguments.target ? `--package ${arguments.target}` : "";
      const result = execSync(`cargo build --release ${target}`, { encoding: 'utf-8' });
      
      return {
        content: [{ 
          type: "text", 
          text: `Build completed successfully:\n${result}` 
        }]
      };
    }
    
    if (name === "run_tests") {
      const command = {
        unit: "cargo test --lib",
        integration: "cargo test --test '*'",
        doc: "cargo test --doc",
        all: "cargo test --all"
      }[arguments.test_type];
      
      const result = execSync(command, { encoding: 'utf-8' });
      
      return {
        content: [{ 
          type: "text", 
          text: `Test results:\n${result}` 
        }]
      };
    }
    
  } catch (error) {
    return {
      content: [{ 
        type: "text", 
        text: `Error: ${error.message}`,
        isError: true
      }],
      isError: true
    };
  }
});

// Start server
const transport = new StdioServerTransport();
await server.connect(transport);
```

**Key Takeaways**:
- ✅ **TypeScript SDK** for server implementation
- ✅ **ListToolsRequestSchema** for tool registration
- ✅ **CallToolRequestSchema** for tool execution
- ✅ **Error handling** with try-catch
- ✅ **Stdio transport** for communication
- ✅ **Works with Claude, OpenCode, and MCP-compatible clients**

---

## Anti-Patterns to Avoid

### 1. The "Monolith" Anti-Pattern

❌ **AVOID**: Giant SKILL.md files with everything inline

```markdown
# DON'T DO THIS - 10,000 line file with everything inline
---
name: everything-skill
description: Does everything
---

## All Documentation
[10,000 lines of documentation...]

## All Commands
[hundreds of commands...]

## All Troubleshooting
[thousands of lines...]
```

**Problems**:
- Slow to load
- Wastes tokens
- Hard to navigate
- Difficult to maintain

✅ **INSTEAD**: Use progressive disclosure

```markdown
# DO THIS - Main file with links
---
name: everything-skill
description: Does everything
---

## Quick Reference

- **[Concepts](concepts.md)** - Learn fundamentals
- **[Commands](commands.md)** - Complete command reference
- **[Troubleshooting](troubleshooting.md)** - Common issues and fixes

## Overview

[Brief 3-5 sentence overview...]
```

### 2. The "Mystery Command" Anti-Pattern

❌ **AVOID**: Commands without explanation

```markdown
## Commands

```bash
cargo build --release --workspace -Z unstable-options
```

❌ What does this do? Why use these flags? What's the output?
```

**Problems**:
- Users don't understand what commands do
- Fear of breaking things
- Can't troubleshoot when things go wrong

✅ **INSTEAD**: Document every command

```markdown
## Build for Production

```bash
# Build optimized release binaries for all workspace crates
# Uses LTO (Link-Time Optimization) for maximum performance
# Output: ./target/release/ directory with binaries
cargo build --release --workspace
```

**Expected Output**:
```
   Compiling memory-core v0.1.14
   Compiling memory-storage-turso v0.1.14
   Finished release [optimized] target(s) in 2m 15s
```

**Use When**:
- Preparing production deployment
- Running performance benchmarks
- Testing with optimizations enabled
```

### 3. The "Hardcode Secrets" Anti-Pattern

❌ **CRITICAL SECURITY ISSUE**: Never hardcode credentials

```markdown
## Database Setup

```bash
# DON'T DO THIS - Exposes credentials in version control!
mysql -u admin -ppassword123 -e "CREATE DATABASE memory_system"
```
```

**Problems**:
- Credentials visible in git history
- Anyone with repo access has secrets
- Can't rotate credentials easily
- Security vulnerability

✅ **INSTEAD**: Use environment variables

```markdown
## Database Setup

```bash
# DO THIS - Use environment variables
# Required: DB_USER, DB_PASSWORD, DB_NAME
mysql -u "$DB_USER" -p"$DB_PASSWORD" -e "CREATE DATABASE $DB_NAME"
```

## Setup Instructions

1. Copy environment template:
   ```bash
   cp .env.example .env
   ```

2. Edit `.env` with your credentials:
   ```bash
   DB_USER=your_db_user
   DB_PASSWORD=your_secure_password
   DB_NAME=memory_system
   ```

3. Source environment file:
   ```bash
   source .env
   ```

4. Run setup script:
   ```bash
   ./scripts/setup-database.sh
   ```
```

### 4. The "Ambiguous Description" Anti-Pattern

❌ **AVOID**: Unclear skill descriptions

```yaml
---
# DON'T DO THIS - What does this actually do?
description: "A tool for working with Rust code"
---
```

**Problems**:
- Can't tell when to use it
- Conflicts with similar tools
- Wasted time loading irrelevant skills

✅ **INSTEAD**: Specific, actionable descriptions

```yaml
---
# DO THIS - Clear and actionable
description: "Build Rust workspace with optimizations. Use when preparing production releases or running performance benchmarks."
---
```

### 5. The "Missing Context" Anti-Pattern

❌ **AVOID**: Commands without context or prerequisites

```markdown
## Run Database Migration

```bash
# What does this need to work? What's the result?
diesel migration run
```
```

**Problems**:
- Users don't know prerequisites
- Commands fail without explanation
- Can't troubleshoot effectively

✅ **INSTEAD**: Full context and prerequisites

```markdown
## Run Database Migrations

**Prerequisites**:
- Diesel CLI installed: `cargo install diesel_cli`
- Database connection configured: `DATABASE_URL` set
- Pending migrations exist: `diesel migration pending` shows files

**What This Does**:
Applies pending database migrations to bring schema to latest version. Creates/updates tables based on migrations in `migrations/` directory.

**Expected Output**:
```
Running migration 2024-02-13-000000_create_episodes_table
Running migration 2024-02-13-000001_create_patterns_table
Migrations complete!
```

```bash
# Apply all pending migrations
diesel migration run

# Verify migrations applied
diesel migration list
```

**Troubleshooting**:
- **Error**: "Migration directory not found"
  - **Solution**: Run from project root where `migrations/` exists
- **Error**: "Database connection failed"  
  - **Solution**: Verify `DATABASE_URL` is set correctly
```

---

## 2026 Updates and Emerging Trends

### New Features (2025-2026)

#### Claude Code

1. **Strict Tool Use** (2025 Q4)
   - Guaranteed schema validation with `strict: true`
   - Prevents type mismatches and invalid parameters
   - **Best for**: Production agents requiring reliability

2. **Web Search Tool** (`web_search_20250305`)
   - Server-side search without implementation
   - Updated monthly for 2026 relevance
   - **Best for**: Current information beyond training data

3. **Parallel Tool Calls** (Enhanced 2025)
   - Multiple tools in single response
   - All results in single user message
   - **Best for**: Independent operations (weather + time)

#### OpenCode

1. **MCP Server Support** (2026 Q1)
   - Native MCP server integration
   - Universal tool compatibility
   - **Best for**: Cross-environment skills

2. **Skill Rules Enhancement** (2025)
   - Regex pattern matching in triggers
   - File path filtering
   - **Best for**: Automatic skill loading

#### MCP Protocol

1. **Apps Extension** (2025 Q4)
   - Standard for UI-embedded AI chatbots
   - Served by MCP servers
   - **Best for**: Custom AI interfaces

2. **Experimental Skills Extension** (2026)
   - Skills discovery via MCP primitives
   - Distributed by Skills Over MCP Interest Group
   - **Status**: Experimental, watching for standardization

3. **SDK Expansions** (2025-2026)
   - PHP SDK (2025)
   - Ruby SDK (2025)
   - Swift SDK (2026)
   - Go SDK (2026)
   - **Coverage**: 10+ major languages now supported

### Emerging Trends

#### 1. **Unified Tool Registries**

**Trend**: Moving toward centralized tool registries accessible across environments.

**Examples**:
- **MCP Registry**: Official registry at https://github.com/mcp
- **GitHub Marketplace**: Copilot extensions
- **OpenCode Plugins**: Community-contributed skills

**Benefit**: Write once, use everywhere

#### 2. **AI-Generated Skills**

**Trend**: AI systems generating their own skill definitions.

**Example Workflow**:
```
User: "Create a skill for building this project"
AI: Analyzes project structure
AI: Generates SKILL.md with appropriate commands
AI: Tests commands work correctly
AI: Commits to repository
```

**2026 Status**: Early experimentation, not yet mainstream

#### 3. **Standardized Error Responses**

**Trend**: Error handling converging on common patterns.

**Emerging Standard**:
```json
{
  "error": {
    "code": "BUILD_TIMEOUT",
    "message": "Build exceeded 120s timeout",
    "suggestions": [
      "Use dev mode: ./scripts/build-rust.sh dev",
      "Reduce parallel jobs: CARGO_BUILD_JOBS=4"
    ],
    "documentation": "https://example.com/docs/build-timeouts"
  }
}
```

**Benefit**: Consistent error handling across environments

#### 4. **Performance Metrics in Skills**

**Trend**: Skills documenting their own performance characteristics.

**Example**:
```markdown
## Performance

| Operation | Target (P95) | Actual | Status |
|-----------|-------------|--------|--------|
| Build | < 2m | 1m 45s | ✅ |
| Test | < 30s | 22s | ✅ |
| Lint | < 10s | 8s | ✅ |
```

**2026 Status**: Best practice in OpenCode, emerging elsewhere

#### 5. **Security Auditing in Skills**

**Trend**: Skills documenting security considerations.

**Example**:
```markdown
## Security Considerations

**Inputs**:
- `--target`: Validated against whitelist
- `--env`: Must be one of [dev, staging, prod]

**Outputs**:
- Build artifacts: Written to `./target/` only
- Logs: May contain file paths, not secrets
- No credential leakage: Uses `$GITHUB_TOKEN` env var

**Dependencies**:
- Requires: cargo, rustc
- Network: Downloads crates.io dependencies
- File Access: Read-only to source tree
```

**2026 Status**: Emerging best practice, not yet universal

---

## Decision Matrix: Which Pattern to Use?

### Quick Decision Guide

**Use Direct Bash CLI (OpenCode pattern) when**:
- ✅ Commands are simple (1-2 steps)
- ✅ High-frequency operations (development workflow)
- ✅ Team comfortable with bash
- ✅ Environment supports bash code blocks
- ❌ Don't need to run in different AI environments

**Use Structured Tools (Claude Code pattern) when**:
- ✅ Building API-driven application
- ✅ Need JSON schema validation
- ✅ Using Claude Code or compatible API
- ✅ Want strict type checking (`strict: true`)
- ❌ Don't need cross-environment portability

**Use MCP Protocol when**:
- ✅ Need cross-environment compatibility
- ✅ Building for multiple AI platforms
- ✅ Want ecosystem integration
- ✅ Require discoverability
- ❌ Don't mind extra implementation complexity

**Use Hybrid Approach when**:
- ✅ Want both portability (MCP) and simplicity (bash)
- ✅ Have simple commands (bash) + complex tools (MCP)
- ✅ Team uses multiple AI environments
- ❌ Have maintenance capacity for both systems

### Environment-Specific Recommendations

| If Using... | Recommended Pattern | Why |
|--------------|-------------------|-----|
| **OpenCode only** | Direct bash CLI + YAML frontmatter | Simplest, fastest, native support |
| **Claude Code only** | API tools + `strict: true` | Best validation, production reliability |
| **Both OpenCode + Claude** | MCP Protocol | Write once, use both environments |
| **Cursor + VS Code** | Natural language rules | IDE-native integration |
| **Multi-environment team** | MCP Protocol | Cross-platform compatibility |
| **Simple project** | Direct bash CLI | Lowest complexity |
| **Complex production system** | MCP Protocol | Scalability, discoverability |

---

## Implementation Roadmap

### Step 1: Audit Current Skills

**Action**: Inventory existing SKILL.md files

```bash
# Find all skill files
find . -name "SKILL.md" -o -name "skill-*.md"

# Check for YAML frontmatter
head -5 **/SKILL.md | grep -q "^---" && echo "✓ Has frontmatter"

# Check for bash code blocks
grep -c '```bash' **/SKILL.md
```

**Output**: Spreadsheet with:
- Skill name
- Environment (OpenCode/Claude/etc.)
- Has frontmatter? (Y/N)
- Bash code block count
- Sub-file references (progressive disclosure)
- Estimated token count

### Step 2: Choose Target Pattern

**Based on audit**:
- If **single environment**: Use environment's native pattern
- If **multi-environment**: Implement MCP protocol
- If **uncertain**: Start with direct bash (easiest to migrate later)

### Step 3: Standardize YAML Frontmatter

**Universal Template**:
```yaml
---
name: skill-name
description: Action-oriented description of what this does and when to use it.
version: "1.0.0"
tags: [rust, testing, cli]
priority: high
triggers:
  keywords: ["keyword1", "keyword2"]
  patterns: [".*pattern.*"]
  files: ["**/matching-files"]
---
```

**Apply to all skills** for consistency.

### Step 4: Implement Progressive Disclosure

**For skills > 300 lines**:
1. Extract sections into separate files
2. Add links in main `SKILL.md`
3. Test links work correctly

**Example**:
```bash
# Extract troubleshooting section
# From: SKILL.md (line 150-300)
# To: troubleshooting.md

# Add link
echo "## Troubleshooting" >> SKILL.md
echo "For common issues and solutions, see **[Troubleshooting](troubleshooting.md)**" >> SKILL.md
```

### Step 5: Document CLI Commands

**For every command**:
1. Add comment explaining what it does
2. Document expected output
3. Specify when to use it
4. List prerequisites

**Template**:
```markdown
## Action Name

**Prerequisites**: List what's needed before running

**What This Does**: 1-2 sentences explaining the action

**Expected Output**: Example of successful result

```bash
# Comment explaining command
command_here --with --flags
```

**Use When**:
- Situation 1: Description
- Situation 2: Description
```

### Step 6: Add Troubleshooting Sections

**For complex skills** with potential failure modes:

1. **Identify common errors** (from team experience)
2. **Document symptoms** (what users see)
3. **Provide diagnostic commands** (how to verify)
4. **Offer solutions** (how to fix)
5. **List prevention** (how to avoid)

### Step 7: Test and Validate

**Validation Checklist**:
```markdown
## Testing

- [ ] YAML frontmatter valid
- [ ] All links work (sub-file references)
- [ ] All commands execute without errors
- [ ] Troubleshooting tested with real errors
- [ ] Security review (no hardcoded secrets)
- [ ] Performance acceptable (measured if applicable)
- [ ] Team review for clarity
```

### Step 8: Cross-Environment Verification

**If targeting multiple environments**:

1. **Test in OpenCode**: Load skill, invoke tools
2. **Test in Claude Code**: Use tool definitions in API
3. **Test MCP Server**: Connect from different clients
4. **Verify compatibility**: Ensure consistent behavior

**Tools**:
- **OpenCode**: `opencode` CLI with `--test` mode
- **Claude Code**: Test with `claude-opus-4-6` API
- **MCP**: Use `inspector` tool for visual testing

---

## Conclusion

### Key Takeaways for 2026

1. **Universal Pattern**: YAML frontmatter + bash code blocks works everywhere
2. **Progressive Disclosure**: Essential for token optimization and usability
3. **MCP Protocol**: Emerging standard for cross-environment compatibility
4. **Security First**: Never hardcode credentials, validate inputs
5. **Test Everything**: Commands must work as documented

### Quick Start Guide

**For New Skills**:
1. Copy universal template from this guide
2. Fill in YAML frontmatter
3. Add commands with documentation
4. Implement progressive disclosure if > 300 lines
5. Test thoroughly
6. Get team review

**For Existing Skills**:
1. Audit current state
2. Add missing YAML frontmatter
3. Document undocumented commands
4. Implement progressive disclosure
5. Add troubleshooting sections
6. Test and validate

### Resources

**Official Documentation**:
- **Claude Code**: https://docs.anthropic.com/claude/docs/tool-use
- **OpenCode**: https://opencode.ai/docs
- **MCP Protocol**: https://modelcontextprotocol.io
- **GitHub Copilot**: https://github.com/features/copilot

**Community**:
- **OpenCode Discord**: https://opencode.ai/discord
- **MCP Discussions**: https://github.com/orgs/modelcontextprotocol/discussions
- **Claude Community**: https://www.anthropic.com/discord

**Project Examples**:
- **This Project**: `.opencode/skill/`, `.agents/skills/`, `.claude/skills/`
- **MCP Servers**: https://github.com/modelcontextprotocol/servers
- **Claude Cookbooks**: https://github.com/anthropics/claude-cookbooks

---

**Document Version**: 1.0.0  
**Last Updated**: 2026-02-13  
**Next Review**: 2026-04-01 (quarterly updates recommended)

**Maintained By**: AI Research Team  
**Feedback**: Open issue or discussion for corrections/additions
