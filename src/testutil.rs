use std::sync::atomic::{AtomicU64, Ordering};

static COUNTER: AtomicU64 = AtomicU64::new(0);

/// RAII temporary file. Written on creation, deleted on drop.
pub struct TempFile(std::path::PathBuf);

impl TempFile {
    pub fn write(content: &str) -> Self {
        let id = COUNTER.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!("aoc_test_{id}.txt"));
        std::fs::write(&path, content).unwrap();
        TempFile(path)
    }

    pub fn path(&self) -> &str {
        self.0.to_str().unwrap()
    }
}

impl Drop for TempFile {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}
