# If no sub-command is given, simply list all the available options
_default:
    just --list

# Build the doc
doc *args='':
    RUSTDOCFLAGS='--cfg docsrs' cargo +nightly doc --all-features --no-deps {{args}}

