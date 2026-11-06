//! Local file selection. Only a successful, bounded read replaces the cached custom text.
use std::{io, path::PathBuf};
#[derive(Default)]
pub struct Picker {
    pub dir: PathBuf,
    pub entries: Vec<PathBuf>,
    pub selected: usize,
    pub input: String,
    pub error: String,
}
impl Picker {
    pub fn open(&mut self, dir: PathBuf) -> io::Result<()> {
        let mut entries: Vec<_> = std::fs::read_dir(&dir)?
            .filter_map(Result::ok)
            .map(|e| e.path())
            .collect();
        entries.sort_by_key(|p| {
            (
                !p.is_dir(),
                p.file_name().unwrap_or_default().to_ascii_lowercase(),
            )
        });
        if let Some(parent) = dir.parent() {
            entries.insert(0, parent.to_owned());
        }
        self.dir = dir;
        self.entries = entries;
        self.selected = 0;
        self.input.clear();
        self.error.clear();
        Ok(())
    }
    pub fn path(&self) -> PathBuf {
        let input = self.input.trim().trim_matches(['\'', '"']);
        if input.is_empty() {
            return self
                .entries
                .get(self.selected)
                .cloned()
                .unwrap_or_else(|| self.dir.clone());
        }
        if let Some(rest) = input.strip_prefix("~/")
            && let Some(home) = directories::BaseDirs::new()
        {
            return home.home_dir().join(rest);
        }
        self.dir.join(input)
    }
}
