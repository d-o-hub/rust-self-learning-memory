# Architecture Decision Records

ADRs are immutable once accepted. New decisions get the next free number.

## Known duplicate numbers (aliases — historical)

Two numbers were reused early in the project. Prefer the **canonical** filename when linking.

| Number | Canonical (prefer) | Alias (historical) |
|--------|--------------------|--------------------|
| **025** | `ADR-025-Project-Health-Remediation.md` | `ADR-025-Non-Deterministic-BOCPD-Tests.md` (treat as BOCPD side-note under health remediation era) |
| **054** | `ADR-054-DAG-State-Management-WG134.md` | `ADR-054-CloudEvents-EventEmitter.md` (CloudEvents decision; link explicitly by full filename) |

When citing duplicates in new docs, use the **full filename**, not `ADR-025` alone.

## Validation

```bash
./scripts/validate-plans.sh --adrs --identifiers
```

New ADRs must not reuse an existing number. See ADR-039 / ADR-072 for plans governance.
