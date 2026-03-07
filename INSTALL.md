# Installation

## Prerequisites

- Rust 1.75 or later (install from https://rustup.rs)
- Git

## From Source

```bash
git clone https://github.com/agentralabs/agentic-forge.git
cd agentic-forge
cargo build --workspace --release
cargo install --path crates/agentic-forge-cli
```

## Quick Install

```bash
bash scripts/install.sh
```

## Verify Installation

```bash
aforge version
aforge health
aforge info
```

## FFI Library

To build the C FFI library:

```bash
cargo build --release -p agentic-forge-ffi
```

Output files:
- Static: `target/release/libagentic_forge_ffi.a`
- Dynamic: `target/release/libagentic_forge_ffi.dylib` (macOS)

## Uninstall

```bash
cargo uninstall agentic-forge-cli
```

## Troubleshooting

If `aforge` is not found after installation, ensure `~/.cargo/bin` is in
your PATH:

```bash
export PATH="$HOME/.cargo/bin:$PATH"
```

See [docs/public/troubleshooting.md](docs/public/troubleshooting.md) for
more help.
