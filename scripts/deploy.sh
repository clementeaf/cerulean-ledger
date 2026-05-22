#!/usr/bin/env bash
set -euo pipefail

# Fast deploy: cross-compile (with sccache) → scp → docker rebuild on EC2
# Usage: ./scripts/deploy.sh

EC2_HOST="ec2-user@52.201.112.87"
EC2_KEY="$HOME/.ssh/rust-bc-test.pem"
BINARY="dist/cerulean-node-linux-amd64"
TARGET="x86_64-unknown-linux-gnu"
REMOTE_DIR="~/rust-bc"

# sccache cache dir (shared between host and cross container)
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

echo "==> [3/4] Uploading to EC2..."
scp -i "$EC2_KEY" "$BINARY" "$EC2_HOST:$REMOTE_DIR/dist/"

echo "==> [4/4] Rebuilding container on EC2..."
ssh -i "$EC2_KEY" "$EC2_HOST" "cd $REMOTE_DIR && \
    docker build -f Dockerfile.prebuilt -t rust-bc-node:latest . && \
    docker compose -f docker-compose.sandbox.yml up -d node"

echo ""
echo "==> Deploy complete. Waiting for health check..."
sleep 3
ssh -i "$EC2_KEY" "$EC2_HOST" "docker logs cerulean-sandbox-node --tail 3 2>&1"
echo "Done."
