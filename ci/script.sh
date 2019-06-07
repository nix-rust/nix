#!/bin/sh
# This script takes care of testing your crate

set -ex

. $HOME/.cargo/env

main() {
    # Add a cfg spec to allow disabling specific tests under CI.
    if [ "$TRAVIS" = true ]; then
        export RUSTFLAGS=--cfg=travis
    fi
    if [ "$CIRRUS_CI" = true ]; then
        export RUSTFLAGS=--cfg=cirrus
    fi

    for t in "$TARGET"; do
        # Build debug and release targets
        cross build --target $t
        cross build --target $t --release

        if [ ! -z $DISABLE_TESTS ]; then
            continue
        fi

        # Run tests on debug and release targets.
        cross test --target $t
        cross test --target $t --release
    done
}

# we don't run the "test phase" when doing deploys
if [ -z "$TRAVIS_TAG" -a -z "$CIRRUS_TAG" ]; then
    main
fi
