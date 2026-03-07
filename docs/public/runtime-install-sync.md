---
status: stable
title: Runtime Install Sync
---

# Runtime Install Sync

Installation outputs and runtime behavior must stay synchronized:

- Installed command path must match MCP config entries.
- Profile-specific guidance (`desktop`, `terminal`, `server`) must match runtime expectations.
- Post-install restart/auth guidance must remain accurate.

Guardrails enforce this sync in CI.
