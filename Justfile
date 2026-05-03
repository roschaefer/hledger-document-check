set shell := ["bash", "-cu"]

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

check: test

