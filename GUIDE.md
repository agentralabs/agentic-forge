# User Guide

## What AgenticForge Does

AgenticForge designs complete project architecture before code generation. It
takes a project description and domain, then produces:

1. **Entities** -- domain objects with fields and relationships
2. **Operations** -- actions with signatures and error flows
3. **Dependencies** -- external libraries resolved automatically
4. **File structure** -- directory layout and source files
5. **Code skeletons** -- compilable stubs for all entities
6. **Wiring diagrams** -- component connections and data flows
7. **Test architecture** -- test cases, fixtures, and mocks

## Workflow

### Step 1: Create a Blueprint

```bash
aforge blueprint create my-app --domain api --description "Task management API"
```

### Step 2: Populate Entities

Manually or via inference:

```bash
aforge entity infer <id> "Users create tasks with deadlines and assign to teams"
```

### Step 3: Resolve Dependencies

```bash
aforge dependency resolve <id>
```

### Step 4: Generate Structure

```bash
aforge structure generate <id>
aforge skeleton create <id>
aforge test generate <id>
aforge wiring create <id>
```

### Step 5: Validate and Export

```bash
aforge blueprint validate <id>
aforge export json <id>
aforge export forge <id> ./output.forge
```

## Using the MCP Server

Start the server for integration with AI assistants:

```bash
aforge serve --mode stdio
```

The 15 MCP tools mirror the CLI workflow. See
[docs/public/mcp-tools.md](docs/public/mcp-tools.md) for details.

## Domains

Choose the domain that best matches your project:

| Domain    | Typical Use                    |
|-----------|--------------------------------|
| web       | Frontend web applications      |
| api       | REST/GraphQL APIs              |
| cli       | Command-line tools             |
| library   | Reusable libraries/crates      |
| service   | Background services/workers    |
| database  | Database-centric applications  |
| embedded  | Embedded/IoT systems           |
| mobile    | Mobile applications            |
| desktop   | Desktop GUI applications       |
| plugin    | Plugin/extension systems       |

Each domain influences layer decomposition, dependency inference, and file
structure generation.
