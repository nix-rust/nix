# This script takes care of testing your crate

set -ex

main() {
    # Add a cfg spec to allow disabling specific tests under CI.
    if [ "$TRAVIS" = true ]; then
        export RUSTFLAGS=--cfg=travis
    fi

    # Build debug and release targets
    cross build --target $TARGET
    cross build --target $TARGET --release

    if [ ! -z $DISABLE_TESTS ]; then
        return
    fi

    # Run tests on debug and release targets.
    cross test --target $TARGET
    cross test --target $TARGET --release
}

# we don't run the "test phase" when doing deploys
if [ -z $TRAVIS_TAG ]; then
    main
fi
