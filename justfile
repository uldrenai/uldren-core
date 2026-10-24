set shell := ["bash", "-euo", "pipefail", "-c"]

default:
    @just --list --unsorted

ci: fmt-check lint test tooling-test

fmt:
    deno fmt
    deno fmt --ext md NOTICE
    deno run --frozen --allow-read --allow-write scripts/format-json.ts write
    cargo fmt --all
    cargo fmt --manifest-path tests/contract/production-adapters/Cargo.toml
    cargo fmt --manifest-path fuzz/Cargo.toml

fmt-check:
    deno fmt --check
    deno fmt --check --ext md NOTICE
    deno run --frozen --allow-read scripts/format-json.ts check
    cargo fmt --all --check
    cargo fmt --manifest-path tests/contract/production-adapters/Cargo.toml --check
    cargo fmt --manifest-path fuzz/Cargo.toml --check

lint:
    cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
    cargo clippy --manifest-path tests/contract/production-adapters/Cargo.toml --all-targets --locked -- -W clippy::all -W clippy::pedantic -D warnings

test:
    cargo test --workspace --locked
    cargo test --manifest-path tests/contract/production-adapters/Cargo.toml --locked

test-integration:
    if [ -n "$(find crates -mindepth 3 -maxdepth 3 -type f -path '*/tests/*.rs' -print -quit 2>/dev/null)" ]; then cargo test --workspace --test '*' --locked; else echo "no crate integration test targets"; fi

tools-install:
    deno run --frozen --allow-read --allow-write --allow-env --allow-run scripts/quality.ts install

tooling-test:
    deno check --frozen scripts/quality.ts scripts/quality.test.ts scripts/quality-smoke.ts
    deno lint scripts/quality.ts scripts/quality.test.ts scripts/quality-smoke.ts
    deno test --frozen --allow-read --allow-env scripts/quality.test.ts

tooling-smoke:
    deno run --frozen --allow-read --allow-write --allow-env --allow-run scripts/quality-smoke.ts

tooling-changes baseline:
    deno run --frozen --allow-read --allow-env --allow-run scripts/quality.ts smoke-if-changed "{{baseline}}"

coverage:
    deno run --frozen --allow-read --allow-write --allow-env --allow-run scripts/quality.ts coverage

deny:
    deno run --frozen --allow-read --allow-write --allow-env --allow-run scripts/quality.ts deny

machete:
    deno run --frozen --allow-read --allow-env --allow-run scripts/quality.ts machete

semver baseline="origin/main":
    deno run --frozen --allow-read --allow-env --allow-run scripts/quality.ts semver "{{baseline}}"

fuzz-smoke:
    deno run --frozen --allow-read --allow-write --allow-env --allow-run scripts/quality.ts fuzz

quality: deny machete coverage

all: ci quality semver fuzz-smoke tooling-smoke test-integration
