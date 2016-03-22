#!/bin/bash
#
# Build nix and all tests for as many versions and platforms as can be
# managed.  This requires docker.
#

set -e

BASE_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
RUN_DOCKER="${BASE_DIR}/ci/run-docker.sh"

export RUST_VERSION=1.7.0

export DOCKER_IMAGE=posborne/rust-cross:x86
RUST_TARGET=i686-unknown-linux-gnu ${RUN_DOCKER}
RUST_TARGET=x86_64-unknown-linux-gnu ${RUN_DOCKER}
RUST_TARGET=x86_64-unknown-linux-musl ${RUN_DOCKER}

export DOCKER_IMAGE=posborne/rust-cross:arm
RUST_TARGET=aarch64-unknown-linux-gnu ${RUN_DOCKER}
RUST_TARGET=arm-linux-gnueabi ${RUN_DOCKER}
RUST_TARGET=arm-linux-gnueabihf ${RUN_DOCKER}

export DOCKER_IMAGE=posborne/rust-cross:mips
RUST_TARGET=mips-unknown-linux-gnu ${RUN_DOCKER}
RUST_TARGET=mipsel-unknown-linux-gnu ${RUN_DOCKER}

export DOCKER_IMAGE=posborne/rust-cross:android ${RUN_DOCKER}
RUST_TARGET=arm-linux-androideabi ${RUN_DOCKER}
