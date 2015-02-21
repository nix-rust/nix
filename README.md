# Rust bindings to *nix APIs

Rust friendly bindings to various *nix platform APIs (Linux, Darwin,
...). The goal is to not provide a 100% unified interface, but to unify
what can be while still providing platform specific APIs.

[![Build Status](https://travis-ci.org/carllerche/nix-rust.svg?branch=master)](https://travis-ci.org/carllerche/nix-rust)

## Usage

To use `nix`, first add this to your `Cargo.toml`:

```toml
[dependencies]
nix = "*"
```

Then, add this to your crate root:

```rust
extern crate nix;
```
