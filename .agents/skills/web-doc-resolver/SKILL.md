---
name: web-doc-resolver
description: Resolve queries or URLs into compact, LLM-ready markdown using a low-cost cascade. Prioritizes llms.txt for structured docs, uses websearch/webfetch tools for extraction. Use when you need to fetch documentation, resolve web URLs to markdown, search for technical content, or build context from web sources.
allowed-tools: Bash, webfetch, websearch
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

## Cascade Resolution Strategy

### For URL inputs

Use this cascade (in order):

1. **Check llms.txt first**: Probe `https://origin/llms.txt` for site-provided structured documentation (free, always check first)
2. **webfetch fallback**: Use webfetch tool with markdown format
3. **websearch fallback**: Use websearch to find cached/mirrored versions if direct fetch fails

### For query inputs

Use this cascade (in order):

1. **websearch first**: Use websearch with relevant query (fast, free)
2. **Fetch top results**: Use webfetch to get markdown from top search results if needed

## Implementation

### Python Script (scripts/resolve.py)

The skill includes a Python script that wraps the web tools:

```bash
# Resolve a URL
python scripts/resolve.py "https://docs.rust-lang.org/book/"

# Resolve a query
python scripts/resolve.py "Rust async programming"

# JSON output
python scripts/resolve.py "query" --json

# Custom max chars
python scripts/resolve.py "query" --max-chars 4000
```

### Direct Tool Usage

You can also use the tools directly following the cascade:

```python
# Step 1: Check for llms.txt
webfetch https://example.com/llms.txt

# Step 2: Fetch the URL directly
webfetch --format markdown https://docs.rust-lang.org/book/

# Step 3: Search if fetch fails
websearch "Rust book documentation"
```

## Usage Examples

### Basic URL Resolution

```bash
# Fetch documentation URL (uses cascade: llms.txt → webfetch → websearch)
python scripts/resolve.py "https://docs.rust-lang.org/book/"

# Or use webfetch directly
webfetch https://docs.rust-lang.org/book/
```

### Query Resolution

```bash
# Search for technical information
python scripts/resolve.py "Rust async programming best practices 2026"

# Or use websearch directly
websearch "Tokio runtime configuration options"
```

### Workflow for Building Context

1. **Check for llms.txt first**: Probe `https://origin/llms.txt`
2. **Fetch content**: Use webfetch to get markdown from the URL
3. **Search if needed**: Use websearch for additional context or when fetch fails

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
- Fall back to websearch when direct fetch fails

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

- `scripts/resolve.py` - Main implementation
- `tests/test_resolve.py` - Unit tests
- `samples/sample_basic.py` - Basic usage examples
- `samples/sample_json.py` - JSON output examples
- `reference.md` - Detailed reference documentation
