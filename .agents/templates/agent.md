---
name: agent
description: Template for a new .agents agent family sub-agent
---

# {{ Agent Name }}

## Mission

{{ What this agent is responsible for. }}

## Owner family

[{{ Family }}](../{{ family }}.md)

## Inputs

{{ What this agent receives. }}

## Outputs

{{ What this agent produces. }}

## Boundaries

- May call: {{ allowed core modules / commands }}
- Must not: {{ forbidden direct actions }}

## Workflow

1. {{ Step 1 }}
2. {{ Step 2 }}
3. {{ Step 3 }}

## Verify

- [ ] Output reviewed by owner family primary.
- [ ] Tests / gates pass.
