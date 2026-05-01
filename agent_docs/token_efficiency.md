# Token Efficiency

## Tool Selection Priority (lowest token cost first)
1. **Glob** - File discovery (cheapest, structured)
2. **Grep** - Code search (cheap, file-by-file)
3. **Read** - File inspection (medium)
4. **Bash** - Shell commands (expensive - prefer scripts)

## Verified Patterns
- Grep tool: 1 call → structured file-by-file breakdown
- Glob tool: 1 call → all matching files with paths
- Scripts: 1 call → multiple operations combined

## Target Ratios
- Read:Edit = 2:1 (understand before modifying)
- Bash:Grep = 2:1 (prefer Grep for file/content searches, Bash for file ops/scripts; see [AGENTS.md](../AGENTS.md#tool-selection-enforcement))
- Script:Raw = 3:1 (prefer scripts over raw commands)

## Anti-Patterns (waste tokens)
- `grep -r pattern` in Bash → Use Grep tool
- `find . -name` in Bash → Use Glob tool
- `cat file` in Bash → Use Read tool
- Multiple cargo commands → Use `./scripts/quality-gates.sh`