#!/bin/bash
#
# Run the nix tests in a docker container.  This script expects the following
# environment variables to be set:
# - DOCKER_IMAGE : Docker image to use for testing (e.g. posborne/rust-cross:arm)
# - RUST_VERSION : Rust Version to test against (e.g. 1.7.0)
# - RUST_TARGET  : Target Triple to test

BASE_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

docker run -i -t \
       -v ${BASE_DIR}:/source \
       -e CARGO_TARGET_DIR=/build \
       ${DOCKER_IMAGE} \
       /source/ci/run.sh ${RUST_VERSION} ${RUST_TARGET}
