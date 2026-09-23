---
name: skill
description: Template for a new .agents skill
---

# Skill: {{ Skill Name }}

## When to use

{{ What task this skill is for. }}

## Owner agent families

- {{ family }}
- {{ family }}

## Inputs

{{ Data / context needed. }}

## Outputs

{{ Artifacts produced. }}

## Steps

1. {{ Step 1 }}
2. {{ Step 2 }}
3. {{ Step 3 }}

## Verification checklist

- [ ] Step 1 completed and checked.
- [ ] Step 2 completed and checked.
- [ ] Step 3 completed and checked.
- [ ] Output artifacts exist and are committed.
- [ ] Relevant tests / gates pass.

## Related

- Agent: `../agents/{{ family }}/{{ family }}.md`
- Rule: `../rules/rule-{{ number }}-{{ name }}.md`
