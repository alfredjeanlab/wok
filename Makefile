# Makefile for wk project

SHELL := /bin/bash
SPECS_DIR := tests/specs

.PHONY: install check ci validate spec spec-cli spec-remote spec-todo coverage coverage-spec license

install:
	@scripts/install

# Quick checks
#
# Excluded:
#   SKIP `cargo audit`
#   SKIP `cargo deny`
#
check:
	cargo fmt
	cargo clippy --all -- -D warnings
	quench check --fix --no-cloc
	cargo build --workspace
	cargo test --workspace

# Full pre-release checks
ci:
	cargo fmt
	cargo clippy --all -- -D warnings
	quench check --fix
	cargo build --workspace
	cargo test --all
	cargo audit
	cargo deny check licenses bans sources

validate:
	@scripts/validate

# Run specs via script (pass ARGS for options like --filter, --file)
# Runs CLI parallel, then remote sequential
spec:
	@scripts/spec cli --parallel $(ARGS)
	@scripts/spec remote $(ARGS)

spec-cli:
	@scripts/spec cli --parallel $(ARGS)

spec-remote:
	@scripts/spec remote $(ARGS)

spec-todo:
	@scripts/spec --filter-tags todo:implement $(ARGS)

FMT := --html
coverage:
	@cargo llvm-cov clean --workspace
	@if [ -t 1 ] && [ "$(FMT)" = "--html" ]; then cargo llvm-cov $(FMT) --open; else cargo llvm-cov $(FMT); fi

coverage-spec:
	@cargo llvm-cov clean --workspace
	@echo "Running unit tests with coverage..."
	@cargo llvm-cov --no-report
	@echo "Running specs with coverage..."
	-@LLVM_PROFILE_FILE="$(CURDIR)/target/llvm-cov-target/%p-%m.profraw" \
		WK_BIN="$(CURDIR)/target/debug/wok" \
		WK_REMOTE_BIN="$(CURDIR)/target/debug/wk-remote" \
		scripts/spec cli --parallel
	-@LLVM_PROFILE_FILE="$(CURDIR)/target/llvm-cov-target/%p-%m.profraw" \
		WK_BIN="$(CURDIR)/target/debug/wok" \
		WK_REMOTE_BIN="$(CURDIR)/target/debug/wk-remote" \
		scripts/spec remote
	@echo "Generating coverage report..."
	@if [ -t 1 ] && [ "$(FMT)" = "--html" ]; then cargo llvm-cov report $(FMT) --open; else cargo llvm-cov report $(FMT); fi

license:
	@scripts/license
