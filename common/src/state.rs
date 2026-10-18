//! The state an NF keeps across restarts: one file per NF, `<dir>/<name>.json`,
//! in the directory `[oam] state_dir` names (none: nothing is kept). The NF
//! reads it when it is created (off the workers), and writes it as a
//! snapshot when it changed, at most every [`SNAPSHOT_PERIOD`]: a temporary
//! file, then a rename, so a crash never leaves half a file. Readable by its
//! owner only: it may hold keys.
//!
//! The format is the NF's (JSON by convention); this module only moves text.

use std::fs;
use std::io::{self, Write};
use std::os::unix::fs::OpenOptionsExt;
use std::path::PathBuf;
use std::sync::OnceLock;
use std::time::Duration;

/// A changed state is written at most this often.
pub const SNAPSHOT_PERIOD: Duration = Duration::from_secs(5);

static DIR: OnceLock<PathBuf> = OnceLock::new();

/// Sets the process's state directory (once, at start).
pub fn set_dir(dir: PathBuf) {
    let _ = DIR.set(dir);
}

/// The state file of one NF.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Store {
    path: PathBuf,
}

impl Store {
    /// NF `name`'s file in the process's state directory; `None` without one.
    pub fn for_nf(name: &str) -> Option<Store> {
        DIR.get().map(|dir| Store::at(dir.join(format!("{name}.json"))))
    }

    /// A file of its own (tests).
    pub fn at(path: PathBuf) -> Store {
        Store { path }
    }

    pub fn path(&self) -> &std::path::Path {
        &self.path
    }

    /// The saved text; `None` if nothing was saved yet.
    pub fn load(&self) -> io::Result<Option<String>> {
        match fs::read_to_string(&self.path) {
            Ok(text) => Ok(Some(text)),
            Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(e),
        }
    }

    /// Replaces the saved text, atomically.
    pub fn save(&self, text: &str) -> io::Result<()> {
        if let Some(dir) = self.path.parent() {
            fs::create_dir_all(dir)?;
        }
        let tmp = self.path.with_extension("json.tmp");
        let mut file = fs::OpenOptions::new().write(true).create(true).truncate(true).mode(0o600).open(&tmp)?;
        file.write_all(text.as_bytes())?;
        file.sync_all()?;
        fs::rename(&tmp, &self.path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn saved_text_comes_back_and_only_its_owner_reads_it() {
        use std::os::unix::fs::PermissionsExt;
        let dir = std::env::temp_dir().join(format!("librecn-state-{}", std::process::id()));
        let store = Store::at(dir.join("udm.json"));
        assert_eq!(store.load().unwrap(), None, "nothing saved yet");
        store.save("{\"a\":1}").unwrap();
        store.save("{\"a\":2}").unwrap();
        assert_eq!(store.load().unwrap().as_deref(), Some("{\"a\":2}"));
        assert_eq!(fs::metadata(store.path()).unwrap().permissions().mode() & 0o777, 0o600);
        assert!(!dir.join("udm.json.tmp").exists(), "renamed");
        fs::remove_dir_all(dir).unwrap();
    }
}
