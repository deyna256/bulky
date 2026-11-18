use std::io::{BufRead, BufReader, Write};
use std::path::Path;
use std::process::{Child, Command, Output, Stdio};
use std::sync::mpsc;
use std::time::Duration;
use tempfile::tempdir;

const BIN: &str = env!("CARGO_BIN_EXE_bulky");
const TIMEOUT: Duration = Duration::from_secs(10);

fn bulky(home: &Path) -> Command {
    let mut cmd = Command::new(BIN);
    cmd.env("HOME", home)
        .env("USERPROFILE", home)
        .env_remove("BULKY_HELD");
    cmd
}

fn success(out: Output) -> String {
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8(out.stdout).unwrap()
}

fn lines(stream: impl std::io::Read + Send + 'static) -> mpsc::Receiver<String> {
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        for line in BufReader::new(stream).lines().map_while(Result::ok) {
            if tx.send(line).is_err() {
                break;
            }
        }
    });
    rx
}

// The test executable doubles as a portable child that waits for stdin.
#[test]
#[ignore = "launched by workload()"]
fn held_command() {
    if std::env::var_os("BULKY_TEST_CHILD").is_none() {
        return;
    }
    println!("READY");
    std::io::stdout().flush().unwrap();
    let mut line = String::new();
    std::io::stdin().read_line(&mut line).unwrap();
    std::process::exit(37);
}

fn workload(home: &Path, cwd: &Path) -> Child {
    bulky(home)
        .current_dir(cwd)
        .arg("--")
        .arg(std::env::current_exe().unwrap())
        .args(["--ignored", "--exact", "held_command", "--nocapture"])
        .env("BULKY_TEST_CHILD", "1")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap()
}

fn ready(lines: &mpsc::Receiver<String>) {
    while !lines.recv_timeout(TIMEOUT).unwrap().contains("READY") {}
}

fn finish(mut child: Child) {
    drop(child.stdin.take());
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || tx.send(child.wait()).unwrap());
    assert_eq!(rx.recv_timeout(TIMEOUT).unwrap().unwrap().code(), Some(37));
}

#[test]
fn commands_from_different_directories_share_the_limit() {
    let home = tempdir().unwrap();
    let other = tempdir().unwrap();
    let mut first = workload(home.path(), home.path());
    ready(&lines(first.stdout.take().unwrap()));
    let mut second = workload(home.path(), other.path());
    let waiting = lines(second.stderr.take().unwrap())
        .recv_timeout(TIMEOUT)
        .unwrap();
    assert!(waiting.contains("waiting"), "{waiting}");
    assert!(waiting.contains(&first.id().to_string()), "{waiting}");
    let status = success(bulky(home.path()).arg("--status").output().unwrap());
    assert!(status.contains(&first.id().to_string()));
    finish(first);
    ready(&lines(second.stdout.take().unwrap()));
    finish(second);
}

#[test]
fn changes_to_the_limit_persist_and_cannot_displace_active_slots() {
    let home = tempdir().unwrap();
    let run = |args: &[&str]| bulky(home.path()).args(args).output().unwrap();
    success(run(&["--set-limit", "2"]));
    let mut first = workload(home.path(), home.path());
    ready(&lines(first.stdout.take().unwrap()));
    let mut second = workload(home.path(), home.path());
    ready(&lines(second.stdout.take().unwrap()));
    assert!(!run(&["--set-limit", "1"]).status.success());
    assert!(success(run(&["--status"])).contains("2/2"));
    finish(second);
    success(run(&["--set-limit", "1"]));
    finish(first);
    assert!(success(run(&["--status"])).contains("0/1"));
    assert!(!run(&["--set-limit", "0"]).status.success());
}

#[test]
fn nested_invocations_share_the_outer_slot() {
    let home = tempdir().unwrap();
    let mut child = bulky(home.path())
        .arg("--")
        .arg(BIN)
        .arg("--")
        .arg(BIN)
        .arg("--version")
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    assert_eq!(
        lines(child.stdout.take().unwrap())
            .recv_timeout(TIMEOUT)
            .unwrap(),
        concat!("bulky ", env!("CARGO_PKG_VERSION"))
    );
    assert!(child.wait().unwrap().success());
}

#[test]
fn a_failed_launch_releases_the_slot_and_keeps_arguments_private() {
    let home = tempdir().unwrap();
    let run = |args: &[&str]| bulky(home.path()).args(args).output().unwrap();
    let failed = run(&[
        "--",
        "bulky-test-command-does-not-exist",
        "private-argument",
    ]);
    assert_eq!(failed.status.code(), Some(127));
    assert!(!String::from_utf8_lossy(&failed.stderr).contains("private-argument"));
    assert!(success(run(&["--status"])).contains("0/1"));
    assert!(
        !std::fs::read_to_string(home.path().join(".bulky/1.owner"))
            .unwrap()
            .contains("private-argument")
    );
}

#[cfg(unix)]
#[test]
fn an_interrupt_reaches_the_child_and_releases_the_slot() {
    use nix::sys::signal::{Signal, kill};
    use nix::unistd::Pid;
    let home = tempdir().unwrap();
    let mut child = workload(home.path(), home.path());
    ready(&lines(child.stdout.take().unwrap()));
    kill(Pid::from_raw(child.id() as i32), Signal::SIGTERM).unwrap();
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || tx.send(child.wait()).unwrap());
    assert_eq!(rx.recv_timeout(TIMEOUT).unwrap().unwrap().code(), Some(143));
    assert!(success(bulky(home.path()).arg("--status").output().unwrap()).contains("0/1"));
}

#[cfg(unix)]
#[test]
fn refuses_a_redirected_pool_directory() {
    let home = tempdir().unwrap();
    let other = tempdir().unwrap();
    std::os::unix::fs::symlink(other.path(), home.path().join(".bulky")).unwrap();
    let out = bulky(home.path())
        .args(["--", BIN, "--version"])
        .output()
        .unwrap();
    assert!(!out.status.success());
    assert!(out.stdout.is_empty());
    assert_eq!(other.path().read_dir().unwrap().count(), 0);
}

#[test]
fn stale_nesting_context_does_not_skip_the_queue() {
    let home = tempdir().unwrap();
    let mut first = workload(home.path(), home.path());
    ready(&lines(first.stdout.take().unwrap()));
    let mut second = bulky(home.path())
        .env("BULKY_HELD", "1:expired")
        .args(["--", BIN, "--version"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    assert!(
        lines(second.stderr.take().unwrap())
            .recv_timeout(TIMEOUT)
            .unwrap()
            .contains("waiting")
    );
    finish(first);
    assert!(
        lines(second.stdout.take().unwrap())
            .recv_timeout(TIMEOUT)
            .unwrap()
            .starts_with("bulky ")
    );
    assert!(second.wait().unwrap().success());
}

#[cfg(windows)]
#[test]
fn preserves_a_full_windows_exit_code() {
    let home = tempdir().unwrap();
    let output = bulky(home.path())
        .args(["--", "cmd.exe", "/d", "/c", "exit 513"])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(513));
}
