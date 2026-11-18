# Milestones

## M0: Project foundations

- [ ] Restore a clean workspace build.
- [ ] Define `libcontainer` lifecycle interfaces.
- [ ] Split CLI argument parsing from runtime operations.
- [ ] Add integration-test conventions and a privileged-test marker.

**Exit criteria:** `cargo test --workspace` succeeds in the development
environment, and the project has no placeholder runtime entry points.


## M3: `create`

- [ ] Parse and validate `config.json` with `oci-spec`.
- [ ] Create `/run/runc-rs/<id>/`.
- [ ] Persist the container state and host PID.
- [ ] Create and configure a cgroup v2 directory.
- [ ] Apply `memory.max` and `pids.max`.
- [ ] Prepare the root filesystem and namespace setup.
- [ ] Report setup errors to the CLI before reporting success.

**Exit criteria:** `runc-rs create <id> --bundle <bundle>` leaves a valid
created container that can be inspected through runtime state.

## M4: `start`

- [ ] Read and validate the persisted container state.
- [ ] Open the start gate and allow the setup process to `execve`.
- [ ] Transition state from `created` to `running`.
- [ ] Return a clear error for invalid state transitions.

**Exit criteria:** A `/bin/sh` workload starts as PID 1 in the container.

## M5: `kill` and `delete`

- [ ] Implement signal delivery by host PID.
- [ ] Reject killing containers in incompatible states.
- [ ] Wait for or verify workload exit where required.
- [ ] Remove the cgroup and runtime directory.
- [ ] Clean state when repeated deletion is requested.

**Exit criteria:** `kill` and `delete` complete cleanly for the shell workload,
and teardown cannot remove a live container accidentally.

