set shell := ["bash", "-euo", "pipefail", "-c"]

default:
    @just --list

help:
    @just --list

build:
    cargo build --release

run *args="":
    @args="{{args}}"; args="${args#-- }"; cargo run -- $args

test:
    cargo test

test-scrut *paths="README.md tests/specifications":
    cargo build
    PATH="$(realpath "${CARGO_TARGET_DIR:-target}/debug"):$PATH" scrut test {{paths}}

fmt:
    cargo fmt --all

fmt-check:
    cargo fmt --all -- --check

clippy:
    cargo clippy --all-targets -- -D warnings

machete:
    cargo machete

audit:
    cargo audit

check: fmt-check clippy machete audit test

