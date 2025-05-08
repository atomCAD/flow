#!/bin/sh

set -e

echo Checking syntax...
cargo check

echo Running tests...
cargo test --workspace --all-features

echo Running linter check...
cargo clippy --workspace --all-targets --all-features -- -D warnings

echo Running formatting check...
cargo fmt --all -- --check

echo Checking cargo doc...
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps

echo All done!

# End of file
