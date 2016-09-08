This document lists the steps that lead to a successful release of the Nix
library.

# Before Release

Based on changes since the last release, pick a new version number
following semver conventions. For nix, a change that drops support for
some Rust versions counts as a breaking change, and requires a major bump.

The release is prepared as follows:

- Ask for a new libc version if, necessary. It usually is.
- Make a commit with a message like "Release v0.8.3" with the following
  changes:
  - In `CHANGELOG.md`, rename the Unreleased section to the new version
    followed by the date of the release.
  - In `Cargo.toml`, update the version to the new version.
  - In `Cargo.toml`, change the libc dependency to the latest version.
  - In `README.md`, update the version in the Usage section to the new
    version.
- Make a pull request.
- Once the PR is merged, tag the merge commit, eg `git tag v0.8.3
  $MERGE_COMMIT_SHA1`.
- Push the tag, eg `git push v0.8.3`.

# Create Release

- Checkout the tag.
- Publish to crates.io with `cargo publish`.

# After Release

After the release a commit with the following changes is added to the master
branch.

- Add a new Unreleased section header to CHANGELOG.md.
- In `Cargo.toml`, update the version to the next `-dev` version, eg
  `v0.8.4-dev`.
- In `Cargo.tml`, revert the libc dependency to its git master branch.
- Commit with a message like "Bump to v0.8.4-dev"
- Make a pull request.
