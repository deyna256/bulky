use std::ffi::OsStr;
use std::fs::{self, File};
use std::io::{self, ErrorKind, Write};
use std::path::{Path, PathBuf};
use std::time::Duration;
use tempfile::NamedTempFile;

pub(crate) const MAX_LIMIT: u16 = 256;
pub(crate) const HELD: &str = "BULKY_HELD";

pub(crate) struct Pool {
    dir: PathBuf,
}

pub(crate) struct Admission {
    pub token: String,
    _lock: Option<File>,
}

fn lock_file(path: &Path) -> io::Result<File> {
    File::options()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(path)
}

fn try_lock(file: File) -> io::Result<Option<File>> {
    match file.try_lock() {
        Ok(()) => Ok(Some(file)),
        Err(fs::TryLockError::WouldBlock) => Ok(None),
        Err(fs::TryLockError::Error(error)) => Err(error),
    }
}

fn private_dir(home: &Path) -> io::Result<PathBuf> {
    if !home.is_absolute() {
        return Err(io::Error::other("expected an absolute home directory"));
    }
    let dir = home.join(".bulky");
    let builder = fs::DirBuilder::new();
    #[cfg(unix)]
    let builder = {
        use std::os::unix::fs::{DirBuilderExt, MetadataExt};
        let parent = fs::metadata(home)?;
        if parent.uid() != nix::unistd::Uid::effective().as_raw() || parent.mode() & 0o022 != 0 {
            return Err(io::Error::other(
                "expected a home directory writable only by its owner",
            ));
        }
        let mut builder = builder;
        builder.mode(0o700);
        builder
    };
    match builder.create(&dir) {
        Err(e) if e.kind() == ErrorKind::AlreadyExists => {}
        result => result?,
    }
    let metadata = fs::symlink_metadata(&dir)?;
    if !metadata.is_dir() {
        return Err(io::Error::other("expected .bulky to be a directory"));
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        if metadata.uid() != nix::unistd::Uid::effective().as_raw() || metadata.mode() & 0o077 != 0
        {
            return Err(io::Error::other(
                "expected .bulky to be private to the home owner (mode 0700)",
            ));
        }
    }
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        if metadata.file_attributes() & 0x400 != 0 {
            return Err(io::Error::other(
                "expected .bulky to be a local directory, not a reparse point",
            ));
        }
    }
    Ok(dir)
}

impl Pool {
    pub fn open() -> io::Result<Self> {
        let home = std::env::home_dir()
            .ok_or_else(|| io::Error::other("cannot find the user home directory"))?;
        Ok(Self {
            dir: private_dir(&home)?,
        })
    }

    fn gate(&self) -> io::Result<File> {
        let file = lock_file(&self.dir.join("pool.lock"))?;
        file.lock()?;
        Ok(file)
    }

    fn limit(&self) -> io::Result<u16> {
        match fs::read_to_string(self.dir.join("limit")) {
            Ok(text) => text
                .trim()
                .parse()
                .ok()
                .filter(|n| (1..=MAX_LIMIT).contains(n))
                .ok_or_else(|| io::Error::other("invalid saved slot limit")),
            Err(e) if e.kind() == ErrorKind::NotFound => Ok(1),
            Err(e) => Err(e),
        }
    }

    fn slot_path(&self, n: u16) -> PathBuf {
        self.dir.join(format!("{n}.lock"))
    }

    fn is_busy(&self, n: u16) -> io::Result<bool> {
        let path = self.slot_path(n);
        match File::options().read(true).write(true).open(path) {
            Ok(file) => Ok(try_lock(file)?.is_none()),
            Err(e) if e.kind() == ErrorKind::NotFound => Ok(false),
            Err(e) => Err(e),
        }
    }

    fn owner(&self, n: u16) -> PathBuf {
        self.dir.join(format!("{n}.owner"))
    }

    pub fn set_limit(&self, limit: u16) -> io::Result<()> {
        let _gate = self.gate()?;
        for n in limit + 1..=self.limit()? {
            if self.is_busy(n)? {
                return Err(io::Error::other(format!(
                    "slot {n} is busy; retry when its command finishes"
                )));
            }
        }
        let mut tmp = NamedTempFile::new_in(&self.dir)?;
        writeln!(tmp, "{limit}")?;
        tmp.persist(self.dir.join("limit"))?;
        Ok(())
    }

    fn holders(&self, limit: u16) -> io::Result<Vec<String>> {
        let mut holders = Vec::new();
        for n in 1..=limit {
            if self.is_busy(n)? {
                let owner = fs::read_to_string(self.owner(n))?;
                let description = owner
                    .lines()
                    .nth(1)
                    .ok_or_else(|| io::Error::other("invalid slot owner"))?;
                holders.push(format!("slot {n}: {description}"));
            }
        }
        Ok(holders)
    }

    pub fn status(&self) -> io::Result<String> {
        let _gate = self.gate()?;
        let limit = self.limit()?;
        let holders = self.holders(limit)?;
        let mut lines = vec![format!("{}/{limit} slots busy", holders.len())];
        lines.extend(holders);
        Ok(lines.join("\n"))
    }

    fn inherited(&self, token: &str, limit: u16) -> io::Result<bool> {
        let slot = token
            .split_once(':')
            .and_then(|(n, _)| n.parse::<u16>().ok());
        let Some(n) = slot.filter(|n| (1..=limit).contains(n)) else {
            return Ok(false);
        };
        Ok(self.is_busy(n)? && fs::read_to_string(self.owner(n))?.lines().next() == Some(token))
    }

    fn claim(&self, n: u16, lock: File, program: &OsStr) -> io::Result<Admission> {
        let mut tmp = NamedTempFile::new_in(&self.dir)?;
        let token = format!(
            "{n}:{}",
            tmp.path().file_name().unwrap_or_default().to_string_lossy()
        );
        let name = Path::new(program).file_name().unwrap_or(program);
        writeln!(tmp, "{token}\nPID {} {name:?}", std::process::id())?;
        tmp.persist(self.owner(n))?;
        Ok(Admission {
            token,
            _lock: Some(lock),
        })
    }

    pub fn acquire(&self, program: &OsStr) -> io::Result<Admission> {
        let mut inherited = std::env::var(HELD).ok();
        let mut announce = true;
        loop {
            let gate = self.gate()?;
            let limit = self.limit()?;
            if let Some(token) = inherited.take()
                && self.inherited(&token, limit)?
            {
                return Ok(Admission { token, _lock: None });
            }
            for n in 1..=limit {
                let file = lock_file(&self.slot_path(n))?;
                if let Some(lock) = try_lock(file)? {
                    return self.claim(n, lock, program);
                }
            }
            let holders = if announce {
                Some(self.holders(limit)?)
            } else {
                None
            };
            drop(gate);
            if let Some(holders) = holders {
                if holders.is_empty() {
                    continue;
                }
                eprintln!("bulky: waiting ({})", holders.join(", "));
                announce = false;
            }
            std::thread::sleep(Duration::from_millis(100));
        }
    }
}
