# Contributing to Bulky

Use plain English. Follow the [Code of Conduct](CODE_OF_CONDUCT.md) and
omit generated-by signatures and tool attribution. Report vulnerabilities
privately through [SECURITY.md](SECURITY.md).

## Issues, branches and commits

Search existing issues first. Give each [issue](.github/ISSUE_TEMPLATE/issue.md)
one outcome and explain how to verify it. Assign one type from `bug`,
`feature`, `maintenance`, `docs` or `question`. Add `ci` for tooling and
`good first issue` for small tasks suitable for newcomers. Link duplicates
to the original issue. Close declined work as **not planned**. Maintainers
can create or refresh the labels with the **Set up issue labels** workflow.

Branch from `main`. For issue #42, name the branch `42`. For work without an
issue, use a descriptive branch name.

Follow [Conventional Commits](https://www.conventionalcommits.org/en/v1.0.0/)
for commits and PR titles. Write one line in English. Start with a type,
then describe the change with an imperative verb. Append `(#42)` when the
change belongs to issue #42.

- `fix(cli): preserve the child exit code (#42)`
- `docs: explain the shared limit`
- `chore(release): bump bulky to 0.2.0`

Use `feat` for features and `fix` for bug fixes. Other common types include
`docs`, `chore`, `ci`, `build`, `test`, `perf` and `refactor`. The scope is
optional. For a breaking change, put `!` before the colon and describe what
breaks. Check the staged diff. Keep explanations in the PR and omit commit
trailers. Use the same format for the final squash commit.

## Pull requests and reviews

Open a draft [PR](.github/pull_request_template.md) against `main`. Use
`Closes #42` when it completes the issue. Update affected docs in the same PR.
When ready, request review from @deyna256 or another maintainer. Request
another look after addressing feedback.

Put line-specific findings inline. Give each blocker a consequence and a fix
or check. Mark optional feedback with `Nit:` or `Question:`. Use **Summary**,
**Blocking**, **Non-blocking** and **Checked** as needed in the review.
Resolve blockers and check CI before merging. File unrelated work separately.

## Rust

- Name functions for what they do. Keep modules private until a caller needs
  an interface. Use the loop or iterator that makes the order of work clear.
- Borrow inputs. Give each file lock or child process an owner whose lifetime
  matches the operation. Keep shared locks out of sleeps and child waits.
- Keep paths and command arguments in OS-native types. Return recoverable
  failures with `Result` and `?`. Include the failed operation in diagnostics
  and keep private arguments out of them.
- Use safe standard library operations when they fit. Isolate platform code.
  Explain the safety conditions of each unsafe block. Elsewhere, add a comment
  only when a choice or constraint is hard to see from the code.
- Measure a release build before optimizing. For Bulky, measure immediate
  launches separately from waiting under contention. Use the same workload
  before and after a change and report the timings in the PR.

References: [Rust Book on errors](https://doc.rust-lang.org/book/ch09-02-recoverable-errors-with-result.html),
[loops and iterators](https://doc.rust-lang.org/book/ch13-04-performance.html),
[Rust Performance Book on benchmarking](https://nnethercote.github.io/perf-book/benchmarking.html)
and [profiling](https://nnethercote.github.io/perf-book/profiling.html).

## Local checks

Install rustup and [just](https://just.systems/). Run `just check` from the
checkout to check formatting, run Clippy and run the tests. Rustup supplies
the pinned Rust tools. Use `just format` to apply formatting, `just build` to
build the CLI and `just package` from a clean checkout to verify the crate.
Run `just --list` for individual commands.

Start CLI changes with a real-process test in `tests/cli.rs`. Use a private
home per test, synchronize on events, and give waits a deadline. CI runs Clippy and
the tests on Linux, macOS and Windows. For docs-only changes, check text and links.

Review the `Cargo.lock` diff after `just update`. Cargo and Dependabot apply
a three-day cooldown to new dependency versions. Dependabot checks Cargo
and GitHub Actions monthly.

Keep working notes in `.local/` and agreed decisions in the
[product direction](docs/product-direction.md).

## Publishing a release

1. Set the version in `Cargo.toml`, refresh `Cargo.lock` with `cargo check`, and
   put both changes on `main`.
2. Open [Actions → Release](https://github.com/deyna256/bulky/actions/workflows/release.yml),
   select **Run workflow** and choose `main`. The version comes from Cargo.
   Select **Run release checks without publishing** for a preview.
3. After CI passes, the workflow tags the tested commit as `v<version>`,
   publishes the crate and creates the GitHub Release with generated notes.

If a step fails, select **Re-run failed jobs** on that run. It keeps the
original commit and reuses its tag. If crates.io already has that version,
the workflow continues to the GitHub Release.

The crates.io trusted publisher uses repository `deyna256/bulky`, workflow
`release.yml` and environment `crates-io`.

[crates.io Trusted Publishing](https://crates.io/docs/trusted-publishing)
