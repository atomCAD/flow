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

echo Building book...
sh -c "cd book && mdbook build"

echo Running mdbook tests...
sh -c "cd book && mdbook test"

echo Running code coverage...
cargo llvm-cov --workspace --all-features \
    --fail-uncovered-lines 0 \
    --fail-uncovered-regions 0 \
    --fail-uncovered-functions 0 \
    --show-missing-lines

echo All done!

# End of file
