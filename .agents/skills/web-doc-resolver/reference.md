# Web Doc Resolver - Reference

Detailed documentation for web documentation resolution strategies.

## Provider Cascade

### URL Resolution Order

| Priority | Provider | Cost | API Key Required |
|----------|----------|------|------------------|
| 1 | llms.txt | Free | No |
| 2 | Jina Reader | Free | No |
| 3 | Direct HTTP | Free | No |
| 4 | Web Search | Free | No |

### Query Resolution Order

| Priority | Provider | Cost | API Key Required |
|----------|----------|------|------------------|
| 1 | Web Search | Free | No |
| 2 | Web Fetch | Free | No |

## Common Documentation URLs

### Rust Ecosystem

- Rust Book: `https://doc.rust-lang.org/book/`
- Rust Reference: `https://doc.rust-lang.org/reference/`
- Rust by Example: `https://doc.rust-lang.org/rust-by-example/`
- Tokio Docs: `https://tokio.rs/tokio/tutorial`
- Cargo Book: `https://doc.rust-lang.org/cargo/`
- Rust API Guidelines: `https://rust-lang.github.io/api-guidelines/`

### Python

- Python Docs: `https://docs.python.org/3/`
- PyPI: `https://pypi.org/`

### Web Frameworks

- MDN Web Docs: `https://developer.mozilla.org/`
- React: `https://react.dev/`
- Vue: `https://vuejs.org/guide/`

## Query Patterns

### Finding Specific APIs

```
"{library} {method} documentation"
"{framework} {feature} example"
"{language} {pattern} implementation"
```

### Comparing Options

```
"{option1} vs {option2}"
"{alternative} vs {library}"
"best {category} for {use case}"
```

### Finding Tutorials

```
"{topic} tutorial for beginners"
"how to {do something} in {language}"
"{framework} getting started guide"
```

## Content Extraction Tips

### For Code Examples

- Look for fenced code blocks (```)
- Check for file paths in code (src/main.rs, lib.rs)
- Note dependency imports

### For API Docs

- Extract function signatures
- Note return types
- Look for parameter descriptions
- Check error types

### For Configuration

- Look for TOML, YAML, JSON blocks
- Note environment variables
- Extract default values

## Troubleshooting

### No Content Found

- Try alternative URL (check for /docs, /guide, /latest)
- Use websearch to find mirror sites
- Check if site requires authentication

### Poor Quality Content

- Skip promotional pages
- Prefer official documentation
- Add "site:example.com" to search queries

### Rate Limiting

- Add delays between requests
- Use cached results when available
- Try alternative providers
