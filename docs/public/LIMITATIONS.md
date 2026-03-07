---
status: stable
title: AgenticForge Limitations
---

# AgenticForge Limitations

This document tracks known boundaries and non-goals for the current AgenticForge release.

## Scope Boundaries

- AgenticForge focuses on architecture and blueprint planning before code generation.
- It does not execute generated applications in production environments.
- It does not replace downstream runtime validation in sister systems.

## Operational Limits

- Extremely large blueprint graphs may increase query latency and memory usage.
- MCP usage assumes a compliant MCP client and valid workspace context.
- FFI support is intended for controlled integration layers, not direct end-user scripting.

## Future Work

- Expand adaptive planning for very large monorepo topologies.
- Improve incremental recomputation strategies for high-frequency mutation flows.
