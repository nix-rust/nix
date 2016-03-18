#!/bin/bash
#
# Entry point for all travis builds, this will set up the Travis environment by
# downloading any dependencies. It will then execute the `run.sh` script to
# build and execute all tests.
#
# Much of this script was liberally stolen from rust-lang/libc
#
# Key variables that may be set from Travis:
# - TRAVIS_RUST_VERSION: 1.1.0 ... stable/nightly/beta
# - TRAVIS_OS_NAME: linux/osx
# - DOCKER_IMAGE: posborne/rust-cross:arm
# - TARGET: e.g. arm-unknown-linux-gnueabihf

set -ex

BASE_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

if [ "$TRAVIS_OS_NAME" = "linux" ]; then
  OS=unknown-linux-gnu
elif [ "$TRAVIS_OS_NAME" = "osx" ]; then
  OS=apple-darwin
else
  echo "Unexpected TRAVIS_OS_NAME: $TRAVIS_OS_NAME"
  exit 1
fi

export HOST=$ARCH-$OS
if [ "$TARGET" = "" ]; then
  TARGET=$HOST
fi

if [ "$DOCKER_IMAGE" = "" ]; then
  export RUST_TEST_THREADS=1
  curl -sSL "https://raw.githubusercontent.com/carllerche/travis-rust-matrix/master/test" | bash
  cargo doc --no-deps
else
  export RUST_VERSION=${TRAVIS_RUST_VERSION}
  export RUST_TARGET=${TARGET}
  export DOCKER_IMAGE=${DOCKER_IMAGE}
  ${BASE_DIR}/ci/run-docker.sh
fi
