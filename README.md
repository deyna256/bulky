<h1 align="center">Bulky</h1>

<p align="center">Inspired by <a href="https://github.com/mokevnin/heavy">heavy</a> by <a href="https://github.com/mokevnin">@mokevnin</a>.</p>

<h3 align="center">Already building in another terminal?</h3>

<p align="center">Start your tests now. Bulky waits for the build to finish, then runs them for you.<br>
Wrap heavy commands with <code>bulky --</code> to share one limit across your projects. No daemon required.</p>

<p align="center">
  <a href="https://crates.io/crates/bulky"><img src="https://img.shields.io/crates/v/bulky" alt="Bulky on crates.io"></a>
  <a href="https://github.com/deyna256/bulky/actions/workflows/ci.yml"><img src="https://github.com/deyna256/bulky/actions/workflows/ci.yml/badge.svg?branch=main" alt="CI status on main"></a>
</p>

<p align="center">
  <img src="https://raw.githubusercontent.com/deyna256/bulky/main/docs/assets/demo.gif" width="800" alt="A build runs in ~/api while tests wait in ~/web. When the build finishes, the tests start automatically and pass.">
</p>

## Quick start

Install on Linux, macOS or Windows with `cargo install --locked bulky`.

```sh
bulky -- cargo build     # In one terminal
bulky -- cargo test      # In another project, while the build runs
```

By default, **one wrapped command runs at a time** for your user account on this
machine. Each command keeps its input, output and exit code.

```sh
bulky --status           # See which command holds each slot
bulky --set-limit 2      # Allow two workloads at once
```

<details>
<summary>How slots work</summary>

A slot covers one workload, including its threads and nested Bulky calls.
Bulky prints the holder's PID and program name once when it has to wait.
Any waiting command can take the next free slot.

The limit (1–256) is saved in `.bulky` in your home directory. Changes apply to
new and waiting commands. Bulky refuses to remove a busy slot.

The slot is released when the direct child exits. Force-killing Bulky also
releases it, even if the child keeps running.

</details>

Linux Mint also ships a file renamer named `bulky`. If you have both installed,
use the full path to the one you want.

[Contributing](https://github.com/deyna256/bulky/blob/main/CONTRIBUTING.md) ·
[Security policy](SECURITY.md) · [Apache-2.0 license](LICENSE)
