### OCI specs contract

![img.png](../assets/img.png)

## System boundaries

The runtime is organized as a strict dependency direction:

```text
runc-rs
  -> libcli
       -> libcontainer
            -> Linux APIs
```

- `runc-rs` wires the executable entry point and process-wide configuration.
- `libcli` owns argument parsing, command dispatch, and user-facing errors.
- `libcontainer` owns OCI lifecycle operations, runtime state, and container
  setup orchestration.
- Linux primitives are private modules inside `libcontainer` for the MVP.

This boundary keeps command handling independent of namespace, cgroup, mount,
and process APIs. A CLI command may call only the lifecycle API from
`libcontainer`; it must not call `nix` or manipulate `/run/runc-rs` directly.

## Container lifecycle API

The lifecycle API should expose operations rather than implementation details:

```text
create(bundle, id) -> RuntimeState
start(id) -> RuntimeState
kill(id, signal) -> RuntimeState
delete(id) -> ()
```

The MVP should keep these operations synchronous. Runtime state is persisted
under `/run/runc-rs/<id>/` and is the source of truth for command state
transitions.

### Detailed Architecture (sequence diagram)
```mermaid
sequenceDiagram
    autonumber
    actor User
    participant Main as Main Process (runc-rs CLI)
    participant FS as Host State & Cgroups (/run, /sys/fs/cgroup)
    participant Kernel as Linux Kernel
    participant Inter as Intermediate Process (Container PID 1)
    participant Target as Target Process (Workload /bin/sh)

    Note over User, Target: 1. CREATE PHASE (runc-rs create <id> --bundle <path>)
    User->>Main: runc-rs create my-box --bundle ./bundle
    Main->>Main: Parse & validate config.json
    Main->>FS: Create /run/runc-rs/my-box & mkfifo(exec.fifo)
    Main->>FS: Create /sys/fs/cgroup/my-box & apply limits (memory.max, pids.max)
    Main->>Kernel: Create anonymous pipes (sync_pipe, err_pipe with O_CLOEXEC)

    Main->>Kernel: unshare(CLONE_NEWPID | CLONE_NEWNS | CLONE_NEWUTS | CLONE_NEWIPC | CLONE_NEWNET)
    Main->>Kernel: fork()
    Kernel-->>Inter: Spawn Intermediate Process (Assigned PID 1 in new PID ns)

    par Host Setup (Main)
        Main->>FS: Write Inter's host PID to cgroup.procs
        Main->>Kernel: Close write-end of sync_pipe (signals Inter to proceed)
        Main->>Kernel: Block reading err_pipe (waits for Inter init status)
    and Container Environment Construction (Intermediate)
        Inter->>Kernel: Block reading sync_pipe (waits for cgroup attachment)
        Kernel-->>Inter: Unblocks on sync_pipe EOF
        Inter->>Kernel: mount("/", MS_REC | MS_PRIVATE)
        Inter->>Kernel: Bind mount rootfs & mount spec.mounts (/proc, /sys, /dev)
        Inter->>Kernel: pivot_root(rootfs, .old_root) & umount2(.old_root, MNT_DETACH)
        Inter->>Kernel: sethostname(spec.hostname)
        Inter->>Kernel: Drop capabilities, set NO_NEW_PRIVS, apply seccomp, setgid/setuid
        Inter->>Kernel: chdir(spec.process.cwd)
        Inter->>FS: read(pre-opened exec.fifo FD)
        Note over Inter, FS: Intermediate blocks here, err_pipe stays open until execve
    end

    Main->>FS: Write state.json (status: "created", pid: <host_pid>)
    Main-->>User: Exit CLI cleanly (returns shell prompt)

    Note over User, Target: 2. START PHASE (runc-rs start <id>)
    User->>Main: runc-rs start my-box
    Main->>FS: Read state.json (verify status == "created")
    Main->>FS: open(exec.fifo, O_WRONLY), write byte & close()
    FS-->>Inter: Unblocks read(exec.fifo)
    Main->>FS: Update state.json (status: "running")
    Main-->>User: Exit CLI cleanly

    Inter->>Kernel: execve(spec.process.args, spec.process.env)
    Note over Inter, Target: Kernel closes O_CLOEXEC FDs & overwrites Inter memory in-place
    Kernel-->>Target: Target Process starts executing as PID 1 inside isolated world

    Note over User, Target: 3. RUNTIME ENFORCEMENT & TEARDOWN (kill / delete)
    Target->>Kernel: Syscalls (open, fork, malloc, socket...)
    Kernel-->>Target: Enforces namespaces, rootfs jail, cgroups & seccomp automatically

    User->>Main: runc-rs kill my-box SIGTERM
    Main->>FS: Read pid from state.json
    Main->>Kernel: kill(<host_pid>, SIGTERM)
    Kernel-->>Target: Delivers SIGTERM to PID 1
    Target->>Kernel: exit(0)

    User->>Main: runc-rs delete my-box
    Main->>FS: Verify /proc/<host_pid> no longer exists
    Main->>FS: Remove /sys/fs/cgroup/my-box & /run/runc-rs/my-box
    Main-->>User: Exit CLI cleanly
```
### 3rd party dependencies

- [nix](https://crates.io/crates/nix) - for syscalls
- [serde](https://crates.io/crates/serde) - with `derive` feature for serializing and deserializing structs to/from json
- [serde_json](https://crates.io/crates/serde_json) - for serializing and deserializing structs to/from json
- [oci-spec](https://crates.io/crates/oci-spec) - for working with OCI specifications
- [anyhow](https://crates.io/crates/anyhow) - for error handling
- [clap](https://crates.io/crates/clap) - for parsing command line arguments

### codebase structure
we inspired from [youki](https://github.com/youki-dev/youki) codebase structure.

- libcontainer for runtime implementation
- libcli for cli operations


### process hierarchy
- **main process:**
  - provided command interface
  - parses config.json
  - set up cgroup directory
  - set up /run directory
  - creates IPS pipes
  - spawns the child (intermediate process
- **intermediate process:**
  - set up the rootfs
  - mounts /proc and /dev
  - set hostnames and other namespaces
  - once start this proccess disappear via execve 
- **child process:**
  - runs the containerized application 


