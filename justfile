git_root := `git rev-parse --path-format=absolute --git-common-dir 2>/dev/null | sed 's|/\.git$||'`
export CARGO_TARGET_DIR := env_var_or_default("CARGO_TARGET_DIR", git_root + "/target")
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
    notify-send "Build successfully!"

test *ARGS:
    cargo nextest run --test-threads 3 {{ARGS}}

clippy:
    cargo clippy --all-targets --all-features

fmt-check:
    cargo fmt --all -- --check
    @echo formatting check done

kglance *ARGS="":
    cargo run --bin kglance -- {{ARGS}}

watch +COMMAND='test':
    cargo watch --clear --exec "{{COMMAND}}"

run +arg=".":
    cargo run --bin kglance -- --standalone "{{arg}}"

dev +arg=".":
    cargo run --bin kglance -- "{{arg}}"

restart-daemon:
    -pkill -f "kglance daemon" || true
    cargo run --bin kglance -- daemon

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

# Worktree management
wt-new task base="HEAD":
    #!/usr/bin/env bash
    set -euo pipefail
    root="{{git_root}}"
    wt_dir="$root/.worktrees/{{task}}"

    mkdir -p "$root/.worktrees"
    git worktree add -b "wt/{{task}}" "$wt_dir" "{{base}}"

    for file in .envrc AGENTS.local.md repomix.config.json repomix-output.local.xml; do
        if [ -f "$root/$file" ]; then
            ln -sf "$root/$file" "$wt_dir/$file"
        fi
    done
    echo "✔ Worktree ready at: $wt_dir"

wt-list:
    git worktree list

wt-drop task:
    #!/usr/bin/env bash
    root="{{git_root}}"
    git worktree remove "$root/.worktrees/{{task}}" --force
    git branch -D "wt/{{task}}" || true
    echo "✔ Worktree {{task}} removed"
