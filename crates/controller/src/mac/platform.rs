use std::ffi::c_void;
use std::io;
use std::os::fd::AsRawFd;
use std::os::unix::process::CommandExt;
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::sync::Mutex;
use std::time::{Duration, Instant};

pub type WorkerStream = std::os::unix::net::UnixStream;

pub struct WorkerProcess {
    child: Mutex<Option<Child>>,
    pid: libc::pid_t,
}

impl WorkerProcess {
    pub fn running(&self) -> bool {
        self.child
            .lock()
            .unwrap()
            .as_mut()
            .is_some_and(|child| child.try_wait().ok().flatten().is_none())
    }

    pub fn physical_memory_bytes(&self) -> u64 {
        process_group_physical_memory(self.pid)
    }

    pub fn terminate(&self) {
        let mut guard = self.child.lock().unwrap();
        let Some(mut child) = guard.take() else {
            return;
        };
        unsafe { libc::kill(-self.pid, libc::SIGTERM) };
        let deadline = Instant::now() + Duration::from_secs(2);
        while Instant::now() < deadline {
            if child.try_wait().ok().flatten().is_some() {
                // The leader can exit before its browser subprocesses. Reap the entire group.
                unsafe { libc::kill(-self.pid, libc::SIGKILL) };
                return;
            }
            std::thread::sleep(Duration::from_millis(100));
        }
        unsafe { libc::kill(-self.pid, libc::SIGKILL) };
        let _ = child.wait();
    }
}

impl Drop for WorkerProcess {
    fn drop(&mut self) {
        self.terminate();
    }
}

pub fn spawn_framed_worker(
    executable: &str,
    arguments: &[String],
) -> io::Result<(WorkerProcess, WorkerStream)> {
    let (controller, child_socket) = std::os::unix::net::UnixStream::pair()?;
    let descriptor = child_socket.as_raw_fd();
    let resolved = if executable.ends_with(".app") {
        Path::new(executable).join("Contents/MacOS/WebKitAutomationWorker")
    } else {
        Path::new(executable).to_owned()
    };
    let mut command = Command::new(resolved);
    command
        .args(arguments)
        .arg(format!("--controller-socket-fd={descriptor}"))
        .stdout(Stdio::null())
        .stderr(Stdio::inherit())
        .process_group(0);
    // Only the child clears CLOEXEC; concurrent launches cannot inherit this socket.
    unsafe {
        command.pre_exec(move || {
            if libc::fcntl(descriptor, libc::F_SETFD, 0) < 0 {
                return Err(io::Error::last_os_error());
            }
            // Worker stdout is diagnostic output, including when the CLI emits binary data.
            if libc::dup2(libc::STDERR_FILENO, libc::STDOUT_FILENO) < 0 {
                return Err(io::Error::last_os_error());
            }
            Ok(())
        });
    }
    let child = command.spawn()?;
    let pid = child.id() as libc::pid_t;
    drop(child_socket);
    Ok((
        WorkerProcess {
            child: Mutex::new(Some(child)),
            pid,
        },
        controller,
    ))
}

pub fn spawn_process(executable: &str, arguments: &[String]) -> io::Result<WorkerProcess> {
    let mut command = Command::new(executable);
    command
        .args(arguments)
        .stdout(Stdio::null())
        .stderr(Stdio::inherit())
        .process_group(0);
    let child = command.spawn()?;
    let pid = child.id() as libc::pid_t;
    Ok(WorkerProcess {
        child: Mutex::new(Some(child)),
        pid,
    })
}

pub fn physical_cpu_count() -> usize {
    let mut count: libc::c_int = 0;
    let mut size = std::mem::size_of_val(&count);
    let name = c"hw.physicalcpu";
    let result = unsafe {
        libc::sysctlbyname(
            name.as_ptr(),
            (&mut count as *mut libc::c_int).cast(),
            &mut size,
            std::ptr::null_mut(),
            0,
        )
    };
    if result == 0 && count > 0 {
        count as usize
    } else {
        1
    }
}

pub fn random_token() -> String {
    let mut bytes = [0_u8; 16];
    getrandom::fill(&mut bytes).expect("secure randomness unavailable");
    let mut result = String::with_capacity(32);
    for byte in bytes {
        use std::fmt::Write;
        let _ = write!(result, "{byte:02x}");
    }
    result
}

#[repr(C)]
#[derive(Default)]
struct RUsageInfoV4 {
    uuid: [u8; 16],
    user_time: u64,
    system_time: u64,
    pkg_idle_wkups: u64,
    interrupt_wkups: u64,
    pageins: u64,
    wired_size: u64,
    resident_size: u64,
    phys_footprint: u64,
    proc_start_abstime: u64,
    proc_exit_abstime: u64,
    child_user_time: u64,
    child_system_time: u64,
    child_pkg_idle_wkups: u64,
    child_interrupt_wkups: u64,
    child_pageins: u64,
    child_elapsed_abstime: u64,
    diskio_bytesread: u64,
    diskio_byteswritten: u64,
    cpu_time_qos_default: u64,
    cpu_time_qos_maintenance: u64,
    cpu_time_qos_background: u64,
    cpu_time_qos_utility: u64,
    cpu_time_qos_legacy: u64,
    cpu_time_qos_user_initiated: u64,
    cpu_time_qos_user_interactive: u64,
    billed_system_time: u64,
    serviced_system_time: u64,
}

unsafe extern "C" {
    fn proc_listpids(
        kind: u32,
        typeinfo: u32,
        buffer: *mut c_void,
        buffersize: libc::c_int,
    ) -> libc::c_int;
    fn proc_pid_rusage(pid: libc::c_int, flavor: libc::c_int, buffer: *mut c_void) -> libc::c_int;
}

fn process_group_physical_memory(group: libc::pid_t) -> u64 {
    const PROC_ALL_PIDS: u32 = 1;
    const RUSAGE_INFO_V4: libc::c_int = 4;
    let bytes = unsafe { proc_listpids(PROC_ALL_PIDS, 0, std::ptr::null_mut(), 0) };
    if bytes <= 0 {
        return 0;
    }
    let mut pids = vec![0_i32; bytes as usize / std::mem::size_of::<i32>()];
    let written = unsafe { proc_listpids(PROC_ALL_PIDS, 0, pids.as_mut_ptr().cast(), bytes) };
    let count = (written.max(0) as usize / std::mem::size_of::<i32>()).min(pids.len());
    pids[..count]
        .iter()
        .copied()
        .filter(|pid| *pid > 0 && unsafe { libc::getpgid(*pid) } == group)
        .map(|pid| {
            let mut usage = RUsageInfoV4::default();
            if unsafe {
                proc_pid_rusage(
                    pid,
                    RUSAGE_INFO_V4,
                    (&mut usage as *mut RUsageInfoV4).cast(),
                )
            } == 0
            {
                usage.phys_footprint
            } else {
                0
            }
        })
        .sum()
}
