# ADR 001: Container process creation model

- Status: Proposed
- Date: 2026-11-18
- Decision owners: Project maintainer

## Context

`runc-rs` must create an OCI container process, configure namespaces and a
cgroup, prepare the root filesystem, and replace itself with the workload
process.

The MVP needs a model that is understandable, testable on Linux, and sufficient
for the four core commands. It must not unnecessarily couple architecture
documentation to a specific syscall API before implementation evidence exists.

## Decision

No process creation mechanism is selected yet.

The process is to be validated by a spike before an accepted decision is made.
Candidates are:

- `fork(2)` plus `unshare(2)` and `setns(2)` as needed
- `clone(2)` with namespace flags
- `clone3(2)` with a `pidfd`

## Spike exit criteria

The spike must demonstrate:

- The parent can detect setup failures from the child.
- The child is assigned to the requested cgroup before workload execution.
- Namespace setup is reproducible in a rootful test environment.
- The workload has the expected PID namespace view.
- A failure can leave neither an orphaned workload nor stale runtime state.

## Consequences

Deferring the decision keeps the architecture reviewable but prevents finalizing
the process lifecycle pseudocode. The first implementation milestone must
include the spike and a small Linux integration test.

## Alternatives

| Alternative | Rejection reason for now |
|---|---|
| `fork` | Familiar and portable across older kernels, but needs a spike to prove the synchronization design. |
| `clone` | Direct namespace creation is concise, but flag and stack handling add complexity. |
| `clone3` | Useful for modern kernel features, but not a safe MVP dependency without checking kernel support. |
