//! Word-aware reflow without deleting, inserting or reindexing source characters.
use std::ops::Range;
use unicode_width::UnicodeWidthChar;
pub fn lines(chars: &[char], width: usize) -> Vec<Range<usize>> {
    let width = width.max(1);
    let mut lines = Vec::new();
    let mut start = 0;
    let mut i = 0;
    let mut cells = 0;
    while i < chars.len() {
        let c = chars[i];
        if c == '\n' {
            lines.push(start..i + 1);
            i += 1;
            start = i;
            cells = 0;
            continue;
        }
        if !c.is_whitespace() && (i == 0 || chars[i - 1].is_whitespace()) {
            let word_cells = chars[i..]
                .iter()
                .take_while(|c| !c.is_whitespace())
                .map(|c| c.width().unwrap_or(0))
                .sum::<usize>();
            if cells > 0 && cells + word_cells > width.saturating_sub(1) {
                lines.push(start..i);
                start = i;
                cells = 0;
            }
        }
        let w = c.width().unwrap_or(0);
        if cells + w > width && i > start {
            lines.push(start..i);
            start = i;
            cells = 0;
        }
        cells += w;
        i += 1;
    }
    if start < chars.len() {
        lines.push(start..chars.len());
    }
    if lines.is_empty() {
        lines.push(0..0);
    }
    lines
}
#[derive(Default)]
pub struct Reflow {
    width: usize,
    len: usize,
    pub rows: Vec<Range<usize>>,
}
impl Reflow {
    pub fn update(&mut self, chars: &[char], width: usize) {
        if self.width != width || self.len != chars.len() || self.rows.is_empty() {
            self.width = width;
            self.len = chars.len();
            self.rows = lines(chars, width);
        }
    }
    pub fn clear(&mut self) {
        self.rows.clear();
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn words_stay_whole() {
        let chars: Vec<_> = "the quick brown fox".chars().collect();
        let rows = lines(&chars, 10);
        let text: Vec<String> = rows
            .iter()
            .map(|r| chars[r.clone()].iter().collect())
            .collect();
        assert_eq!(text, vec!["the quick ", "brown fox"]);
    }
    #[test]
    fn preserves_spaces_newlines_and_wide_characters() {
        for s in [
            "one  two\n    three\n",
            "école 世界 hello",
            "abcdefghijklmnop",
            "a\n\nb",
            "abc def",
        ] {
            let chars: Vec<_> = s.chars().collect();
            for width in 2..20 {
                let rows = lines(&chars, width);
                let restored: String = rows.iter().flat_map(|r| chars[r.clone()].iter()).collect();
                assert_eq!(restored, s);
                for r in rows {
                    let cells = chars[r]
                        .iter()
                        .map(|c| c.width().unwrap_or(0))
                        .sum::<usize>();
                    assert!(cells <= width);
                }
            }
        }
    }
    #[test]
    fn resizing_reflows_without_reindexing() {
        let c: Vec<_> = "hello world today".chars().collect();
        let mut cache = Reflow::default();
        cache.update(&c, 12);
        assert_eq!(cache.rows[0], 0..12);
        cache.update(&c, 6);
        assert_eq!(cache.rows[0], 0..6);
        assert!(cache.rows.iter().any(|r| r.contains(&7)));
    }
}
