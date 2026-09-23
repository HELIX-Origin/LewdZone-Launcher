---
name: postmortem
description: Template for incident / bug postmortems
---

# Postmortem: {{ Incident Title }}

## Summary

{{ One-paragraph summary of what happened. }}

## Timeline

| Time | Event |
| --- | --- |
| {{ time }} | {{ event }} |

## Root cause

{{ Detailed explanation. }}

## Impact

{{ Who / what was affected. }}

## Resolution

{{ How it was fixed, including commit/PR links. }}

## Lessons learned

1. {{ Lesson }}
2. {{ Lesson }}

## Action items

- [ ] {{ Preventive action }}
- [ ] {{ Monitoring / test improvement }}
