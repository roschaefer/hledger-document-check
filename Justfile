set shell := ["bash", "-cu"]

default:
    @just --list

help:
    @just --list

build:
    cargo build --release

run *args="":
    @args="{{args}}"; args="${args#-- }"; cargo run -- $args

test: test-rust test-scrut

test-rust:
    cargo test

test-scrut:
    cargo build
    DOCUMENT_CHECK_BIN="$(realpath "${CARGO_TARGET_DIR:-target}/debug/hledger-document-check")" scrut test tests/specifications

test-all: test test-scrut

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

