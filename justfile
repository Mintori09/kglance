all: build test clippy fmt-check kglance

ci:
  RUSTFLAGS='--deny warnings' just all

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
