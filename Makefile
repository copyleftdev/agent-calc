# agent-calc developer and local-install workflow.
#
# Philosophy:
# - Fast targets should stay fast.
# - Heavy targets should be explicit.
# - Install should respect standard local prefixes.
# - CI/release targets should compose smaller targets without hiding failures.

SHELL := /usr/bin/env bash
.SHELLFLAGS := -eu -o pipefail -c

.DEFAULT_GOAL := help

BIN_NAME := agent-calc
PACKAGE := agent-calc
TARGET_DIR := target
PROFILE ?= release
PREFIX ?= $(HOME)/.local
BINDIR ?= $(PREFIX)/bin
INSTALL_PATH := $(BINDIR)/$(BIN_NAME)

CARGO ?= cargo
CARGO_FLAGS ?=
CARGO_TEST_FLAGS ?=
CARGO_MUTANTS_FLAGS ?=

RUST_SOURCES := $(shell find src tests -type f -name '*.rs' 2>/dev/null)
MANIFESTS := Cargo.toml Cargo.lock mutants.toml

.PHONY: help
help: ## Show available targets.
	@awk 'BEGIN {FS = ":.*##"; printf "\nOddly Exact / agent-calc Makefile\n\nUsage:\n  make <target>\n\nTargets:\n"} /^[a-zA-Z0-9_.-]+:.*##/ {printf "  \033[36m%-20s\033[0m %s\n", $$1, $$2}' $(MAKEFILE_LIST)
	@printf "\nVariables:\n"
	@printf "  PREFIX=%s\n" "$(PREFIX)"
	@printf "  BINDIR=%s\n" "$(BINDIR)"
	@printf "  PROFILE=%s\n" "$(PROFILE)"
	@printf "\nExamples:\n"
	@printf "  make test\n"
	@printf "  make install PREFIX=$$HOME/.local\n"
	@printf "  make full\n\n"

.PHONY: all
all: fmt-check check clippy test package ## Run the standard local release gate, excluding mutation.

.PHONY: full
full: all mutants ## Run the full release gate, including mutation testing.

.PHONY: ci
ci: all ## Alias for the non-mutating CI-safe gate.

.PHONY: ready
ready: full ## Alias for the strongest local readiness gate.

.PHONY: best
best: full ## Run the strongest Makefile target.

.PHONY: build
build: ## Build the debug binary.
	$(CARGO) build $(CARGO_FLAGS)

.PHONY: release
release: ## Build the optimized release binary.
	$(CARGO) build --release $(CARGO_FLAGS)

.PHONY: run
run: ## Run agent-calc with ARGS='...'.
	$(CARGO) run $(CARGO_FLAGS) -- $(ARGS)

.PHONY: describe
describe: ## Emit the executable contract JSON.
	$(CARGO) run $(CARGO_FLAGS) -- describe

.PHONY: schema
schema: ## Emit a JSON schema. Use DOMAIN=finance, solve, matrix, etc.
	$(CARGO) run $(CARGO_FLAGS) -- schema $(DOMAIN)

.PHONY: fmt
fmt: ## Format Rust sources.
	$(CARGO) fmt --all

.PHONY: fmt-check
fmt-check: ## Check Rust formatting without writing changes.
	$(CARGO) fmt --all -- --check

.PHONY: check
check: ## Type-check all targets.
	$(CARGO) check --all-targets $(CARGO_FLAGS)

.PHONY: clippy
clippy: ## Run clippy when the component is installed.
	@if rustup component list --installed 2>/dev/null | grep -q '^clippy'; then \
		$(CARGO) clippy --all-targets $(CARGO_FLAGS) -- -D warnings; \
	else \
		echo "clippy component is not installed; run: rustup component add clippy"; \
		exit 2; \
	fi

.PHONY: test
test: ## Run the full test suite.
	$(CARGO) test $(CARGO_FLAGS) $(CARGO_TEST_FLAGS)

.PHONY: test-quiet
test-quiet: ## Run tests with quiet cargo output.
	$(CARGO) test --quiet $(CARGO_FLAGS) $(CARGO_TEST_FLAGS)

.PHONY: mutants
mutants: ## Run cargo-mutants.
	$(CARGO) mutants $(CARGO_MUTANTS_FLAGS)

.PHONY: mutants-list
mutants-list: ## List discovered mutants without running them.
	$(CARGO) mutants --list

.PHONY: bench
bench: ## Run benchmarks if any are present.
	$(CARGO) bench $(CARGO_FLAGS)

.PHONY: package
package: ## Verify the crate can be packaged.
	$(CARGO) package --allow-dirty --locked --offline $(CARGO_FLAGS)

.PHONY: audit
audit: ## Run cargo-audit if installed.
	@if command -v cargo-audit >/dev/null 2>&1; then \
		cargo audit; \
	else \
		echo "cargo-audit is not installed; install with: cargo install cargo-audit"; \
		exit 2; \
	fi

.PHONY: deny
deny: ## Run cargo-deny if installed.
	@if command -v cargo-deny >/dev/null 2>&1; then \
		cargo deny check; \
	else \
		echo "cargo-deny is not installed; install with: cargo install cargo-deny"; \
		exit 2; \
	fi

.PHONY: security
security: audit deny ## Run supply-chain checks that are available locally.

.PHONY: install
install: release ## Install optimized binary to BINDIR. Override PREFIX or BINDIR as needed.
	install -d "$(BINDIR)"
	install -m 0755 "$(TARGET_DIR)/release/$(BIN_NAME)" "$(INSTALL_PATH)"
	@echo "installed $(BIN_NAME) -> $(INSTALL_PATH)"
	@echo "make sure $(BINDIR) is on PATH"

.PHONY: install-local
install-local: install ## Explicit alias for local install.

.PHONY: install-cargo
install-cargo: ## Install via cargo install --path.
	$(CARGO) install --path . --locked --force --root "$(PREFIX)"
	@echo "installed $(BIN_NAME) under $(PREFIX)/bin"

.PHONY: uninstall
uninstall: ## Remove the locally installed binary from BINDIR.
	@if [ -e "$(INSTALL_PATH)" ]; then \
		rm -f "$(INSTALL_PATH)"; \
		echo "removed $(INSTALL_PATH)"; \
	else \
		echo "$(INSTALL_PATH) is not installed"; \
	fi

.PHONY: uninstall-local
uninstall-local: uninstall ## Explicit alias for local uninstall.

.PHONY: reinstall
reinstall: uninstall install ## Rebuild and reinstall the local binary.

.PHONY: install-hooks
install-hooks: ## Install git hooks from hooks/ via core.hooksPath (no file copying needed).
	git config core.hooksPath hooks
	@echo "git hooks active — hooks/ is now the hooks directory"
	@echo "pre-commit: fmt + check + clippy + test + package + smoke"
	@echo "pre-push:   mutation testing on changed src/ files (main only)"

.PHONY: uninstall-hooks
uninstall-hooks: ## Remove the core.hooksPath override and revert to .git/hooks/.
	git config --unset core.hooksPath || true
	@echo "hooks directory reverted to .git/hooks/"

.PHONY: smoke
smoke: build ## Run cheap CLI smoke checks against the debug binary.
	"$(TARGET_DIR)/debug/$(BIN_NAME)" --version
	"$(TARGET_DIR)/debug/$(BIN_NAME)" describe >/dev/null
	"$(TARGET_DIR)/debug/$(BIN_NAME)" schema >/dev/null
	printf '%s\n' '{"expr":{"kind":"add","left":{"kind":"integer","value":"2"},"right":{"kind":"integer","value":"3"}}}' | "$(TARGET_DIR)/debug/$(BIN_NAME)" eval >/dev/null
	printf '%s\n' '{"intent":"eval","expr":{"kind":"add","left":{"kind":"integer","value":"2"},"right":{"kind":"integer","value":"3"}}}' | "$(TARGET_DIR)/debug/$(BIN_NAME)" trace >/dev/null

.PHONY: clean
clean: ## Remove cargo build artifacts.
	$(CARGO) clean

.PHONY: clean-mutants
clean-mutants: ## Remove cargo-mutants output directories.
	rm -rf mutants.out mutants.out.old

.PHONY: distclean
distclean: clean clean-mutants ## Remove all generated local artifacts.

.PHONY: docs
docs: ## Build local Rust docs.
	$(CARGO) doc --no-deps $(CARGO_FLAGS)

.PHONY: open-docs
open-docs: docs ## Print the local docs entry point.
	@printf '%s\n' "$(PWD)/$(TARGET_DIR)/doc/$(subst -,_,$(PACKAGE))/index.html"

.PHONY: tree
tree: ## Show the important project files.
	@printf "Package: %s\nBinary:  %s\nPrefix:  %s\n\n" "$(PACKAGE)" "$(BIN_NAME)" "$(PREFIX)"
	@printf "Rust sources:\n"
	@printf '  %s\n' $(RUST_SOURCES)
	@printf "\nManifests:\n"
	@printf '  %s\n' $(MANIFESTS)

.PHONY: version
version: ## Print toolchain and package versions.
	@printf "package: %s\n" "$(PACKAGE)"
	@$(CARGO) --version
	@rustc --version
	@rustup show active-toolchain 2>/dev/null || true
