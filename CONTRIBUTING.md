# Contributing to AgenticForge

## Getting Started

1. Fork the repository
2. Clone your fork
3. Create a feature branch: `git checkout -b feat/my-feature`
4. Make changes
5. Run tests: `cargo test --workspace`
6. Run guardrails: `bash scripts/check-canonical-sister.sh`
7. Commit with conventional prefix: `feat: add my feature`
8. Push and open a pull request

## Commit Style

Use conventional commit prefixes:
- `feat:` -- new feature
- `fix:` -- bug fix
- `chore:` -- maintenance
- `docs:` -- documentation

## Code Standards

- No `.unwrap()` in MCP server code
- All public APIs must have doc comments
- New inventions must be registered in `inventions/mod.rs`
- New MCP tools must be added to the registry and dispatch match
- New CLI commands must be added to the `Commands` enum

## Testing

- Tests follow the `phase[N]_*.rs` pattern for integration tests
- Aim for comprehensive coverage of all inventions
- Run `cargo test --workspace` before submitting

## Architecture Rules

- Core crate has no dependencies on MCP, CLI, or FFI
- MCP and CLI depend only on Core
- FFI wraps Core only
- All bridges use NoOp defaults for standalone operation

## Review Process

All pull requests require review before merging. Ensure:
- All tests pass
- All guardrail scripts pass
- No new `.unwrap()` in production code
- Commit messages follow the conventional style
