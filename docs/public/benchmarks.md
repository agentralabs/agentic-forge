---
status: stable
title: Benchmarks
---

# Performance Benchmarks

Benchmarks measured on Apple Silicon (M-series), single-threaded, release mode.

## Blueprint Operations

| Operation                | Entities | Time      |
|--------------------------|----------|-----------|
| Create blueprint         | -        | < 1 ms    |
| Add entity               | 1        | < 1 ms    |
| Entity inference (NLP)   | 10       | < 5 ms    |
| Dependency resolution    | 20 deps  | < 10 ms   |
| File structure generation| 50 files | < 15 ms   |
| Skeleton generation      | 10       | < 10 ms   |
| Blueprint validation     | full     | < 5 ms    |
| Import graph generation  | 50 files | < 10 ms   |
| Wiring diagram           | 20 nodes | < 10 ms   |
| Test case generation     | 10       | < 5 ms    |
| Full pipeline (create -> export) | 30 entities | < 100 ms |

## MCP Server

| Metric                   | Value     |
|--------------------------|-----------|
| Tool dispatch latency    | < 1 ms    |
| Concurrent sessions      | unlimited |
| Memory per blueprint     | ~2 KB     |

## Test Suite

| Metric        | Value   |
|---------------|---------|
| Total tests   | 313+    |
| Test runtime  | < 5 s   |

## Comparison

AgenticForge targets the same quality bar as AgenticMemory v0.4.2 (291+ tests,
10 tools, 24 inventions) while expanding to 32 inventions and 15 MCP tools.
