# Product direction

Bulky coordinates heavy commands across terminals, projects and agents for
one OS user on a personal machine. A slot admits one workload. CPU, memory
and the workload's internal threads remain under the command's control.
Coordination covers commands that use Bulky and share the user's state
directory. Separate machines, accounts and filesystem namespaces are outside
this scope.

## Current behavior

The [README](../README.md) describes the command interface, shared limit,
nesting and lifetime of a slot. Filesystem locks enforce the limit. Bulky
checks for a live outer slot before accepting a nested call. Pool errors
stop the command from starting. CI uses the same pool rules as local runs.

## Open questions

- Migration from `heavy`, including its crate API, `HEAVY_*` variables,
  named pools and `--slots`.
- Whether workloads need FIFO ordering, timeouts or a longer-lived slot
  when a wrapper is killed or its child starts background work.
- Whether users need named pools and how these would share the total limit.
