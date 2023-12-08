# Contributing to nix

We're really glad you're interested in contributing to nix! This
document has a few pointers and guidelines to help get you started.

To have a welcoming and inclusive project, nix uses the Rust project's
[Code of Conduct][conduct]. All contributors are expected to follow it.

[conduct]: https://www.rust-lang.org/conduct.html


# Issues

We use GitHub's [issue tracker][issues].

[issues]: https://github.com/nix-rust/nix/issues


## Bug reports

Before submitting a new bug report, please [search existing
issues][issue-search] to see if there's something related. If not, just
[open a new issue][new-issue]!

As a reminder, the more information you can give in your issue, the
easier it is to figure out how to fix it. For nix, this will likely
include the OS and version, and the architecture.

[issue-search]: https://github.com/nix-rust/nix/search?utf8=%E2%9C%93&q=is%3Aissue&type=Issues
[new-issue]: https://github.com/nix-rust/nix/issues/new


## Feature / API requests

If you'd like a new API or feature added, please [open a new
issue][new-issue] requesting it. As with reporting a bug, the more
information you can provide, the better.


## Labels

We use labels to help manage issues. The structure is modeled after
[Rust's issue labeling scheme][rust-labels]:
- **A-** prefixed labels state which area of the project the issue
  relates to
- **E-** prefixed labels explain the level of experience necessary to fix the
  issue
- **O-** prefixed labels specify the OS for issues that are OS-specific
- **R-** prefixed labels specify the architecture for issues that are
  architecture-specific

[rust-labels]: https://github.com/rust-lang/rust/blob/master/CONTRIBUTING.md#issue-triage


# Pull requests

GitHub pull requests are the primary mechanism we use to change nix. GitHub itself has
some [great documentation][pr-docs] on using the Pull Request feature. We use the 'fork and
pull' model described there.

Please make pull requests against the `master` branch.

If you change the API by way of adding, removing or changing something or if
you fix a bug, please add an appropriate note, every note should be a new markdown 
file under the [changelog directory][cl] stating the change made by your pull request, 
the filename should be in the following format:

```
<PULL_REQUEST_ID>.<TYPE>.md
```

These are 4 `TYPE`s available:

1. `added`
2. `changed`
3. `fixed`
4. `removed`

Let's say you have added a new API to nix, then a change log like this should
be added (assume it is PR #0)

```md
# file: 0.added.md
Added a new API xxx
```

And having multiple change logs for one PR is allowed.

[cl]: https://github.com/nix-rust/nix/tree/master/changelog
[pr-docs]: https://help.github.com/articles/using-pull-requests/

## Testing

nix has a test suite that you can run with `cargo test --all-features`. Ideally, we'd like pull
requests to include tests where they make sense. For example, when fixing a bug,
add a test that would have failed without the fix.

After you've made your change, make sure the tests pass in your development
environment. We also have continuous integration set up on [Cirrus-CI][cirrus-ci]
and GitHub Action, which might find some issues on other platforms. The CI will
run once you open a pull request.

[cirrus-ci]: https://cirrus-ci.com/github/nix-rust/nix

### Disabling a test in the CI environment

Sometimes there are features that cannot be tested in the CI environment. To
stop a test from running under CI, add `skip_if_cirrus!()` to it. Please
describe the reason it shouldn't run under CI, and a link to an issue if
possible! Other tests cannot be run under QEMU, which is used for some
architectures. To skip them, add a `#[cfg_attr(qemu, ignore)]` attribute to
the test.

## GitHub Merge Queues

We use GitHub merge queues to ensure that subtle merge conflicts won't result
in failing code. If you add or remove a CI job, remember to adjust the
required status checks in the repository's branch protection rules!

## API conventions

If you're adding a new API, we have a [document with
conventions][conventions] to use throughout the nix project.

[conventions]: https://github.com/nix-rust/nix/blob/master/CONVENTIONS.md
