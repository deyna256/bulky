# Bulky

Bulky limits how many heavy commands one user runs at once across projects
on the same machine. It needs no background service.

```sh
bulky -- cargo build     # first terminal
bulky -- cargo test      # another project, in a second terminal
```

The default is one slot. If the build is still running, the tests wait.
Bulky prints the holder's PID and program name to stderr once when it waits.
An immediate launch is quiet. The command keeps its terminal streams and exit
code.

```sh
bulky --status
bulky --set-limit 2
```

The limit (1–256) is saved in `.bulky` in your home directory. A change takes
effect for new and waiting commands. A reduction that would remove a busy slot
fails. Nested calls reuse their active outer slot, including in CI. One slot
counts one workload regardless of how many threads it starts.

Bulky holds its slot until the direct child exits. Force-killing the wrapper
releases its slot even if the child survives. Waiting order is unspecified.

## Build from source

```sh
cargo install --locked --path .
```

Linux Mint also provides a file renamer named `bulky`. If both are installed,
use the full path to select the intended program.

See `CONTRIBUTING.md` in the checkout for development and review. Send
vulnerability reports through the [security policy](SECURITY.md).

Licensed under [Apache-2.0](LICENSE).
