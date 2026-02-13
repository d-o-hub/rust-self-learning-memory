# Extraction Patterns

## Components/Crates

Look for:
- "crate", "component", "module", "package"
- Directory tree structures in code blocks
- Architecture diagrams
- Component lists

## Dependencies

Look for:
- "depends on", "imports", "requires"
- "must not depend", "should not import"
- "flow:", "→", "-->", "⇒"
- Dependency rules and constraints

## Performance

Look for:
- "target:", "metric:", "goal:"
- "<Xms", "P95", "P99", "latency", "throughput"
- Performance requirements tables
- Benchmark specifications

## Security

Look for:
- "security", "threat", "attack", "vulnerability"
- "sanitize", "validate", "authenticate"
- Security requirement lists
- Attack surface descriptions

## Data Models

Look for:
- "struct", "enum", "type", "interface"
- "table", "schema", "field", "column"
- Data model diagrams
- Type definitions

## APIs

Look for:
- "pub fn", "public function", "API"
- "endpoint", "method", "operation"
- Function signatures
- Interface definitions

## Compliance Levels

### Compliant
- Element fully implemented as planned
- No deviations
- All requirements met

### Partial
- Element exists but incomplete
- Some requirements met, others missing
- Functional but not fully compliant

### Non-Compliant
- Element missing entirely
- Significant violations
- Major architectural drift
