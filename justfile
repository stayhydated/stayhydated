set windows-shell := ["pwsh.exe", "-NoLogo", "-Command"]

default:
    @just --list

fmt:
    cargo sort-derives
    cargo fmt
    cargo es-fluent fmt --all
    bun run fmt
    taplo fmt
    rumdl fmt .

clippy:
    cargo clippy --workspace --all-features

check:
    cargo es-fluent status -P web --all
    cargo check --workspace --all-features

test:
    cargo test --workspace --all-features --all-targets

cov:
    cargo llvm-cov --workspace --exclude xtask --exclude web --all-features --all-targets

web-build:
    cargo xtask build web

web:
    dx serve --platform web --package web

web-preview: web-build
    cd web && bun run preview
