set shell := ["bash", "-euo", "pipefail", "-c"]

default:
    @just --list --unsorted

ci: fmt-check lint test

fmt:
    deno fmt
    deno fmt --ext md NOTICE
    cargo fmt --all

fmt-check:
    deno fmt --check
    deno fmt --check --ext md NOTICE
    cargo fmt --all --check

lint:
    cargo clippy --workspace --all-targets --all-features --locked -- -D warnings

test:
    cargo test --workspace --locked

test-integration:
    if [ -n "$(find crates -mindepth 3 -maxdepth 3 -type f -path '*/tests/*.rs' -print -quit 2>/dev/null)" ]; then cargo test --workspace --test '*' --locked; else echo "no crate integration test targets"; fi

coverage:
    cargo llvm-cov --workspace --locked --fail-under-lines 0 --lcov --output-path target/llvm-cov/lcov.info

deny:
    cargo deny --workspace check

machete:
    cargo machete

semver:
    cargo semver-checks check-release --workspace --baseline-rev main

all: ci deny machete coverage semver test-integration
