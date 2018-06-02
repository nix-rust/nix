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
  - Add a new Unreleased section header to `CHANGELOG.md` with the four
    Added, Changed, Fixed, and Removed sections added.
  - In `Cargo.toml`, update the version to the new version.
  - In `Cargo.toml`, change the libc dependency to the latest version.
  - In `README.md`, update the version in the Usage section to the new
    version.
- Confirm that everything's ready for a release by running
  `cargo publish --dry-run`
- Make a pull request.
- Once the PR is merged, tag the merge commit, e.g. `git tag v0.8.3
  $MERGE_COMMIT_SHA1`.
- Push the tag, e.g. `git push origin v0.8.3`.

# Create Release

- Checkout the tag.
- Publish to crates.io with `cargo publish`.

# After Release

Once a release is done, all that's left is to change the `libc` version
back to using the version from git. So make a pull request containing a
simple commit entitled "Start the next dev cycle" that changes the `libc`
dependency in `Cargo.toml` to using it from git master.
