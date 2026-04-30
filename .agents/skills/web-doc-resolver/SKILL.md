---
name: web-doc-resolver
description: Resolve queries or URLs into compact, LLM-ready markdown using a low-cost cascade. Prioritizes llms.txt for structured docs, uses web fetch/search tools for extraction. Use when you need to fetch documentation, resolve web URLs to markdown, search for technical content, or build context from web sources.
allowed-tools: Bash, webfetch, websearch, WebFetch, WebSearch, web_fetch, web_search
---

# Web Documentation Resolver

Resolve query or URL inputs into compact, high-signal markdown using an intelligent cascade.

## Platform Tool Mapping

| Platform | Fetch Tool | Search Tool |
|----------|------------|-------------|
| opencode | `webfetch` | `websearch` |
| claude code | `WebFetch` (MCP) | `WebSearch` (MCP) |
| blackbox | `web_fetch` | `web_search` |
| Python | `scripts/resolve.py` (auto-detects) | `scripts/resolve.py` (auto-detects) |

## Cascade Resolution Strategy

### URL Inputs
1. **llms.txt**: Probe `https://origin/llms.txt` first (free, structured)
2. **Fetch**: Use platform fetch tool for markdown content
3. **Search fallback**: Find cached/mirrored versions if fetch fails

### Query Inputs
1. **Search first**: Use platform search tool (fast, free)
2. **Fetch top results**: Get markdown from promising results

## Python Script

```bash
python scripts/resolve.py "https://docs.rust-lang.org/book/"  # URL
python scripts/resolve.py "Rust async programming"           # Query
python scripts/resolve.py "query" --json --max-chars 4000    # Options
```

## Direct Tool Usage

```bash
# opencode
webfetch https://example.com/llms.txt
webfetch --format markdown https://docs.rust-lang.org/book/
websearch "Rust book documentation"

# claude code (MCP)
WebFetch(url="https://example.com/llms.txt")
WebFetch(url="https://docs.rust-lang.org/book/")
WebSearch(query="Rust book documentation")

# blackbox
web_fetch(url="https://example.com/llms.txt", prompt="Extract all content")
web_search(query="Rust book documentation")
```

## Best Practices

- Check `/llms.txt` first for structured documentation
- Use specific queries: "tokio spawn vs spawn_blocking difference" > "tokio"
- Add year to queries for current info: "Rust async 2026"
- Prefer official docs; try mirrors if primary fails

## Quality Indicators

Good: Code examples, API signatures, config samples, version info, clear structure
Poor: Boilerplate, paywalls, login requirements, heavy ads

## Error Handling

- Cascade fallback on provider failures
- Log errors for debugging
- Search fallback when direct fetch fails

## Testing

```bash
cd .agents/skills/web-doc-resolver
python -m pytest tests/ -v
```

## Files

`scripts/resolve.py` - Main implementation | `tests/test_resolve.py` - Unit tests | `reference.md` - Detailed reference
