---
status: stable
title: Playbooks Agent Integration
---

# Playbooks Agent Integration

## Goal

Define repeatable integration playbooks for agent orchestration around AgenticForge.

## Baseline Playbook

1. Install and verify `aforge` and MCP server availability.
2. Initialize blueprint workflow context.
3. Run architecture/validation passes before code generation.
4. Persist artifacts and hand off to downstream sisters.

## Guardrails

- Keep MCP calls deterministic and schema-valid.
- Enforce runtime hardening checks in CI before release.
- Ensure post-install and runtime docs remain synchronized.
