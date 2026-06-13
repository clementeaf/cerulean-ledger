#!/usr/bin/env bash
set -euo pipefail

# Fast deploy: cross-compile (with sccache) → optional scp → docker rebuild on remote host
#
# Usage:
#   ./scripts/deploy.sh                                    # build only → dist/
#   DEPLOY_HOST=user@host DEPLOY_KEY=~/.ssh/key.pem ./scripts/deploy.sh
#
# Cerulean AWS EC2 removed 2026-06. Set DEPLOY_HOST + DEPLOY_KEY for remote upload.

BINARY="dist/cerulean-node-linux-amd64"
TARGET="x86_64-unknown-linux-gnu"
REMOTE_DIR="~/rust-bc"
DEPLOY_HOST="${DEPLOY_HOST:-}"
DEPLOY_KEY="${DEPLOY_KEY:-}"

export SCCACHE_DIR="${SCCACHE_DIR:-$HOME/.cache/sccache}"
export RUSTC_WRAPPER="sccache"
export AWS_LC_SYS_CMAKE_BUILDER=1

mkdir -p "$SCCACHE_DIR"

echo "==> [1/4] Cross-compiling for $TARGET (sccache: $SCCACHE_DIR)"
time cross build --release --target "$TARGET" --bin rust-bc

echo "==> [2/4] Copying binary to dist/"
cp "target/$TARGET/release/rust-bc" "$BINARY"
SIZE=$(du -h "$BINARY" | cut -f1)
echo "    Binary: $BINARY ($SIZE)"

if [[ -z "$DEPLOY_HOST" || -z "$DEPLOY_KEY" ]]; then
    echo ""
    echo "==> Build complete (local only)."
    echo "    To deploy remotely: DEPLOY_HOST=user@host DEPLOY_KEY=~/.ssh/key.pem $0"
    exit 0
fi

echo "==> [3/4] Uploading to $DEPLOY_HOST..."
scp -i "$DEPLOY_KEY" "$BINARY" "$DEPLOY_HOST:$REMOTE_DIR/dist/"

echo "==> [4/4] Rebuilding container on remote host..."
ssh -i "$DEPLOY_KEY" "$DEPLOY_HOST" "cd $REMOTE_DIR && \
    docker build -f Dockerfile.prebuilt -t rust-bc-node:latest . && \
    docker compose -f docker-compose.sandbox.yml up -d node"

echo ""
echo "==> Deploy complete. Waiting for health check..."
sleep 3
ssh -i "$DEPLOY_KEY" "$DEPLOY_HOST" "docker logs cerulean-sandbox-node --tail 3 2>&1"
echo "Done."
