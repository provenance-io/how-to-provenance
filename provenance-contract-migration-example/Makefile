.PHONY: all
all: fmt build test lint schema

.PHONY: fmt
fmt:
	@cargo fmt --all -- --check

.PHONY: build
build:
	@cargo wasm

.PHONY: doc
doc:
	@cargo doc --open

.PHONY: test
test:
	@cargo unit-test

.PHONY: lint
lint:
	@cargo clippy -- -D warnings

.PHONY: schema
schema:
	@cargo schema

.PHONY: optimize
optimize:
	@docker run --rm -v $(CURDIR):/code \
		--mount type=volume,source=provenance-contract-migration-example_cache,target=/code/target \
		--mount type=volume,source=provenance-contract-migration-example_registry_cache,target=/usr/local/cargo/registry \
		cosmwasm/rust-optimizer:0.12.6
