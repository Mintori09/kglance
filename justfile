export RUSTFLAGS := "-C link-arg=-fuse-ld=mold " + env_var_or_default("RUSTFLAGS", "")

all: build test clippy fmt-check kglance

ci:
    RUSTFLAGS='--deny warnings -C link-arg=-fuse-ld=mold' just all

pr: ci
    gh pr create --web

push: ci
    git push

build:
    cargo build --all

test FILTER='':
    cargo test --all {{FILTER}}

clippy:
    cargo clippy --all-targets --all-features

fmt-check:
    cargo +nightly fmt --all -- --check
    @echo formatting check done

kglance:
    ./target/release/kglance

watch +COMMAND='test':
    cargo watch --clear --exec "{{COMMAND}}"

run +arg=".":
    cargo check --no-default-features
    cargo run --no-default-features --bin kglance --release -- "{{arg}}"

release:
    cargo build --release
    notify-send "Build successfully!"

tags version:
    #!/usr/bin/env bash
    set -euo pipefail

    TAG="{{version}}"

    echo "Checking tag: $TAG"

    if git rev-parse "$TAG" >/dev/null 2>&1; then
        echo "Deleting local tag: $TAG"
        git tag -d "$TAG"
    fi

    if git ls-remote --tags --exit-code origin "refs/tags/$TAG" >/dev/null 2>&1; then
        echo "Deleting remote tag on origin: $TAG"
        git push origin --delete "$TAG"
    fi

    echo "Creating new tag: $TAG"
    git tag "$TAG"

    echo "Pushing tag $TAG to origin"
    git push origin "$TAG"
