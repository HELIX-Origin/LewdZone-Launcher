---
name: rule
description: Template for a new numbered project rule
---

# Rule {{ number }}: {{ title }}

## Scope

{{ One-sentence scope: what this rule governs. }}

## Rationale

{{ Why this rule exists. Link to ADRs, threat model, or governance principle if relevant. }}

## Requirements

1. {{ Requirement 1. }}
2. {{ Requirement 2. }}
3. {{ Requirement 3. }}

## Examples

### ✅ Compliant

{{ Example of compliant code or behavior. }}

### ❌ Non-compliant

{{ Example of a violation and why. }}

## Verification

- [ ] Tests demonstrate compliance.
- [ ] Affected code paths were reviewed by the owning agent family.
- [ ] Docs (README/wiki/rules) are updated if behavior changes.

## Related

- Rule {{ related number }} ({{ related title }})
- ADR {{ number }}-{{ slug }}
- `wiki/{{ page }}`
