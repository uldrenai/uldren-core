set shell := ["bash", "-euo", "pipefail", "-c"]

default:
    @just --list --unsorted

ci: fmt-check lint test

fmt:
    deno fmt
    deno fmt --ext md NOTICE
    deno run --frozen --allow-read --allow-write scripts/format-json.ts write
    cargo fmt --all
    cargo fmt --manifest-path tests/contract/production-adapters/Cargo.toml

fmt-check:
    deno fmt --check
    deno fmt --check --ext md NOTICE
    deno run --frozen --allow-read scripts/format-json.ts check
    cargo fmt --all --check
    cargo fmt --manifest-path tests/contract/production-adapters/Cargo.toml --check

lint:
    cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
    cargo clippy --manifest-path tests/contract/production-adapters/Cargo.toml --all-targets --locked -- -W clippy::all -W clippy::pedantic -D warnings

test:
    cargo test --workspace --locked
    cargo test --manifest-path tests/contract/production-adapters/Cargo.toml --locked

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
