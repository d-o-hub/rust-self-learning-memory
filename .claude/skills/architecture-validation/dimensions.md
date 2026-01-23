# Validation Dimensions

The skill validates across multiple dimensions, dynamically discovered from plan files:

## 1. Component/Module Structure

- Planned crates, packages, modules exist
- Directory organization matches plans
- Component boundaries maintained

## 2. Dependency Architecture

- Dependency rules followed
- No circular dependencies
- Proper abstraction layers
- No unwanted dependencies

## 3. Data Models

- Structs, enums, types match plans
- Required fields present
- Schemas implemented correctly

## 4. APIs and Interfaces

- Public APIs match planned signatures
- Required functions exist
- Traits/interfaces implemented

## 5. Performance Architecture

- Benchmarks exist for targets
- Performance requirements documented
- Resource limits implemented

## 6. Security Architecture

- Security measures implemented
- Attack surfaces addressed
- Input validation present
- No hardcoded secrets

## 7. Testing Strategy

- Test types match plan
- Coverage requirements met
- Test infrastructure present

## 8. Integration Patterns

- External integrations match design
- Communication patterns correct
- Protocol implementations compliant
