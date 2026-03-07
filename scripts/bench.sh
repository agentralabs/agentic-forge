#!/usr/bin/env bash
set -euo pipefail
echo "Running AgenticForge benchmarks..."
cargo bench -p agentic-forge-core
echo "Benchmark complete. Results in target/criterion/"
