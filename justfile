set windows-shell := ["pwsh.exe", "-NoLogo", "-Command"]

default:
    @just --list

fmt:
    cargo sort-derives
    cargo fmt
    cargo es-fluent fmt -P web --all
    taplo fmt
    rumdl fmt .

clippy:
    cargo clippy --workspace --all-features

check:
    cargo es-fluent status -P web --all
    cargo check --workspace --all-features

test:
    cargo test --workspace --all-features --all-targets

web:
    dx serve --platform web --package web

cov:
    cargo llvm-cov --workspace --all-features --all-targets
