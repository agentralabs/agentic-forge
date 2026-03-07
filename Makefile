.PHONY: build test check clean install release lint fmt doc serve guardrails

build:
	cargo build --workspace

test:
	cargo test --workspace

check:
	cargo check --workspace

clean:
	cargo clean

install:
	cargo install --path crates/agentic-forge-cli --force

release:
	cargo build --workspace --release

lint:
	cargo clippy --workspace -- -D warnings

fmt:
	cargo fmt --all

fmt-check:
	cargo fmt --all -- --check

doc:
	cargo doc --workspace --no-deps

serve:
	cargo run -p agentic-forge-cli -- serve --mode stdio

guardrails:
	bash scripts/check-canonical-consistency.sh
	bash scripts/check-command-surface.sh
	bash scripts/check-mcp-consolidation.sh

full-check:
	bash scripts/check-canonical-sister.sh

hardening:
	bash scripts/check-runtime-hardening.sh

primary:
	bash scripts/test-primary-problems.sh

all: fmt-check lint test guardrails
