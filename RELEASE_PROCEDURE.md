This document lists the steps that lead to a successful release of the Nix
library.

# Before Release

Nix uses [cargo release](https://github.com/crate-ci/cargo-release) to automate
the release process. Based on changes since the last release, pick a new
version number following semver conventions. For Nix, a change that drops
support for some Rust versions counts as a breaking change, and requires a
major bump.

The release is prepared as follows:

> NOTE: the following procedure should be done directly against the master 
> branch of the repo.

- Clone the `nix-rust/nix` repository with your preferred way, and `cd` to it:

  ```sh
  $ git clone https://github.com/nix-rust/nix.git
  $ cd nix
  ```

- If we are using `libc` from git, replace it with a usable release from crates.io.
 
  ```diff
  [dependencies]
  -libc = { git = "https://github.com/rust-lang/libc", rev = "<Revision>", features = ["extra_traits"] }
  +libc = { version = "<Version>", features = ["extra_traits"] }
  ```
  
- Update the version number in `Cargo.toml`
- Generate `CHANGELOG.md` for this release by 

  ```sh
  $ towncrier build --version=<VERSION> --yes
  Loading template...
  Finding news fragments...
  Rendering news fragments...
  Writing to newsfile...
  Staging newsfile...
  Removing the following files:
  nix/changelog/xxxx.xxxx.md
  nix/changelog/xxxx.xxxx.md
  ...
  nix/changelog/xxxx.xxxx.md
  Removing news fragments...
  Done!
  ``` 

- Push the changes made by the above steps to the master branch

- Ensure you have a crates.io token 
  1. With the `publish-update` scope
  2. Can be used for crate `nix`
  3. It is set via `cargo login`

  If not, create a new token [here](https://crates.io/settings/tokens), and set
  it. 

- Confirm that everything's ready for a release by running
  `cargo release <VERSION>`
- Create the release with `cargo release -x <VERSION>`, this step will publish
  the version to crates.io and push the new version tag to GitHub.

- Congratulations on a new Nix release!
