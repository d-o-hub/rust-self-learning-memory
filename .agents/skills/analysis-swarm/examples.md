# Swarm Examples

## Example: API Design Decision

**Question**: Should we use REST or GraphQL for the new API?

### RYAN Analysis
- Security: GraphQL has over-fetching risks
- Scalability: REST simpler to cache at edges
- Documentation: REST has more mature tooling
- Compliance: Both can meet requirements

### FLASH Analysis
- Speed: GraphQL reduces round trips
- User Experience: Clients get exactly what they need
- Iteration: Schema evolution easier with GraphQL
- MVP: GraphQL frontend-developer friendly

### SOCRATES Questions
- "What's the real traffic pattern?"
- "Who maintains the backend vs frontend?"
- "What's the cost of over-engineering?"

### Consensus
- Use REST for this iteration
- Plan GraphQL migration for v2
- Document decision rationale

## Example: Performance vs Code Quality

**Question**: Accept technical debt for deadline?

### RYAN
- Debt accumulates interest
- Refactoring cost grows exponentially
- Security shortcuts create vulnerabilities

### FLASH
- Users need working features now
- Debt can be repaid iteratively
- Perfect is enemy of good

### Synthesis
- Accept tactical debt with explicit tracking
- Schedule debt repayment in next sprint
- Document all shortcuts taken
