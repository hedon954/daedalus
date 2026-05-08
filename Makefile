CRATES_MANIFEST := crates/Cargo.toml
TARGET_DIR := $$(cargo metadata --manifest-path $(CRATES_MANIFEST) --format-version 1 --no-deps | sed -n 's/.*"target_directory":"\([^"]*\)".*/\1/p')

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
	@echo "daedalus: $(TARGET_DIR)/release/daedalus"
	@echo "daedalus-tui: $(TARGET_DIR)/release/daedalus-tui"

ci: fmt-check check clippy test
