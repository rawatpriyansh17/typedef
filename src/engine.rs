//! Pure typing/scoring logic. Time is injected so tests never sleep.
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, clap::ValueEnum)]
pub enum Mode {
    Time,
    Words,
    Quote,
    Code,
    Endless,
    Zen,
    Daily,
    Weak,
    Custom,
}
impl Mode {
    pub const ALL: [Self; 9] = [
        Self::Time,
        Self::Words,
        Self::Quote,
        Self::Endless,
        Self::Zen,
        Self::Daily,
        Self::Weak,
        Self::Custom,
        Self::Code,
    ];
    pub fn endless(self) -> bool {
        matches!(self, Self::Endless | Self::Zen)
    }
    pub fn icon(self) -> &'static str {
        match self {
            Self::Time => "◷",
            Self::Words => "≡",
            Self::Quote => "“",
            Self::Endless => "∞",
            Self::Zen => "◇",
            Self::Daily => "◈",
            Self::Weak => "⌁",
            Self::Custom => "↥",
            Self::Code => "λ",
        }
    }
    pub fn description(self) -> &'static str {
        match self {
            Self::Time => {
                "Race the clock. The timer starts with your first key; duration is set in Practice settings."
            }
            Self::Words => {
                "Finish a fixed word count at your own pace. Speed and accuracy are scored when you finish."
            }
            Self::Quote => {
                "Type a complete passage, including its capitals and punctuation. Each test picks a bundled quote."
            }
            Self::Endless => {
                "An unlimited stream of words with live stats and 50-word laps. New words keep arriving. Ctrl+D saves and ends."
            }
            Self::Zen => {
                "Unlimited guided words with a quiet, focused screen. No timer or word limit. Ctrl+D saves and ends."
            }
            Self::Daily => {
                "A shared 60-second challenge based on today's UTC date. Everyone on the same version gets the same seed."
            }
            Self::Weak => {
                "Practice words containing your most error-prone keys. Starts with regular words until you have error history."
            }
            Self::Custom => {
                "Practice your own text or source file. Press O to browse or paste a path. UTF-8, up to 1 MiB; nothing is uploaded."
            }
            Self::Code => {
                "Practice a Rust, Python or JavaScript snippet. Enter types a newline; Tab fills up to four leading indentation spaces."
            }
        }
    }
    pub fn label(self) -> &'static str {
        match self {
            Self::Time => "time",
            Self::Words => "words",
            Self::Quote => "quote",
            Self::Code => "code",
            Self::Endless => "endless",
            Self::Zen => "zen",
            Self::Daily => "daily",
            Self::Weak => "weak keys",
            Self::Custom => "custom",
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum Difficulty {
    #[default]
    Normal,
    Expert,
    Master,
}
impl Difficulty {
    pub fn label(self) -> &'static str {
        match self {
            Self::Normal => "normal",
            Self::Expert => "expert",
            Self::Master => "master",
        }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub mode: Mode,
    pub seconds: u64,
    pub words: usize,
    pub punctuation: bool,
    pub numbers: bool,
    pub difficulty: Difficulty,
    pub theme: usize,
    pub caret: usize,
    pub line_gap: usize,
    pub text_width: usize,
    pub show_keyboard: bool,
    pub keyboard_layout: usize,
    pub live_stats: bool,
    pub focus: bool,
    pub error_flash: bool,
    pub accent: usize,
    pub panel_style: usize,
    pub key_style: usize,
    pub graph_style: usize,
    pub bold_text: bool,
    pub visible_lines: usize,
    pub large_results: bool,
    pub celebrations: bool,
    pub blind: bool,
    pub no_backspace: bool,
    pub stop_on_error: bool,
    pub pace: usize,
    pub daily_goal: usize,
    pub tab_restart: bool,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            mode: Mode::Time,
            seconds: 30,
            words: 25,
            punctuation: false,
            numbers: false,
            difficulty: Difficulty::Normal,
            theme: 0,
            caret: 0,
            line_gap: 1,
            text_width: 80,
            show_keyboard: true,
            keyboard_layout: 0,
            live_stats: true,
            focus: false,
            error_flash: false,
            accent: 0,
            panel_style: 0,
            key_style: 0,
            graph_style: 0,
            bold_text: false,
            visible_lines: 5,
            large_results: true,
            celebrations: true,
            blind: false,
            no_backspace: false,
            stop_on_error: false,
            pace: 0,
            daily_goal: 3,
            tab_restart: false,
        }
    }
}
impl Settings {
    pub fn sanitize(&mut self) {
        self.seconds = self.seconds.clamp(1, 3600);
        self.words = self.words.clamp(1, 10000);
        self.theme %= crate::theme::THEMES.len();
        self.caret %= 3;
        self.line_gap = self.line_gap.min(2);
        if self.text_width != 0 {
            self.text_width = self.text_width.clamp(40, 240);
        }
        self.keyboard_layout %= 3;
        self.accent %= 6;
        self.panel_style %= 4;
        self.key_style %= 3;
        self.graph_style %= 3;
        self.visible_lines = self.visible_lines.clamp(1, 9);
        self.pace = self.pace.min(250);
        self.daily_goal = self.daily_goal.clamp(1, 20);
        if self.blind || self.no_backspace {
            self.stop_on_error = false;
        }
    }
}
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct KeyStat {
    pub hits: u64,
    pub errors: u64,
}
#[derive(Debug, Clone)]
pub struct Session {
    pub target: Vec<char>,
    pub typed: Vec<char>,
    pub settings: Settings,
    pub started: bool,
    pub finished: bool,
    pub failed: bool,
    pub elapsed: f64,
    pub attempts: u64,
    pub correct_attempts: u64,
    pub combo: u64,
    pub best_combo: u64,
    pub keys: BTreeMap<char, KeyStat>,
    pub samples: Vec<u64>,
    pub pace_points: Vec<(f64, f64)>,
    pub raw_points: Vec<(f64, f64)>,
    pub mistake_positions: std::collections::BTreeSet<usize>,
    last_sample: u64,
}
impl Session {
    pub fn new(text: String, settings: Settings) -> Self {
        Self {
            target: text.chars().collect(),
            typed: vec![],
            settings,
            started: false,
            finished: false,
            failed: false,
            elapsed: 0.,
            attempts: 0,
            correct_attempts: 0,
            combo: 0,
            best_combo: 0,
            keys: BTreeMap::new(),
            samples: vec![],
            pace_points: vec![],
            raw_points: vec![],
            mistake_positions: Default::default(),
            last_sample: 0,
        }
    }
    pub fn timed(&self) -> bool {
        matches!(self.settings.mode, Mode::Time | Mode::Daily)
    }
    pub fn tick(&mut self, elapsed: f64) {
        if !self.started || self.finished {
            return;
        }
        self.elapsed = if self.timed() {
            elapsed.min(self.settings.seconds as f64)
        } else {
            elapsed
        };
        if self.elapsed as u64 > self.last_sample {
            self.last_sample = self.elapsed as u64;
            self.samples.push(self.wpm() as u64);
            self.pace_points.push((self.elapsed, self.wpm()));
            self.raw_points.push((self.elapsed, self.raw()));
        }
        if self.timed() && elapsed >= self.settings.seconds as f64 {
            self.finished = true;
        }
    }
    pub fn input(&mut self, c: char) {
        if self.finished || self.typed.len() >= self.target.len() {
            return;
        }
        self.started = true;
        self.attempts += 1;
        let expected = self.target[self.typed.len()];
        let good = c == expected;
        let stat = self.keys.entry(expected).or_default();
        stat.hits += 1;
        if good {
            self.correct_attempts += 1;
            self.combo += 1;
            self.best_combo = self.best_combo.max(self.combo);
        } else {
            stat.errors += 1;
            self.mistake_positions.insert(self.typed.len());
            self.combo = 0;
        }
        if good || !self.settings.stop_on_error {
            self.typed.push(c);
        }
        if (!good && self.settings.difficulty == Difficulty::Master)
            || (c == ' '
                && self.settings.difficulty == Difficulty::Expert
                && self.typed.iter().zip(&self.target).any(|(a, b)| a != b))
        {
            self.failed = true;
            self.finished = true;
        }
        if self.typed.len() == self.target.len() && !self.timed() && !self.settings.mode.endless() {
            self.finished = true;
        }
    }
    /// Only expand indentation actually present at the start of the target line.
    pub fn indent(&mut self) -> usize {
        if self.finished || !matches!(self.settings.mode, Mode::Code | Mode::Custom) {
            return 0;
        }
        let pos = self.typed.len();
        let line_start = self.target[..pos]
            .iter()
            .rposition(|c| *c == '\n')
            .map_or(0, |i| i + 1);
        if !self.target[line_start..pos].iter().all(|c| *c == ' ') {
            return 0;
        }
        let count = self.target[pos..]
            .iter()
            .take_while(|c| **c == ' ')
            .count()
            .min(4 - ((pos - line_start) % 4));
        for _ in 0..count {
            self.input(' ');
        }
        count
    }
    pub fn backspace(&mut self) {
        if !self.finished && !self.settings.no_backspace {
            self.typed.pop();
            self.combo = 0;
        }
    }
    pub fn delete_word(&mut self) {
        if self.finished || self.settings.no_backspace {
            return;
        }
        while self.typed.last() == Some(&' ') {
            self.typed.pop();
        }
        while self.typed.last().is_some_and(|c| *c != ' ') {
            self.typed.pop();
        }
        self.combo = 0;
    }
    pub fn missed_words(&self) -> Vec<String> {
        let mut words = Vec::new();
        let mut start = 0;
        for end in 0..=self.target.len() {
            if end == self.target.len() || self.target[end].is_whitespace() {
                if self.mistake_positions.range(start..end).next().is_some() {
                    let word: String = self.target[start..end].iter().collect();
                    if !word.is_empty() && !words.contains(&word) {
                        words.push(word);
                    }
                }
                start = end + 1;
            }
        }
        words
    }
    pub fn correct(&self) -> usize {
        self.typed
            .iter()
            .zip(&self.target)
            .filter(|(a, b)| a == b)
            .count()
    }
    pub fn wpm(&self) -> f64 {
        if self.elapsed < 0.1 {
            0.
        } else {
            self.correct() as f64 * 12. / self.elapsed
        }
    }
    pub fn raw(&self) -> f64 {
        if self.elapsed < 0.1 {
            0.
        } else {
            self.attempts as f64 * 12. / self.elapsed
        }
    }
    pub fn accuracy(&self) -> f64 {
        if self.attempts == 0 {
            100.
        } else {
            100. * self.correct_attempts as f64 / self.attempts as f64
        }
    }
    pub fn consistency(&self) -> f64 {
        if self.samples.is_empty() {
            return 100.;
        }
        let mean = self.samples.iter().sum::<u64>() as f64 / self.samples.len() as f64;
        if mean == 0. {
            return 0.;
        }
        let variance = self
            .samples
            .iter()
            .map(|x| (*x as f64 - mean).powi(2))
            .sum::<f64>()
            / self.samples.len() as f64;
        (100. * (1. - variance.sqrt() / mean)).clamp(0., 100.)
    }
    pub fn progress(&self) -> f64 {
        if self.settings.mode.endless() {
            (self.typed.iter().filter(|c| **c == ' ').count() % 50) as f64 / 50.
        } else if self.timed() {
            self.elapsed / self.settings.seconds as f64
        } else {
            self.typed.len() as f64 / self.target.len().max(1) as f64
        }
        .clamp(0., 1.)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn tab_expands_only_leading_indentation_without_swallowing_code() {
        let mut s = Session::new(
            "      x y".into(),
            Settings {
                mode: Mode::Code,
                ..Settings::default()
            },
        );
        assert_eq!(s.indent(), 4);
        assert_eq!(s.indent(), 2);
        assert_eq!(s.indent(), 0);
        s.input('x');
        assert_eq!(s.indent(), 0);
        assert_eq!(s.attempts, 7);
        assert_eq!(s.accuracy(), 100.);
        let mut normal = Session::new("    x".into(), Settings::default());
        assert_eq!(normal.indent(), 0);
        assert!(!normal.started);
    }
    #[test]
    fn corrections_do_not_erase_errors() {
        let mut s = Session::new("abc".into(), Settings::default());
        s.input('x');
        s.backspace();
        s.input('a');
        s.tick(6.);
        assert_eq!(s.accuracy(), 50.);
        assert_eq!(s.wpm(), 2.);
        assert_eq!(s.raw(), 4.);
    }
    #[test]
    fn deadline_freezes_score() {
        let mut s = Session::new(
            "abc".into(),
            Settings {
                seconds: 15,
                ..Settings::default()
            },
        );
        s.input('a');
        s.tick(20.);
        s.input('b');
        assert!(s.finished);
        assert_eq!(s.elapsed, 15.);
        assert_eq!(s.typed.len(), 1);
    }
    #[test]
    fn master_fails_immediately() {
        let mut s = Session::new(
            "abc".into(),
            Settings {
                difficulty: Difficulty::Master,
                ..Settings::default()
            },
        );
        s.input('z');
        assert!(s.failed);
    }
    #[test]
    fn unicode_counts_characters() {
        let mut s = Session::new(
            "é界".into(),
            Settings {
                mode: Mode::Custom,
                ..Settings::default()
            },
        );
        s.input('é');
        s.input('界');
        assert!(s.finished);
        assert_eq!(s.correct(), 2);
    }
    #[test]
    fn expert_allows_correction() {
        let mut s = Session::new(
            "ab cd".into(),
            Settings {
                difficulty: Difficulty::Expert,
                ..Settings::default()
            },
        );
        s.input('x');
        s.backspace();
        for c in "ab ".chars() {
            s.input(c)
        }
        assert!(!s.failed);
    }
    #[test]
    fn word_deletion() {
        let mut s = Session::new("one two three".into(), Settings::default());
        for c in "one two ".chars() {
            s.input(c)
        }
        s.delete_word();
        assert_eq!(s.typed.iter().collect::<String>(), "one ");
    }
}

#[cfg(test)]
mod training_tests {
    use super::*;
    #[test]
    fn stop_on_error_records_attempt_without_advancing() {
        let mut s = Session::new(
            "hi".into(),
            Settings {
                mode: Mode::Words,
                stop_on_error: true,
                ..Settings::default()
            },
        );
        s.input('x');
        assert!(s.typed.is_empty());
        s.input('h');
        s.input('i');
        assert!(s.finished);
        assert_eq!(s.attempts, 3);
        assert!((s.accuracy() - 200. / 3.).abs() < 0.001);
    }
    #[test]
    fn confidence_blocks_both_deletions() {
        let mut s = Session::new(
            "hello world".into(),
            Settings {
                no_backspace: true,
                ..Settings::default()
            },
        );
        for c in "hello ".chars() {
            s.input(c);
        }
        s.backspace();
        s.delete_word();
        assert_eq!(s.typed.len(), 6);
    }
    #[test]
    fn corrected_mistakes_still_get_word_review() {
        let mut s = Session::new("hello world".into(), Settings::default());
        s.input('x');
        s.backspace();
        for c in "hello world".chars() {
            s.input(c);
        }
        assert_eq!(s.missed_words(), vec!["hello"]);
    }
    #[test]
    fn chart_points_keep_real_elapsed_time() {
        let mut s = Session::new("hello world".into(), Settings::default());
        s.input('h');
        s.tick(1.4);
        s.input('e');
        s.tick(3.2);
        assert_eq!(s.pace_points[0].0, 1.4);
        assert_eq!(s.pace_points[1].0, 3.2);
        assert_eq!(s.raw_points.len(), 2);
    }
}
