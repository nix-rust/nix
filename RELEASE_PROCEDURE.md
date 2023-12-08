This document lists the steps that lead to a successful release of the Nix
library.

# Before Release

Nix uses [cargo release](https://github.com/crate-ci/cargo-release) to automate
the release process. Based on changes since the last release, pick a new
version number following semver conventions. For nix, a change that drops
support for some Rust versions counts as a breaking change, and requires a
major bump.

The release is prepared as follows:

- Ask for a new libc version if, necessary. It usually is. Then update the
  dependency in `Cargo.toml` to rely on a release from crates.io.
 
  ```diff
  [dependencies]
  -libc = { git = "https://github.com/rust-lang/libc", rev = "<Revision>", features = ["extra_traits"] }
  +libc = { version = "<New Version>", features = ["extra_traits"] }
  ```
  
- Update the version number in `Cargo.toml`
- Generate `CHANGELOG.md` for this release by 
  `towncrier build --version=<VERSION> --yes`
- Confirm that everything's ready for a release by running
  `cargo release <patch|minor|major>`
- Create the release with `cargo release -x <patch|minor|major>`
- Push the created tag to GitHub.
