use crate::pool::{Admission, HELD};
use std::ffi::{OsStr, OsString};
use std::io;
use std::process::{Command, ExitStatus};

pub fn run(program: &OsStr, args: impl Iterator<Item = OsString>, admission: &Admission) -> i32 {
    let mut cmd = Command::new(program);
    cmd.args(args).env(HELD, &admission.token);
    match wait_for_command(cmd) {
        Ok(status) => exit_code(status),
        Err(error) => {
            eprintln!("bulky: {program:?}: {error}");
            match error.kind() {
                io::ErrorKind::NotFound => 127,
                io::ErrorKind::PermissionDenied => 126,
                _ => 1,
            }
        }
    }
}

#[cfg(unix)]
fn wait_for_command(mut cmd: Command) -> io::Result<ExitStatus> {
    use nix::sys::signal::{SaFlags, SigAction, SigHandler, SigSet, Signal, kill, sigaction};
    use nix::unistd::Pid;
    use std::os::unix::process::CommandExt;

    let mut signals = SigSet::empty();
    for signal in [
        Signal::SIGINT,
        Signal::SIGTERM,
        Signal::SIGHUP,
        Signal::SIGCHLD,
    ] {
        signals.add(signal);
    }
    let original = signals.thread_swap_mask(nix::sys::signal::SigmaskHow::SIG_BLOCK)?;
    // Darwin discards ignored SIGCHLD events before sigwait can observe them.
    extern "C" fn child_changed(_: i32) {}
    let action = SigAction::new(
        SigHandler::Handler(child_changed),
        SaFlags::SA_NOCLDSTOP,
        SigSet::empty(),
    );
    // SAFETY: the handler is empty. The child only restores signal state before exec.
    unsafe {
        let old_action = sigaction(Signal::SIGCHLD, &action)?;
        cmd.pre_exec(move || {
            sigaction(Signal::SIGCHLD, &old_action)?;
            Ok(original.thread_set_mask()?)
        });
    }
    let mut child = cmd.spawn()?;
    let pid = Pid::from_raw(child.id() as i32);
    loop {
        if let Some(status) = child.try_wait()? {
            return Ok(status);
        }
        let signal = signals.wait()?;
        if signal != Signal::SIGCHLD {
            kill(pid, signal)?;
        }
    }
}

#[cfg(unix)]
fn exit_code(status: ExitStatus) -> i32 {
    use std::os::unix::process::ExitStatusExt;
    status
        .code()
        .unwrap_or_else(|| 128 + status.signal().unwrap_or(0))
}

#[cfg(windows)]
fn wait_for_command(mut cmd: Command) -> io::Result<ExitStatus> {
    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn SetConsoleCtrlHandler(handler: extern "system" fn(u32) -> i32, add: i32) -> i32;
    }
    extern "system" fn console_interrupt(event: u32) -> i32 {
        // The foreground child receives Ctrl+C and Ctrl+Break from the console directly.
        i32::from(matches!(event, 0 | 1))
    }
    // SAFETY: this callback has the Windows handler ABI and lives for the process lifetime.
    if unsafe { SetConsoleCtrlHandler(console_interrupt, 1) } == 0 {
        let error = io::Error::last_os_error();
        // Detached processes have no console handler to install.
        if error.raw_os_error() != Some(6) {
            return Err(error);
        }
    }
    cmd.status()
}

#[cfg(windows)]
fn exit_code(status: ExitStatus) -> i32 {
    status.code().unwrap_or(1)
}
