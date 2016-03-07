#!/bin/bash
#
# Build nix and all tests for as many versions and platforms as can be
# managed.  This requires docker.
#

set -e

RUST_VERSIONS=${RUST_VERSIONS:-"\
    1.6.0 \
    1.7.0 \
    beta \
    nightly"}

# Disabled (not working presently) but with some support in the target
# image:
# - i686-apple-darwin
# - x86_64-apple-darwin

RUST_TARGETS=${RUST_TARGETS:-"\
    aarch64-unknown-linux-gnu \
    arm-linux-androideabi \
    arm-unknown-linux-gnueabi \
    arm-unknown-linux-gnueabihf \
    i686-unknown-linux-gnu \
    mips-unknown-linux-gnu \
    mipsel-unknown-linux-gnu \
    x86_64-unknown-linux-gnu \
    x86_64-unknown-linux-musl"}

DOCKER_IMAGE=${DOCKER_IMAGE:-"posborne/rust-cross"}

BASE_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

test_nix() {
    version=$1
    target=$2

    docker run -i -t \
           -v ${BASE_DIR}:/source \
           -e CARGO_TARGET_DIR=/build \
           ${DOCKER_IMAGE} \
           /source/ci/run.sh ${version} ${target}
}

# Ensure up to date (short compared to everything else)
docker pull ${DOCKER_IMAGE}

# Run tests for each version/target combination
for version in ${RUST_VERSIONS}; do
    for target in ${RUST_TARGETS}; do
        test_nix $version $target || true
    done
done
