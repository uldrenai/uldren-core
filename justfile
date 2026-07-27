set shell := ["bash", "-euo", "pipefail", "-c"]

ci: fmt-check

all: ci test-integration

fmt:
    deno fmt
    deno fmt --ext md NOTICE
    test ! -f Cargo.toml || cargo fmt --all

fmt-check:
    deno fmt --check
    deno fmt --check --ext md NOTICE
    test ! -f Cargo.toml || cargo fmt --all --check

test-integration:
    test ! -f Cargo.toml || cargo test --workspace --test '*'
