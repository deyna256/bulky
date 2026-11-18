<h1 align="center">Bulky</h1>

<p align="center">Inspired by <a href="https://github.com/mokevnin/heavy">heavy</a> by @mokevnin.</p>

<p align="center">
  <a href="https://crates.io/crates/bulky"><img src="https://img.shields.io/crates/v/bulky" alt="Bulky version on crates.io"></a>
  <a href="https://github.com/deyna256/bulky/actions/workflows/ci.yml"><img src="https://github.com/deyna256/bulky/actions/workflows/ci.yml/badge.svg?branch=main" alt="CI status on main"></a>
</p>

Bulky gives heavy commands one shared limit across your terminals and projects.
Wrap a build or test run with `bulky --` and it starts as soon as a slot is
free. The limit applies to commands you wrap under the same user account on
one machine. No daemon or per-project setup is needed.

## Install

On Linux, macOS or Windows, install with Cargo:

```sh
cargo install --locked bulky
```

To install from a local checkout, run `cargo install --locked --path .`.
Linux Mint also ships a file renamer called `bulky`. If you have both,
use the full path to the executable you want.

## Try it in two terminals

Start a build in one project:

```sh
bulky -- cargo build
```

While the build is running, start tests in another project:

```sh
bulky -- cargo test
```

The default is **one slot**. The tests wait until the build exits, then start
automatically. While waiting, Bulky prints the slot holder's PID and program
name to stderr once. If a slot is already free, Bulky starts quietly.
Your command keeps its terminal streams and exit code.

## Inspect or change the limit

| Command | What it does |
| --- | --- |
| `bulky --status` | Shows how many slots are busy and who holds them. |
| `bulky --set-limit 2` | Allows up to two wrapped workloads to run at once. |

Bulky saves the limit (1–256) in `.bulky` in your home directory. Changes
apply to new and waiting commands. If lowering the limit would remove a busy
slot, Bulky asks you to retry after its command finishes.

## What a slot covers

- One slot covers one wrapped workload, however many threads it starts.
  Nested `bulky` calls reuse the outer slot.
- Bulky holds the slot until its direct child exits. Force-killing Bulky
  releases the slot even if the child keeps running.
- When several commands are waiting, any one of them can take the next free slot.

[Contributing](https://github.com/deyna256/bulky/blob/main/CONTRIBUTING.md) ·
[Security policy](SECURITY.md) · [Apache-2.0 license](LICENSE)
