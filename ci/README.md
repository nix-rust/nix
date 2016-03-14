Test Infrastructure
===================

The ci directory contains scripts that aid in the testing of nix both
in our continuous integration environment (Travis CI) but also for
developers working locally.

Nix interfaces very directly with the underlying platform (usually via
libc) and changes need to be tested on a large number of platforms to
avoid problems.

Running Tests For Host Architecture
-----------------------------------

Running the tests for one's host architecture can be done by simply
doing the following:

    $ cargo test

Running Tests Against All Architectures/Versions
------------------------------------------------

Testing for other architectures is more involved.  Currently,
developers may run tests against several architectures and versions of
rust by running the `ci/run-all.sh` script.  This scripts requires
that docker be set up.  This will take some time:

    $ ci/run-all.sh

The list of versions and architectures tested by this can be
determined by looking at the contents of the script.  The docker image
used is [posborne/rust-cross][posborne/rust-cross].

[posborne/rust-cross]: https://github.com/rust-embedded/docker-rust-cross

Running Test for Specific Architectures/Versions
------------------------------------------------

Suppose we have a failing test with Rust 1.7 on the raspberry pi.  In
that case, we can run the following:

    $ DOCKER_IMAGE=posborne/rust-cross:arm \
          RUST_VERSION=1.7.0 \
          RUST_TARGET=arm-unknown-linux-gnueabihf ci/run-docker.sh

Currently, the docker images only support Rust 1.7.  To get a better
idea of combinations that might work, look at the contents of the
[travis configuration](../.travis.yml) or [run-all.sh](run-all.sh).
