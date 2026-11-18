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

Branch from `main`. For issue #42, name the branch `42` and use a one-line
commit such as `#42: preserve the child exit code`. For work without an issue,
use a descriptive branch and omit the number from the commit. Check the
staged diff. Keep explanations in the PR and omit commit trailers.

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
