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

