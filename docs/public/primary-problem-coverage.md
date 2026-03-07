---
status: stable
title: Primary Problem Coverage
---

# Primary Problem Coverage

AgenticForge primary problem: deliver complete, coherent project blueprints before implementation.

## Covered Areas

- Domain-level architecture decomposition
- Entity and dependency modeling
- Wiring/data flow planning
- Blueprint validation and consistency checks
- Token-aware query surfaces for MCP consumers

## Validation Evidence

- `scripts/test-primary-problems.sh` executes the primary regression suite.
- CI guardrails enforce canonical, install, runtime, and primary-problem checks.
