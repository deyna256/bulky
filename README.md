<h1 align="center">Bulky</h1>

<p align="center">Inspired by <a href="https://github.com/mokevnin/heavy">heavy</a> by <a href="https://github.com/mokevnin">@mokevnin</a>.</p>

<h3 align="center">Stop babysitting your builds.</h3>

<p align="center">Build in one terminal. Kick off tests in another.<br>
With <code>bulky --</code> on both commands, the tests start when the build finishes.</p>

<p align="center">
  <a href="https://crates.io/crates/bulky"><img src="https://img.shields.io/crates/v/bulky" alt="Bulky on crates.io"></a>
  <a href="https://github.com/deyna256/bulky/actions/workflows/ci.yml"><img src="https://github.com/deyna256/bulky/actions/workflows/ci.yml/badge.svg?branch=main" alt="CI status on main"></a>
</p>

<p align="center">
  <picture>
    <source media="(prefers-reduced-motion: reduce)" srcset="https://raw.githubusercontent.com/deyna256/bulky/main/docs/assets/demo-still.png">
    <img src="https://raw.githubusercontent.com/deyna256/bulky/main/docs/assets/demo.gif" width="800" alt="Two projects share one slot. While cargo build runs in ~/api, cargo test waits in ~/web. The tests start automatically when the build finishes.">
  </picture>
</p>

## Quick start

Install on Linux, macOS or Windows with `cargo install --locked bulky`.

```sh
bulky -- cargo build     # In one terminal
bulky -- cargo test      # In another project, while the build runs
```

By default, Bulky runs **one command at a time** across your projects on this
machine. The limit is per user. Commands keep their input, output and exit code.
Bulky needs no background service.

```sh
bulky --status           # See what's running
bulky --set-limit 2      # Run up to two commands at once
```

<details>
<summary>How slots work</summary>

A command uses one slot, even if it starts threads or calls Bulky again.
If Bulky has to wait, it prints the slot holder's PID and program name once.
Any waiting command can take the next free slot.

Bulky saves the limit (1–256) in `.bulky` in your home directory. Changing it
affects new and waiting commands. You can only remove slots that are free.

Bulky frees the slot when the command it started exits. Force-killing Bulky also
frees the slot, even if the command keeps running.

</details>

Linux Mint also ships a file renamer named `bulky`. If you have both installed,
use the full path to the one you want.

[Contributing](https://github.com/deyna256/bulky/blob/main/CONTRIBUTING.md) ·
[Security policy](SECURITY.md) · [Apache-2.0 license](LICENSE)
