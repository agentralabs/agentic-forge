---
status: stable
title: Faq
---

# Frequently Asked Questions

## What is AgenticForge?

AgenticForge is Sister #11 in the Agentra Labs ecosystem. It is a blueprint
engine that designs complete project architecture before code generation begins.

## What file format does it use?

AgenticForge uses the `.forge` file format for persisting blueprints.

## How many inventions does it have?

32 inventions across 8 tiers, covering decomposition, entity modeling,
operations, file structure, dependencies, skeleton generation, integration
wiring, and test architecture.

## Can I use it standalone?

Yes. All sister bridges use NoOp defaults, so AgenticForge works without any
other sister installed.

## What domains are supported?

Ten domains: `web`, `api`, `cli`, `library`, `service`, `database`,
`embedded`, `mobile`, `desktop`, `plugin`.

## How do I validate a blueprint?

```bash
aforge blueprint validate <blueprint_id>
```

This runs the BlueprintValidator which checks for missing entities, circular
dependencies, incomplete structures, and other issues.

## How does entity inference work?

The EntityInferrer parses natural language descriptions and extracts domain
objects with fields and relationships. Use `aforge entity infer` or the
`forge_entity_infer` MCP tool.

## What is the MCP transport?

AgenticForge uses stdio JSON-RPC transport following the MCP quality standard.
Error code `-32803` is used for unknown tools.

## Where is blueprint data stored?

In-memory by default. Use `aforge export forge <id> <path>` to persist
blueprints to `.forge` files on disk.
