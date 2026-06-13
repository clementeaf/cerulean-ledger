#!/usr/bin/env bash
# Build a fully static cerulean-node binary (musl-linked).
#
# Usage:
#   ./scripts/build-static.sh              # Build image + extract binary
#   DEPLOY_HOST=host DEPLOY_KEY=~/.ssh/key.pem ./scripts/build-static.sh --deploy
#
# Output: ./dist/cerulean-node-linux-amd64
#
# Note: requires Dockerfile.static (removed from repo). Use deploy.sh or cargo build instead.
# The binary has ZERO runtime dependencies — runs on any Linux.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
DIST="$REPO_ROOT/dist"
BINARY="cerulean-node-linux-amd64"
IMAGE="cerulean-node:static"

cd "$REPO_ROOT"

if [[ ! -f Dockerfile.static ]]; then
    echo "ERROR: Dockerfile.static not found."
    echo "  Use:  cross build --release --target x86_64-unknown-linux-gnu --bin rust-bc"
    echo "  Or:   ./scripts/deploy.sh  (cross-compile to dist/)"
    exit 1
fi

echo "=== Building static binary (musl) ==="
docker build -f Dockerfile.static -t "$IMAGE" .

echo "=== Extracting binary ==="
mkdir -p "$DIST"
CONTAINER_ID=$(docker create "$IMAGE")
docker cp "$CONTAINER_ID:/usr/local/bin/cerulean-node" "$DIST/$BINARY"
docker rm "$CONTAINER_ID" > /dev/null

chmod +x "$DIST/$BINARY"
SIZE=$(du -h "$DIST/$BINARY" | cut -f1)
echo "=== Built: $DIST/$BINARY ($SIZE) ==="
file "$DIST/$BINARY"

if [[ "${1:-}" == "--deploy" ]]; then
    DEPLOY_KEY="${DEPLOY_KEY:-}"
    DEPLOY_USER="${DEPLOY_USER:-ec2-user}"
    DEPLOY_HOST="${DEPLOY_HOST:-}"

    if [[ -z "$DEPLOY_HOST" || -z "$DEPLOY_KEY" ]]; then
        echo ""
        echo "ERROR: --deploy requires DEPLOY_HOST and DEPLOY_KEY (Cerulean EC2 removed 2026-06)."
        exit 1
    fi

    echo ""
    echo "=== Deploying to $DEPLOY_HOST ==="
    scp -i "$DEPLOY_KEY" -o StrictHostKeyChecking=no \
        "$DIST/$BINARY" "$DEPLOY_USER@$DEPLOY_HOST:~/cerulean-node"

    ssh -i "$DEPLOY_KEY" -o StrictHostKeyChecking=no "$DEPLOY_USER@$DEPLOY_HOST" \
        "chmod +x ~/cerulean-node && echo 'Deployed: \$(~/cerulean-node --version 2>/dev/null || echo ok)'"

    echo "=== Deploy complete ==="
fi
