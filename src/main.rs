use nix::mount::{mount, umount2, MntFlags, MsFlags};
use nix::sched::{unshare, CloneFlags};
use nix::sys::stat::Mode;
use nix::sys::wait::waitpid;
use nix::unistd::{chdir, execvp, fork, mkdir, pivot_root, sethostname, ForkResult};
use std::ffi::CString;
use std::fs::remove_dir;
use std::path::PathBuf;

fn main() {
    let rootfs = PathBuf::from("/mnt/workspace/test_ground/contianer-bundle_v0/rootfs");

    unshare(CloneFlags::CLONE_NEWUTS | CloneFlags::CLONE_NEWPID | CloneFlags::CLONE_NEWNS)
        .expect("failed to unshare");

    match unsafe { fork() }.expect("failed to fork") {
        ForkResult::Parent { child } => {
            waitpid(child, None).expect("waitpid failed");
        }
        ForkResult::Child => {
            sethostname("runc-rs-box").expect("sethostname failed");

            // 1. Make mount propagation private
            mount(
                None::<&str>,
                "/",
                None::<&str>,
                MsFlags::MS_REC | MsFlags::MS_PRIVATE,
                None::<&str>,
            )
                .expect("make / private failed");

            // 2. Bind-mount rootfs onto itself
            mount(
                Some(&rootfs),
                &rootfs,
                None::<&str>,
                MsFlags::MS_BIND | MsFlags::MS_REC,
                None::<&str>,
            )
                .expect("bind mount rootfs failed");

            // 3. Pivot root into the bundle's rootfs
            let old_root = rootfs.join(".old_root");
            let _ = mkdir(&old_root, Mode::S_IRWXU);

            pivot_root(&rootfs, &old_root).expect("pivot_root failed");
            chdir("/").expect("chdir failed");

            // 4. Unmount and delete old host root
            umount2("/.old_root", MntFlags::MNT_DETACH).expect("umount2 failed");
            remove_dir("/.old_root").expect("remove_dir failed");

            // 5. Mount /proc
            mount(
                Some("proc"),
                "/proc",
                Some("proc"),
                MsFlags::empty(),
                None::<&str>,
            )
                .expect("mount /proc failed");

            // 6. Execute shell
            let cmd = CString::new("/bin/sh").unwrap();
            execvp(&cmd, &[&cmd]).expect("execvp failed");
        }
    }
}