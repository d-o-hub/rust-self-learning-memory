---
name: web-search-researcher
description: Research topics using web search and content fetching to find accurate, current information. Use when you need modern information, official documentation, best practices, technical solutions, or comparisons beyond your training data. Provides systematic web research with strategic searches, content analysis, and synthesized findings.
---

# Web Search Research

Expert web research specialist skill for finding accurate, relevant information from web sources using WebSearch and WebFetch tools.

## When to Use

Use this skill when you need to:
- Find modern information not in your training data
- Locate official documentation for APIs, libraries, or frameworks
- Research best practices and current recommendations
- Discover technical solutions to specific problems
- Compare technologies, approaches, or tools
- Verify version-specific information or recent changes
- Investigate emerging trends or recent developments

## Core Responsibilities

When conducting web research, you will:

### 1. Analyze the Query

Break down the user's request to identify:
- Key search terms and concepts
- Types of sources likely to have answers (documentation, blogs, forums, academic papers)
- Multiple search angles to ensure comprehensive coverage

### 2. Execute Strategic Searches

- Start with broad searches to understand the landscape
- Refine with specific technical terms and phrases
- Use multiple search variations to capture different perspectives
- Include site-specific searches when targeting known authoritative sources (e.g., "site:docs.stripe.com webhook signature")

### 3. Fetch and Analyze Content

- Use WebFetch to retrieve full content from promising search results
- Prioritize official documentation, reputable technical blogs, and authoritative sources
- Extract specific quotes and sections relevant to the query
- Note publication dates to ensure currency of information

### 4. Synthesize Findings

- Organize information by relevance and authority
- Include exact quotes with proper attribution
- Provide direct links to sources
- Highlight any conflicting information or version-specific details
- Note any gaps in available information

## Search Strategies

### For API/Library Documentation

- Search for official docs first: "[library name] official documentation [specific feature]"
- Look for changelog or release notes for version-specific information
- Find code examples in official repositories or trusted tutorials

**Example Search**:
```
"Stripe webhook signature verification" official documentation
site:stripe.com webhook signature validation
```

### For Best Practices

- Search for recent articles (include year in search when relevant)
- Look for content from recognized experts or organizations
- Cross-reference multiple sources to identify consensus
- Search for both "best practices" and "anti-patterns" to get full picture

**Example Search**:
```
"Rust async best practices" 2024
"Tokio performance anti-patterns"
```

### For Technical Solutions

- Use specific error messages or technical terms in quotes
- Search Stack Overflow and technical forums for real-world solutions
- Look for GitHub issues and discussions in relevant repositories
- Find blog posts describing similar implementations

**Example Search**:
```
"tokio runtime panic" redb write transaction
site:github.com tokio blocking write transaction
```

### For Comparisons

- Search for "X vs Y" comparisons
- Look for migration guides between technologies
- Find benchmarks and performance comparisons
- Search for decision matrices or evaluation criteria

**Example Search**:
```
"redb vs sled" performance comparison
"SQLite vs Turso" migration guide
```

## Search Efficiency

- Start with 2-3 well-crafted searches before fetching content
- Fetch only the most promising 3-5 pages initially
- If initial results are insufficient, refine search terms and try again
- Use search operators effectively:
  - **Quotes** for exact phrases: `"exact phrase"`
  - **Minus** for exclusions: `rust -game`
  - **site:** for specific domains: `site:docs.rs`
  - **OR** for alternatives: `tokio OR async-std`
- Consider searching in different forms: tutorials, documentation, Q&A sites, and discussion forums

## Output Format

Structure your findings as:

```markdown
## Summary
[Brief overview of key findings - 2-3 sentences]

## Detailed Findings

### [Topic/Source 1]
**Source**: [Name with link]
**Relevance**: [Why this source is authoritative/useful]
**Key Information**:
- Direct quote or finding (with link to specific section if possible)
- Another relevant point

### [Topic/Source 2]
**Source**: [Name with link]
**Relevance**: [Why this source is authoritative/useful]
**Key Information**:
- Finding 1
- Finding 2

## Additional Resources
- [Relevant link 1] - Brief description
- [Relevant link 2] - Brief description

## Gaps or Limitations
[Note any information that couldn't be found or requires further investigation]
```

## Quality Guidelines

### Accuracy
- Always quote sources accurately and provide direct links
- Include specific section links when possible
- Preserve technical terminology exactly as written

### Relevance
- Focus on information that directly addresses the user's query
- Filter out tangential or outdated information
- Prioritize actionable insights

### Currency
- Note publication dates and version information when relevant
- Indicate if information may be outdated
- Look for recent updates or newer alternatives

### Authority
- Prioritize official sources, recognized experts, and peer-reviewed content
- Note the credibility of each source
- Be skeptical of unverified claims

### Completeness
- Search from multiple angles to ensure comprehensive coverage
- Don't stop at the first result - validate with multiple sources
- Identify consensus vs. outlier opinions

### Transparency
- Clearly indicate when information is outdated, conflicting, or uncertain
- Acknowledge gaps in available information
- Distinguish between official guidance and community opinions

## Research Workflow

### Step 1: Plan Searches
```markdown
Query: [User's question]
Key concepts: [List main terms]
Search variations:
1. [Broad search]
2. [Specific technical search]
3. [Site-specific search]
```

### Step 2: Execute Searches
- Run 2-3 initial searches
- Review search results for promising sources
- Identify authoritative and relevant URLs

### Step 3: Fetch Content
- Use WebFetch on 3-5 most promising URLs
- Extract relevant information
- Note publication dates and context

### Step 4: Synthesize
- Organize findings by theme/topic
- Identify consensus and conflicts
- Structure using output format template

### Step 5: Report
- Present findings clearly
- Provide actionable insights
- Note any limitations or gaps

## Examples

### Example 1: API Documentation Research

**Query**: "How do I verify webhook signatures in Stripe?"

**Search Strategy**:
1. `"Stripe webhook signature verification" official documentation`
2. `site:stripe.com webhook endpoints security`
3. `"Stripe webhook" signature example code`

**Expected Output**:
- Link to official Stripe webhook security docs
- Code examples for signature verification
- Common pitfalls and best practices
- Version-specific considerations

### Example 2: Best Practices Research

**Query**: "What are the best practices for async Rust error handling?"

**Search Strategy**:
1. `"Rust async error handling" best practices 2024`
2. `"Tokio error handling" patterns`
3. `site:blog.rust-lang.org async errors`
4. `"anyhow vs thiserror" async context`

**Expected Output**:
- Official Rust async book recommendations
- Expert blog posts from recognized Rust developers
- Comparison of error handling libraries
- Real-world examples and patterns

### Example 3: Technical Problem Solving

**Query**: "Why is my Tokio runtime blocking on redb writes?"

**Search Strategy**:
1. `"tokio blocking" redb write transaction`
2. `site:github.com tokio spawn_blocking database`
3. `"redb" async tokio integration`
4. `"database write blocking async runtime"`

**Expected Output**:
- Explanation of blocking operations in async runtimes
- Solutions using spawn_blocking
- GitHub issues with similar problems
- Performance considerations

## Integration with Other Skills

- **episode-start**: Use web research to gather context before starting episodes
- **feature-implement**: Research API documentation and best practices before implementation
- **debug-troubleshoot**: Search for similar error patterns and solutions
- **architecture-validation**: Research architectural patterns and trade-offs

## Best Practices

### DO:
- ✓ Use specific, technical search terms
- ✓ Include version numbers when relevant
- ✓ Search official documentation first
- ✓ Cross-reference multiple sources
- ✓ Note publication dates
- ✓ Provide direct links
- ✓ Quote sources accurately
- ✓ Indicate source authority

### DON'T:
- ✗ Stop at the first search result
- ✗ Trust unverified sources
- ✗ Ignore publication dates
- ✗ Mix up different versions
- ✗ Omit source attribution
- ✗ Make assumptions without verification
- ✗ Overlook conflicting information

## Troubleshooting

### If Search Returns Poor Results
- Refine search terms (more specific or more general)
- Try different keyword combinations
- Use site-specific searches
- Search for related concepts

### If Sources Are Outdated
- Add year to search query
- Look for "latest" or "newest" modifiers
- Check official changelog or release notes
- Search GitHub for recent issues/discussions

### If Information Conflicts
- Identify version differences
- Check publication dates
- Consider source authority
- Note all perspectives in findings

### If No Information Found
- Broaden search scope
- Try alternative terminology
- Search adjacent topics
- Clearly report the gap in findings

## Summary

Web search research is a systematic approach to finding accurate, current information:
1. **Analyze** the query to identify key concepts
2. **Search** strategically using multiple variations
3. **Fetch** content from authoritative sources
4. **Synthesize** findings with proper attribution
5. **Report** organized, actionable insights

Always prioritize accuracy, cite sources, and be transparent about limitations. Your goal is to be the user's expert guide to web information.
