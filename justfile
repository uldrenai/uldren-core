set shell := ["bash", "-euo", "pipefail", "-c"]

ci: fmt-check

all: ci test-integration

fmt:
    test ! -f Cargo.toml || cargo fmt --all

fmt-check:
    test ! -f Cargo.toml || cargo fmt --all --check

test-integration:
    test ! -f Cargo.toml || cargo test --workspace --test '*'
