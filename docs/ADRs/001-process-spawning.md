# ADR 001: Container process creation model

- Status: Accepted
- Date: 2026-11-24
- Decision owners: n4zguu1

## Context

`runc-rs` must create an OCI container process, configure namespaces and a
cgroup, prepare the root filesystem, and replace itself with the workload
process.

The MVP needs a model that is understandable, testable on Linux, and sufficient
for the four core commands. A spike validated `clone3(2)` on the target Linux
environment, confirming atomic namespace creation, direct cgroup v2 placement,
and race-free parent-child lifecycle management via a process file descriptor.

## Decision

`runc-rs` will use `clone3(2)` with `CLONE_PIDFD` and `CLONE_INTO_CGROUP` as
its container process creation mechanism.

- **Namespace & PID 1 setup:** `clone3(2)` creates the child directly inside the
  requested namespaces (including `CLONE_NEWPID`), avoiding a double-fork sequence
  and placing the child immediately as PID 1 in its new PID namespace.
- **Cgroup assignment:** Passing `CLONE_INTO_CGROUP` with an open cgroup v2
  directory file descriptor places the child into the target cgroup atomically
  at process creation before any user-space instructions execute.
- **Process tracking & failure handling:** `CLONE_PIDFD` provides a race-free
  `pidfd` for monitoring child exit (`poll`/`waitid`) and signaling
  (`pidfd_send_signal`), while a synchronization pipe coordinates rootfs setup
  and propagates child setup errors back to the parent for deterministic cleanup.

## Consequences

- **Positive:**
  - Eliminates intermediate grandchild processes for `CLONE_NEWPID` and avoids
    post-fork cgroup migration races.
  - Provides race-free process lifecycle tracking and signaling via `pidfd`
    without PID reuse hazards.
  - Keeps process creation self-contained in a single structured `clone_args`
    syscall without manual child stack allocation.
- **Negative / Trade-offs:**
  - Requires a modern Linux kernel baseline (Linux 5.7+ for `CLONE_INTO_CGROUP`
    and cgroup v2), dropping support for legacy kernels.
  - Requires direct `syscall(SYS_clone3, ...)` bindings and careful alignment of
    `struct clone_args` in Rust, as `libc` exposure varies across targets.
  - Code executed in the child between `clone3(2)` and `execve(2)` must remain
    async-signal-safe and close all non-essential `O_CLOEXEC` descriptors.

## Alternatives

| Alternative | Rejection reason |
|---|---|
| `fork(2)` + `unshare(2)` / `setns(2)` | Requires a second `fork(2)` after `unshare(CLONE_NEWPID)` to establish PID 1 and introduces a window where the child runs outside the target cgroup before migration. |
| `clone(2)` | Flag space is exhausted for newer namespace flags, lacks `CLONE_INTO_CGROUP`, and requires managing an explicit child stack pointer. |