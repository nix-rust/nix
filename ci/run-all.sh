#!/bin/bash
#
# This is **not** meant to be run on CI, but rather locally instead. If you're
# on a Linux machine you'll be able to run most of these, but otherwise this'll
# just attempt to run as many platforms as possible!

set -e

RUST_VERSIONS=${RUST_VERSIONS:-"\
    1.6.0 \
    1.7.0 \
    stable \
    beta \
    nightly"}

RUST_TARGETS=${RUST_TARGETS:-"\
    aarch64-unknown-linux-gnu \
    arm-linux-androideabi \
    arm-unknown-linux-gnueabi \
    arm-unknown-linux-gnueabihf \
    i686-apple-darwin \
    i686-unknown-linux-gnu \
    mips-unknown-linux-gnu \
    mipsel-unknown-linux-gnu \
    x86_64-apple-darwin \
    x86_64-unknown-linux-gnu \
    x86_64-unknown-linux-musl"}

DOCKER_IMAGE="rust-crossbuilder"

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

for version in ${RUST_VERSIONS}; do
    for target in ${RUST_TARGETS}; do
        test_nix $version $target
    done
done
