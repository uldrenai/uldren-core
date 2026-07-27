set shell := ["bash", "-euo", "pipefail", "-c"]

ci: fmt-check
    cargo check --workspace --all-targets --locked
    cargo test --workspace --locked

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
    if [ -n "$(find crates -mindepth 3 -maxdepth 3 -type f -path '*/tests/*.rs' -print -quit 2>/dev/null)" ]; then cargo test --workspace --test '*' --locked; else echo "no crate integration test targets"; fi
