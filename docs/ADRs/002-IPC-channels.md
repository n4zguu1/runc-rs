# ADR 002: Inter-process communication and failure reporting

- Status: Draft
- Date: 2026-11-18
- Decision owners: n4zguu1

## Context

Container setup spans the CLI process, a setup process, and the workload
process. Setup failures must reach the parent before `create` reports success.
The interface must support process synchronization without becoming a general
message bus. also to handle the oci-specs [lifecycle](https://github.com/opencontainers/runtime-spec/blob/main/runtime.md#lifecycle) errors and propagate them to the parent process.

## Decision

No IPC mechanism is selected yet. The implementation should compare the
following patterns during the process creation spike:

- Anonymous pipes for setup acknowledgement and error delivery.
- A named FIFO for the explicit `start` gate.
- `pidfd` or polling for workload and teardown status.

## Consequences

The final decision will define file descriptor ownership, close timing,
failure serialization, and cleanup behavior. It should be recorded here once
the spike proves the synchronization protocol.