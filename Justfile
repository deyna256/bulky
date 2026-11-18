check: format-check lint test

format:
    cargo fmt --all

format-check:
    cargo fmt --all -- --check

lint:
    cargo clippy --locked --all-targets -- -D warnings

test:
    cargo test --locked

build:
    cargo build --locked

package:
    cargo package --locked

update:
    cargo update
