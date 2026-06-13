#!/usr/bin/env bash
# Build Cerulean Ledger release binary on a disposable EC2 spot instance.
#
# Usage:
#   ./scripts/build-release.sh              # Spot build → dist/ (S3 upload if bucket set)
#   ./scripts/build-release.sh --deploy     # Above + deploy if DEPLOY_HOST set
#
# Cerulean AWS stack removed 2026-06. Provide current infra via env:
#   BUILD_SUBNET, BUILD_SECURITY_GROUP, BUILD_KEY_NAME, BUILD_SSH_KEY  (required)
#   RELEASE_S3_BUCKET                                                  (optional upload)
#   DEPLOY_HOST, DEPLOY_USER                                           (optional --deploy)
#
# Cost: ~$0.03 per build (c5.2xlarge spot, ~5 min)

set -euo pipefail

REGION="${AWS_REGION:-us-east-1}"
SUBNET="${BUILD_SUBNET:-}"
SG="${BUILD_SECURITY_GROUP:-}"
KEY_NAME="${BUILD_KEY_NAME:-}"
SSH_KEY="${BUILD_SSH_KEY:-}"
INSTANCE_TYPE="${BUILD_INSTANCE_TYPE:-c5.2xlarge}"
AMI="${BUILD_AMI:-ami-0c7217cdde317cfec}"
S3_BUCKET="${RELEASE_S3_BUCKET:-}"
VERSION="${VERSION:-$(date +%Y%m%d-%H%M%S)}"
BINARY_NAME="cerulean-node-linux-amd64"
S3_KEY="releases/${VERSION}/${BINARY_NAME}"
DEPLOY_HOST="${DEPLOY_HOST:-}"
DEPLOY_USER="${DEPLOY_USER:-ec2-user}"

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

require_aws_build_env() {
    local missing=()
    [[ -z "$SUBNET" ]] && missing+=("BUILD_SUBNET")
    [[ -z "$SG" ]] && missing+=("BUILD_SECURITY_GROUP")
    [[ -z "$KEY_NAME" ]] && missing+=("BUILD_KEY_NAME")
    [[ -z "$SSH_KEY" ]] && missing+=("BUILD_SSH_KEY")
    if ((${#missing[@]} > 0)); then
        echo "ERROR: Cerulean AWS build infra removed. Set: ${missing[*]}"
        exit 1
    fi
}

cd "$REPO_ROOT"
require_aws_build_env

echo "=== Cerulean Ledger Release Build ==="
echo "  Version:  $VERSION"
echo "  Instance: $INSTANCE_TYPE (spot)"
if [[ -n "$S3_BUCKET" ]]; then
    echo "  S3:       s3://$S3_BUCKET/$S3_KEY"
else
    echo "  S3:       (skipped — RELEASE_S3_BUCKET not set)"
fi
echo ""

if [[ -n "$S3_BUCKET" ]] && ! aws s3 ls "s3://$S3_BUCKET" --region "$REGION" 2>/dev/null; then
    echo "Creating S3 bucket: $S3_BUCKET"
    aws s3 mb "s3://$S3_BUCKET" --region "$REGION"
fi

echo "=== Packaging source ==="
TAR="/tmp/cerulean-src-${VERSION}.tar.gz"
tar czf "$TAR" \
    --exclude=target --exclude=.git --exclude=node_modules \
    --exclude=block-explorer-vite --exclude=cerulean-voto \
    --exclude=dist --exclude='*.pdf' .
echo "  Source: $(du -h "$TAR" | cut -f1)"

echo "=== Launching build instance ==="
INSTANCE_ID=$(aws ec2 run-instances \
    --region "$REGION" \
    --image-id "$AMI" \
    --instance-type "$INSTANCE_TYPE" \
    --key-name "$KEY_NAME" \
    --subnet-id "$SUBNET" \
    --security-group-ids "$SG" \
    --associate-public-ip-address \
    --instance-market-options '{"MarketType":"spot","SpotOptions":{"SpotInstanceType":"one-time"}}' \
    --block-device-mappings '[{"DeviceName":"/dev/xvda","Ebs":{"VolumeSize":30,"VolumeType":"gp3"}}]' \
    --tag-specifications "ResourceType=instance,Tags=[{Key=Name,Value=cerulean-build-${VERSION}}]" \
    --query 'Instances[0].InstanceId' --output text)

echo "  Instance: $INSTANCE_ID"
echo "  Waiting for running state..."
aws ec2 wait instance-running --region "$REGION" --instance-ids "$INSTANCE_ID"

BUILD_HOST=$(aws ec2 describe-instances --region "$REGION" \
    --instance-ids "$INSTANCE_ID" \
    --query 'Reservations[0].Instances[0].PublicIpAddress' --output text)

echo "  IP: $BUILD_HOST"
echo "  Waiting for SSH..."
for _ in $(seq 1 12); do
    if ssh -i "$SSH_KEY" -o StrictHostKeyChecking=no -o ConnectTimeout=5 "ec2-user@$BUILD_HOST" "echo ok" 2>/dev/null; then
        break
    fi
    sleep 5
done

echo "=== Setting up build environment ==="
ssh -i "$SSH_KEY" -o StrictHostKeyChecking=no "ec2-user@$BUILD_HOST" "
    sudo dnf install -y gcc gcc-c++ make clang clang-devel openssl-devel protobuf-compiler pkg-config perl 2>/dev/null || \
    sudo yum install -y gcc gcc-c++ make clang clang-devel openssl-devel protobuf-compiler pkg-config perl
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain nightly-2025-05-01
"

echo "=== Uploading source ==="
scp -i "$SSH_KEY" -o StrictHostKeyChecking=no "$TAR" "ec2-user@$BUILD_HOST:~/src.tar.gz"

echo "=== Compiling (this takes ~5 min on c5.2xlarge) ==="
ssh -i "$SSH_KEY" -o StrictHostKeyChecking=no "ec2-user@$BUILD_HOST" "
    source \$HOME/.cargo/env
    mkdir -p ~/build && cd ~/build
    tar xzf ~/src.tar.gz
    cargo build --release --bin rust-bc 2>&1 | tail -5
    ls -lh target/release/rust-bc
    file target/release/rust-bc
"

echo "=== Downloading binary ==="
mkdir -p "$REPO_ROOT/dist"
scp -i "$SSH_KEY" -o StrictHostKeyChecking=no \
    "ec2-user@$BUILD_HOST:~/build/target/release/rust-bc" \
    "$REPO_ROOT/dist/$BINARY_NAME"

chmod +x "$REPO_ROOT/dist/$BINARY_NAME"
SIZE=$(du -h "$REPO_ROOT/dist/$BINARY_NAME" | cut -f1)
echo "  Binary: $REPO_ROOT/dist/$BINARY_NAME ($SIZE)"

if [[ -n "$S3_BUCKET" ]]; then
    echo "=== Uploading to S3 ==="
    aws s3 cp "$REPO_ROOT/dist/$BINARY_NAME" "s3://$S3_BUCKET/$S3_KEY" --region "$REGION"
    echo "  s3://$S3_BUCKET/$S3_KEY"
fi

echo "=== Terminating build instance ==="
aws ec2 terminate-instances --region "$REGION" --instance-ids "$INSTANCE_ID" > /dev/null
echo "  $INSTANCE_ID terminated"
rm -f "$TAR"

echo ""
echo "=== Build complete ==="
echo "  Binary: dist/$BINARY_NAME ($SIZE)"
if [[ -n "$S3_BUCKET" ]]; then
    echo "  S3:     s3://$S3_BUCKET/$S3_KEY"
fi
echo "  Cost:   ~\$0.03"

if [[ "${1:-}" == "--deploy" ]]; then
    if [[ -z "$DEPLOY_HOST" ]]; then
        echo ""
        echo "ERROR: --deploy requires DEPLOY_HOST (Cerulean prod EC2 removed 2026-06)."
        exit 1
    fi

    echo ""
    echo "=== Deploying to $DEPLOY_HOST ==="
    scp -i "$SSH_KEY" -o StrictHostKeyChecking=no \
        "$REPO_ROOT/dist/$BINARY_NAME" "$DEPLOY_USER@$DEPLOY_HOST:~/cerulean-node-new"

    ssh -i "$SSH_KEY" -o StrictHostKeyChecking=no "$DEPLOY_USER@$DEPLOY_HOST" "
        chmod +x ~/cerulean-node-new
        cd ~/rust-bc
        docker compose -f docker-compose.sandbox.yml down
        docker compose -f docker-compose.sandbox.yml up -d
    "

    echo "=== Deploy complete ==="
fi
