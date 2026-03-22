#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$SCRIPT_DIR/.."

cd "$ROOT"

echo "==> Running lua_api_gen..."
cargo run -p lua_api_gen

echo "==> Building editor (triggers build.rs)..."
cargo build -p editor

echo "==> Syncing _engine scripts to game projects..."
SRC="editor/scripts/_engine"
for game_dir in games/*/; do
    target="${game_dir}Resources/scripts/_engine"
    if [ -d "$target" ]; then
        echo "    Updating $target"
        rsync -a --delete "$SRC/" "$target/"
    fi
done

echo "==> Launching editor and building game-playtest in parallel..."
cargo build -p game --bin game-playtest --release &
PLAYTEST_BUILD_PID=$!

cargo run -p editor

wait $PLAYTEST_BUILD_PID && echo "==> game-playtest build complete." || echo "==> game-playtest build failed."
