# Makefile for wk project

SHELL := /bin/bash
SPECS_DIR := checks/specs

.PHONY: help install check validate spec spec-cli spec-remote spec-todo quality stress stress-docker bench coverage coverage-spec license

help:
	@echo "Targets:"
	@echo "  make install     - Build and install wok to ~/.local/bin"
	@echo "  make check       - Run fmt, clippy, check, audit, test"
	@echo "  make validate    - Run all validation checks"
	@echo "  make quality     - Run quality evaluation"
	@echo "  make stress      - Run stress tests (native)"
	@echo "  make stress-docker - Run stress tests in Docker (recommended)"
	@echo "  make bench       - Run benchmarks"
	@echo "  make coverage    - Generate code coverage report (unit tests)"
	@echo "  make coverage-spec - Generate coverage from unit tests + specs"
	@echo "  make license     - Add license headers to source files"
	@echo ""
	@echo "Spec Targets:"
	@echo "  make spec                            - Run all specs"
	@echo "  make spec-cli                        - Run CLI specs"
	@echo "  make spec-remote                     - Run remote specs"
	@echo "  make spec-todo                       - Run unimplemented specs"
	@echo ""
	@echo "Spec Options (via ARGS):"
	@echo "  make spec ARGS='--filter \"pattern\"'  - Filter tests by name"
	@echo "  make spec ARGS='--file path.bats'    - Run specific file"
	@echo "  make spec-cli ARGS='--filter list'   - Combine suite + filter"

install:
	@scripts/install

check:
	cargo fmt --check
	cargo clippy -- -D warnings
	cargo check
	quench check --fix
	cargo audit
	cargo build --workspace
	cargo test

validate:
	@scripts/validate

quality:
	@checks/quality/evaluate.sh

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

stress:
	@checks/stress/run.sh $(ARGS)

stress-docker:
	@checks/stress/docker-run.sh $(ARGS)

bench:
	@checks/benchmarks/run.sh $(if $(ARGS),$(ARGS),all)

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
		WK_BIN="$(CURDIR)/target/debug/wk" \
		WK_REMOTE_BIN="$(CURDIR)/target/debug/wk-remote" \
		scripts/spec cli --parallel
	-@LLVM_PROFILE_FILE="$(CURDIR)/target/llvm-cov-target/%p-%m.profraw" \
		WK_BIN="$(CURDIR)/target/debug/wk" \
		WK_REMOTE_BIN="$(CURDIR)/target/debug/wk-remote" \
		scripts/spec remote
	@echo "Generating coverage report..."
	@if [ -t 1 ] && [ "$(FMT)" = "--html" ]; then cargo llvm-cov report $(FMT) --open; else cargo llvm-cov report $(FMT); fi

license:
	@scripts/license
