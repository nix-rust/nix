set -ex

main() {
    curl https://sh.rustup.rs -sSf | \
        sh -s -- -y --default-toolchain $TRAVIS_RUST_VERSION

    local target=
    if [ $TRAVIS_OS_NAME = linux ]; then
        target=x86_64-unknown-linux-gnu
    else
        target=x86_64-apple-darwin
    fi

    # TODO At some point you'll probably want to use a newer release of `cross`,
    # simply change the argument to `--tag`.
    curl -LSfs https://japaric.github.io/trust/install.sh | \
        sh -s -- \
           --force \
           --git japaric/cross \
           --tag v0.1.4 \
           --target $target
}

main
