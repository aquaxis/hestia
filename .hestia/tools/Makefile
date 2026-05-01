PREFIX ?= /usr/local/bin
CARGO := cargo
TOOLS_DIR := $(shell dirname $(realpath $(lastword $(MAKEFILE_LIST))))
RELEASE_DIR := $(TOOLS_DIR)/target/release

BINARIES := hestia \
	hestia-ai-conductor hestia-rtl-conductor hestia-fpga-conductor \
	hestia-asic-conductor hestia-pcb-conductor hestia-hal-conductor \
	hestia-apps-conductor hestia-debug-conductor hestia-rag-conductor \
	hestia-ai-cli hestia-rtl-cli hestia-fpga-cli hestia-asic-cli \
	hestia-pcb-cli hestia-hal-cli hestia-apps-cli hestia-debug-cli \
	hestia-rag-cli

.PHONY: all build test lint fmt fmt-fix check install clean help

all: build test lint

build:
	$(CARGO) build --release --manifest-path $(TOOLS_DIR)/Cargo.toml

test:
	$(CARGO) test --manifest-path $(TOOLS_DIR)/Cargo.toml

lint:
	$(CARGO) clippy --all-targets --all-features --manifest-path $(TOOLS_DIR)/Cargo.toml -- -D warnings

fmt:
	$(CARGO) fmt --all --manifest-path $(TOOLS_DIR)/Cargo.toml -- --check

fmt-fix:
	$(CARGO) fmt --all --manifest-path $(TOOLS_DIR)/Cargo.toml

check:
	$(CARGO) check --all-targets --manifest-path $(TOOLS_DIR)/Cargo.toml

install: build
	@mkdir -p $(PREFIX)
	@for bin in $(BINARIES); do \
		if [ -f "$(RELEASE_DIR)/$$bin" ]; then \
			echo "  INSTALL $$bin -> $(PREFIX)/$$bin"; \
			cp "$(RELEASE_DIR)/$$bin" "$(PREFIX)/$$bin"; \
			chmod 755 "$(PREFIX)/$$bin"; \
		fi; \
	done
	@echo ""
	@echo "Installed $(shell for b in $(BINARIES); do [ -f "$(RELEASE_DIR)/$$b" ] && echo 1; done | wc -l) binaries to $(PREFIX)"
	@echo "Run 'hestia init' to initialize a project."

uninstall:
	@for bin in $(BINARIES); do \
		if [ -f "$(PREFIX)/$$bin" ]; then \
			echo "  REMOVE  $(PREFIX)/$$bin"; \
			rm -f "$(PREFIX)/$$bin"; \
		fi; \
	done
	@echo "Uninstalled all hestia binaries from $(PREFIX)"

clean:
	$(CARGO) clean --manifest-path $(TOOLS_DIR)/Cargo.toml

help:
	@echo "Hestia — Hardware Engineering Stack for Tool Integration and Automation"
	@echo ""
	@echo "Targets:"
	@echo "  all          Build, test, and lint (default)"
	@echo "  build        Build all binaries in release mode"
	@echo "  test         Run all tests"
	@echo "  lint         Run clippy with warnings as errors"
	@echo "  fmt          Check code formatting"
	@echo "  fmt-fix      Fix code formatting"
	@echo "  check        Run cargo check"
	@echo "  install      Install binaries to PREFIX (default: /usr/local/bin)"
	@echo "  uninstall    Remove installed binaries from PREFIX"
	@echo "  clean        Remove build artifacts"
	@echo "  help         Show this help"
	@echo ""
	@echo "Variables:"
	@echo "  PREFIX       Installation prefix (default: /usr/local/bin)"
	@echo ""
	@echo "Examples:"
	@echo "  make install PREFIX=~/.local/bin"
	@echo "  make build && make test"