CRATES_MANIFEST := crates/Cargo.toml
TARGET_DIR := $$(cargo metadata --manifest-path $(CRATES_MANIFEST) --format-version 1 --no-deps | sed -n 's/.*"target_directory":"\([^"]*\)".*/\1/p')
BOLD := \033[1m
DIM := \033[2m
GREEN := \033[32m
CYAN := \033[36m
RESET := \033[0m

.PHONY: fmt fmt-check check clippy test build ci

fmt:
	cargo fmt --manifest-path $(CRATES_MANIFEST) --all

fmt-check:
	cargo fmt --manifest-path $(CRATES_MANIFEST) --all -- --check

check:
	cargo check --manifest-path $(CRATES_MANIFEST) --workspace --all-targets --all-features

clippy:
	cargo clippy --manifest-path $(CRATES_MANIFEST) --workspace --all-targets --all-features --tests --benches -- -D warnings

test:
	cargo test --manifest-path $(CRATES_MANIFEST) --workspace --all-features

build:
	cargo build --manifest-path $(CRATES_MANIFEST) --workspace --all-targets --all-features --release
	@printf "\n$(BOLD)$(GREEN)Build artifacts$(RESET)\n"
	@printf "  $(CYAN)%-12s$(RESET) %s\n" "daedalus:" "$(TARGET_DIR)/release/daedalus"
	@printf "  $(CYAN)%-12s$(RESET) %s\n" "daedalus-tui:" "$(TARGET_DIR)/release/daedalus-tui"
	@printf "$(DIM)Tip: copy one of these paths or add it to your shell PATH.$(RESET)\n"

ci: fmt-check check clippy test
