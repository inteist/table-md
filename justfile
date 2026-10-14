default:
    @just --list

build:
    cargo build --release

install:
    cargo install --path .

run *args:
    cargo run --quiet -- {{args}}

test:
    cargo test --all-features
    cargo test --no-default-features

fmt:
    cargo fmt

lint:
    cargo clippy --all-targets --all-features -- -D warnings

check:
    cargo fmt --check
    cargo clippy --all-targets --all-features -- -D warnings
    cargo test --all-features
    cargo test --no-default-features

clean:
    cargo clean
