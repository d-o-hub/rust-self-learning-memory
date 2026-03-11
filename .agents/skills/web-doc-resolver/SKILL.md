---
name: web-doc-resolver
description: Resolve queries or URLs into compact, LLM-ready markdown using a low-cost cascade. Prioritizes llms.txt for structured docs, uses web fetch/search tools for extraction. Use when you need to fetch documentation, resolve web URLs to markdown, search for technical content, or build context from web sources.
allowed-tools: Bash, webfetch, websearch, WebFetch, WebSearch, web_fetch, web_search
---

# Web Documentation Resolver

Resolve query or URL inputs into compact, high-signal markdown for agents and RAG systems using an intelligent cascade.

## When to Use This Skill

Activate this skill when you need to:
- Fetch and parse documentation from a URL
- Search for technical information across the web
- Build context from web sources
- Extract markdown from websites
- Query for technical documentation, APIs, or code examples

## Platform Tool Mapping

This skill works across multiple platforms. Use the appropriate tools for your platform:

| Platform | Fetch Tool | Search Tool |
|----------|------------|-------------|
| **opencode** | `webfetch` | `websearch` |
| **claude code** | `WebFetch` (MCP) | `WebSearch` (MCP) |
| **blackbox** | `web_fetch` | `web_search` |
| **Python script** | Auto-detects available tools | Auto-detects available tools |

## Cascade Resolution Strategy

### For URL inputs

Use this cascade (in order):

1. **Check llms.txt first**: Probe `https://origin/llms.txt` for site-provided structured documentation (free, always check first)
2. **Fetch URL**: Use platform's fetch tool to get markdown content
3. **Search fallback**: Use platform's search tool to find cached/mirrored versions if direct fetch fails

### For query inputs

Use this cascade (in order):

1. **Search first**: Use platform's search tool with relevant query (fast, free)
2. **Fetch top results**: Use fetch tool to get markdown from top search results if needed

## Implementation

### Python Script (scripts/resolve.py)

The skill includes a Python script that auto-detects available tools:

```bash
# Resolve a URL
python scripts/resolve.py "https://docs.rust-lang.org/book/"

# Resolve a query
python scripts/resolve.py "Rust async programming"

# JSON output
python scripts/resolve.py "query" --json

# Custom max chars
python scripts/resolve.py "query" --max-chars 4000

# Force specific backend
python scripts/resolve.py "query" --backend httpx
```

### Direct Tool Usage by Platform

#### opencode
```bash
# Check for llms.txt
webfetch https://example.com/llms.txt

# Fetch URL
webfetch --format markdown https://docs.rust-lang.org/book/

# Search
websearch "Rust book documentation"
```

#### claude code (MCP)
```python
# Check for llms.txt
WebFetch(url="https://example.com/llms.txt")

# Fetch URL
WebFetch(url="https://docs.rust-lang.org/book/")

# Search
WebSearch(query="Rust book documentation")
```

#### blackbox
```python
# Check for llms.txt
web_fetch(url="https://example.com/llms.txt", prompt="Extract all content")

# Fetch URL
web_fetch(url="https://docs.rust-lang.org/book/", prompt="Extract main content")

# Search
web_search(query="Rust book documentation")
```

## Usage Examples

### Basic URL Resolution

```bash
# Using Python script (auto-detects backend)
python scripts/resolve.py "https://docs.rust-lang.org/book/"

# Or use platform tool directly
webfetch https://docs.rust-lang.org/book/  # opencode
```

### Query Resolution

```bash
# Using Python script
python scripts/resolve.py "Rust async programming best practices 2026"

# Or use platform tool directly
websearch "Tokio runtime configuration options"  # opencode
```

### Workflow for Building Context

1. **Check for llms.txt first**: Probe `https://origin/llms.txt`
2. **Fetch content**: Use fetch tool to get markdown from the URL
3. **Search if needed**: Use search tool for additional context or when fetch fails

## Best Practices

- **Check for llms.txt first**: Many documentation sites have `/llms.txt` for structured content
- **Use specific queries**: "rust tokio spawn vs spawn_blocking difference" gets better results than "rust tokio"
- **Filter by date**: Add "2025" or "2026" to queries for current information
- **Prefer official docs**: Always check official documentation first
- **Try multiple sources**: If one URL fails, search for alternative mirrors

## Quality Indicators

Good content has:
- Code examples with language markers
- API signatures and type annotations
- Configuration examples
- Version information
- Clear headings and structure

Poor content has:
- Excessive boilerplate/navigation
- Paywall blocks
- Login requirements
- Heavy advertising

## Error Handling

- Provider failures should trigger cascade fallback
- Use alternative sources when primary sources fail
- Log errors for debugging
- Fall back to search when direct fetch fails

## Testing

Run tests:
```bash
cd .agents/skills/web-doc-resolver
python -m pytest tests/ -v
```

Run samples:
```bash
python samples/sample_basic.py
python samples/sample_json.py
```

## Files

- `scripts/resolve.py` - Main implementation (multi-backend)
- `tests/test_resolve.py` - Unit tests
- `samples/sample_basic.py` - Basic usage examples
- `samples/sample_json.py` - JSON output examples
- `reference.md` - Detailed reference documentation
